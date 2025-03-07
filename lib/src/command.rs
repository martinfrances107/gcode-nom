use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::multi::separated_list1;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

use crate::params::parse_a;
use crate::params::parse_b;
use crate::params::parse_c;
use crate::params::parse_e;
use crate::params::parse_f;
use crate::params::parse_s;
use crate::params::parse_u;
use crate::params::parse_v;
use crate::params::parse_w;
use crate::params::parse_x;
use crate::params::parse_y;
use crate::params::parse_z;
use crate::params::PosVal;

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
    ///G90 – Set Positioning Mode
    ///
    /// "G90 ; Set all axes to absolute"
    G90,
    /// G91 – Set Positioning Mode
    ///
    /// "G91 ; Set all axes to relative"
    G91,
    /// Set the current position
    /// eg. "G92 E0"
    /// TODO:  F and S are not permitted here.
    G92(HashSet<PosVal>),
    /// Drop G - no further action.
    GDrop(u16),
    /// Drop M - no further action.
    MDrop(u16),
    /// No Operation eg a blank line "".
    Nop,
}

impl Command {
    /// Decodes a `GCode` command.
    ///
    /// # Errors
    ///   When match fails.
    pub fn parse_line(line: &str) -> IResult<&str, Self> {
        // Most common first.
        alt((
            parse_g1,
            map(tag("G21"), |_| Self::G21),
            map(tag("G90"), |_| Self::G90),
            map(tag("G91"), |_| Self::G91),
            parse_g92,
            // Command::G0 - Non printing moves are infrequent.
            //  eg "The benchy" example has none.
            parse_g0,
            // Dropping "bed leveling", "dock sled", "Retract", "Stepper motor", "Mechanical Gantry Calibration"
            map(g_drop, Self::GDrop),
            map(m_drop, Self::MDrop),
            map(tag(";"), |_| Self::Nop),
            map(tag(""), |_| Self::Nop),
        ))
        .parse(line)
    }
}

/// G commands that require no further action
///
/// # Errors
///   When match fails.
pub fn g_drop(i: &str) -> IResult<&str, u16> {
    map_res(preceded(tag("G"), digit1), str::parse).parse(i)
}

/// # Errors
///   When match fails.
fn parse_g0(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G0 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G0(hs)
        }),
    )
    .parse(i)
}

/// Linear move
///
/// # Errors
///   When match fails.
fn parse_g1(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G1 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G1(hs)
        }),
    )
    .parse(i)
}

/// G92 Set current position.
///
/// # Errors
///   When match fails.
fn parse_g92(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G92 "),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f63> value.
            let hs = HashSet::from_iter(vals);
            Command::G92(hs)
        }),
    )
    .parse(i)
}

///
/// # Errors
///   When match fails.
fn pos_many(i: &str) -> IResult<&str, Vec<PosVal>> {
    separated_list1(tag(" "), pos_val).parse(i)
}

///
/// # Errors
///   When match fails.
fn pos_val(i: &str) -> IResult<&str, PosVal> {
    alt((
        parse_a, parse_b, parse_c, parse_e, parse_f, parse_s, parse_u, parse_v, parse_w, parse_x,
        parse_y, parse_z,
    ))
    .parse(i)
}

/// Drop M code - no further action
///
/// # Errors
///   When match fails.
pub fn m_drop(i: &str) -> IResult<&str, u16> {
    map_res(preceded(tag("M"), digit1), str::parse).parse(i)
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
            ("G1 Z5", Ok(("", Command::G1([PosVal::Z(5_f64)].into())))),
            (
                "G1 Z5 F5000 ; lift nozzle",
                Ok((
                    " ; lift nozzle",
                    Command::G1([PosVal::Z(5_f64), PosVal::F(5000_f64)].into()),
                )),
            ),
            (
                "G1 E1.00000 F1800.00000 ; text",
                Ok((
                    " ; text",
                    Command::G1([PosVal::E(1.0_f64), PosVal::F(1800_f64)].into()),
                )),
            ),
            (
                "G1 Z0.350 F7800.000",
                Ok((
                    "",
                    Command::G1([PosVal::Z(0.350_f64), PosVal::F(7800_f64)].into()),
                )),
            ),
        ];

        for (line, expected) in text_commands {
            let actual = parse_g1(line);
            assert_eq!(actual, expected);
        }
    }
    #[test]
    const fn parse_g_drop() {}
}
