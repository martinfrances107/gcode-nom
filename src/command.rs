use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::number::complete::double;
use nom::sequence::preceded;
use nom::IResult;

use crate::pos::parse_a;
use crate::pos::parse_b;
use crate::pos::parse_c;
use crate::pos::parse_e;
use crate::pos::parse_f;
use crate::pos::parse_s;
use crate::pos::parse_u;
use crate::pos::parse_v;
use crate::pos::parse_w;
use crate::pos::parse_x;
use crate::pos::parse_y;
use crate::pos::parse_z;
use crate::pos::PosVal;

/// Commands: -
///
/// "The G0 and G1 commands add a linear move to the queue to be performed after all previous moves are completed."
/// [GCODE doc](<https://marlinfw.org/docs/gcode/G000-G001.html>)
///
/// Missing Commands :-
///  "bezier"
///  ... TODO maybe more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    ///  "G0 for non-print moves. It makes G-code more adaptable to lasers, engravers, etc."
    G0(HashSet<PosVal>),
    /// Printable move
    G1(HashSet<PosVal>),
    /// Home all axes
    G21,
    // "G90 ; Set all axes to absolute"
    G90,
    // "G91 ; Set all axes to relative"
    G91,
    // Set the current position to the values specified.
    // eg. "G92 E0"
    // TODO:  F and S are not permitted here.
    G92(HashSet<PosVal>),
    /// Drop - ie no further action.
    GDrop(u16),
    MDrop(u16),

    /// No Operation eg a blank line "".
    Nop,
}

impl Command {
    pub fn parse_line(line: &str) -> IResult<&str, Self> {
        // Most common first.
        alt((
            parse_g1,
            map(tag("G21 "), |_| Self::G21),
            map(tag("G90 "), |_| Self::G90),
            map(tag("G91 "), |_| Self::G91),
            parse_g92,
            // Command::G0 - Non printing moves are infrequent.
            //  eg "The benchy" example has none.
            parse_g0,
            // Dropping "bed leveling", "dock sled", "Retract", "Stepper motor", "Mechanical Gantry Calibration"
            map(g_drop, Self::GDrop),
            map(m_drop, Self::MDrop),
            map(tag(";"), |_| Self::Nop),
            map(tag(""), |_| Self::Nop),
        ))(line)
    }
}

// G commands that require no further action
pub fn g_drop(i: &str) -> IResult<&str, u16> {
    map(preceded(tag("G"), double), |val| {
        let out: u16 = val as u16;
        out
    })(i)
}

fn parse_g0(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G0 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G0(hs)
        }),
    )(i)
}

fn parse_g1(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G1 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G1(hs)
        }),
    )(i)
}

fn parse_g92(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G92 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f63> value.
            let hs = HashSet::from_iter(vals);
            Command::G92(hs)
        }),
    )(i)
}

fn pos_many(i: &str) -> IResult<&str, Vec<PosVal>> {
    separated_list1(tag(" "), pos_val)(i)
}

fn pos_val(i: &str) -> IResult<&str, PosVal> {
    alt((
        parse_a, parse_b, parse_c, parse_e, parse_f, parse_s, parse_u, parse_v, parse_w, parse_x,
        parse_y, parse_z,
    ))(i)
}

pub fn m_drop(i: &str) -> IResult<&str, u16> {
    map(preceded(tag("M"), double), |val| {
        let out: u16 = val as u16;
        out
    })(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    const fn parse_comment() {}

    #[test]
    fn g1() {
        // let default = PosPayload::<f64>::default();

        let text_commands = [
            ("G1 Z5", Command::G1([PosVal::Z(5_f64)].into())),
            (
                "G1 Z5 F5000 ; lift nozzle",
                Command::G1([PosVal::Z(5_f64), PosVal::F(5000_f64)].into()),
            ),
            (
                "G1 E1.00000 F1800.00000 ; text",
                Command::G1([PosVal::E(1.0_f64), PosVal::F(1800_f64)].into()),
            ),
            (
                "G1 Z0.350 F7800.000",
                Command::G1([PosVal::Z(0.350_f64), PosVal::F(7800_f64)].into()),
            ),
        ];

        for (line, expected) in text_commands {
            let actual = parse_g1(line);
            println!("expected  {expected:?}");
            println!("actual  {actual:?}");
            match actual {
                Ok((_, actual)) => {
                    assert_eq!(actual, expected);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }
    }
    #[test]
    const fn parse_g_drop() {}
}
