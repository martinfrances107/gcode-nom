use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::multi::separated_list1;
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command<'a> {
    G1(HashSet<PosVal>),
    /// Home all axes
    G21,
    /// use absolute coordinates.
    G90,
    // use relative position
    G91,
    // set position
    G92(HashSet<PosVal>),
    /// Drop - ie no further action.
    GDrop(&'a str),
    MDrop(&'a str),

    /// No Operation
    Nop,
}

impl<'a> Command<'a> {
    pub(crate) fn parse_line(line: &str) -> IResult<&str, Command> {
        // Most common first.
        alt((
            parse_g1,
            map(tag("G21 "), |_| Command::G21),
            map(tag("G90 "), |_| Command::G90),
            map(tag("G91 "), |_| Command::G91),
            parse_g92,
            // TODO Missing "bezier"
            //
            // Dropping "bed leveling", "dock sled", "Retract", "Stepper motor", "Mechanicla Gantry Calibration"
            map(g_drop, Command::GDrop),
            map(m_drop, Command::MDrop),
            map(tag(";"), |_| Command::Nop),
            map(tag(""), |_| Command::Nop),
        ))(line)
    }
}

// G commands that require no further action
fn g_drop(i: &str) -> IResult<&str, &str> {
    preceded(tag("G "), digit1)(i)
}

fn parse_g1(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G1 "),
        map(pos_many, |val: Vec<PosVal>| {
            let mut hs: HashSet<PosVal> = HashSet::new();
            for item in val {
                hs.insert(item);
            }

            Command::G1(hs)
        }),
    )(i)
}

fn parse_g92(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G92 "),
        map(pos_many, |val: Vec<PosVal>| {
            let mut hs: HashSet<PosVal> = HashSet::new();
            for item in val {
                hs.insert(item);
            }

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

fn m_drop(i: &str) -> IResult<&str, &str> {
    preceded(tag("M"), digit1)(i)
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
