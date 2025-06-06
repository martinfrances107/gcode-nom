use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::character::complete::line_ending;
use nom::character::complete::not_line_ending;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::multi::many;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::IResult;
use nom::Parser;

use crate::arc::parse_arc_a;
use crate::arc::parse_arc_b;
use crate::arc::parse_arc_c;
use crate::arc::parse_arc_e;
use crate::arc::parse_arc_f;
use crate::arc::parse_arc_i;
use crate::arc::parse_arc_j;
use crate::arc::parse_arc_s;
use crate::arc::parse_arc_u;
use crate::arc::parse_arc_v;
use crate::arc::parse_arc_w;
use crate::arc::parse_arc_x;
use crate::arc::parse_arc_y;
use crate::arc::parse_arc_z;
use crate::arc::ArcVal;
use crate::arc::Form as ArcForm;

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
    /// G2 – Clockwise Arc
    G2(ArcForm),
    /// G3 – Counter-clockwise Arc
    G3(ArcForm),

    // G5 - Bézier Cubic Spline
    // TODO must implement
    /// Home all axes
    G21,
    ///G90 – Set Positioning Mode Absolute
    ///
    /// "G90 ; Set all axes to absolute"
    G90,
    /// G91 – Set Positioning Mode Relative
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
    /// ; This is a comment
    Comment(String),
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
            parse_g0,
            parse_g2,
            parse_g3,
            // TODO add G5 - Bézier Cubic Spline
            map(tag("G21"), |_| Self::G21),
            map(tag("G90"), |_| Self::G90),
            map(tag("G91"), |_| Self::G91),
            parse_g92,
            parse_comment,
            // Dropping "bed leveling", "dock sled", "Retract", "Stepper motor", "Mechanical Gantry Calibration"
            map(g_drop, Self::GDrop),
            map(m_drop, Self::MDrop),
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
        (tag("G0"), space0),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G0(hs)
        }),
    )
    .parse(i)
}

// Collect everything after the semicolon until the end of the line.
// as a comment string
fn parse_comment(i: &str) -> IResult<&str, Command> {
    preceded(
        (space0, tag(";")),
        terminated(
            map(not_line_ending, |v: &str| Command::Comment(v.to_string())),
            line_ending,
        ),
    )
    .parse(i)
}

/// Linear move
///
/// May or may not include whitespace separators.
///
/// G1X94.838Y81.705F9000
/// G1 X94.838Y81.705 F9000 ; comment text
///
/// NB - The command is dropped and cannot be recovered.
///
/// # Errors
///   When match fails.
fn parse_g1(i: &str) -> IResult<&str, Command> {
    preceded(
        (tag("G1"), space0),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            Command::G1(hs)
        }),
    )
    .parse(i)
}

/// G2 Clockwise arc
///
/// May or may not include whitespace separators.
///
/// G2X94.838Y81.705F9000
/// G2 X94.838Y81.705 F9000 ; comment text
///
/// NB - The command is dropped and cannot be recovered.
///
/// # Errors
///   When match fails.
fn parse_g2(i: &str) -> IResult<&str, Command> {
    preceded(
        (tag("G2"), space0),
        map_res(arc_many, |vals: Vec<ArcVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            let mut has_ij = false;
            let mut has_r = false;
            for val in &hs {
                match val {
                    ArcVal::I(_) | ArcVal::J(_) => {
                        // If I or J is present, then we have a "IJ" form.
                        has_ij = true;
                    }
                    ArcVal::R(_) => {
                        // If R is present, then we have a "R" form.
                        has_r = true;
                    }
                    _ => {}
                }
            }
            // Checks (I,J) and R are mutually exclusive.
            // If both are present then the command is invalid.
            // If neither is present then the command is invalid.
            match (has_ij, has_r) {
                (true, false) => Ok(Command::G2(ArcForm::IJ(hs))),
                (false, true) => Ok(Command::G2(ArcForm::R(hs))),
                _ => {
                    // Invalid G2 command: must have either I,J or R but not both,
                    Err("Invalid G2 command: must have either I,J or R but not both")
                }
            }
        }),
    )
    .parse(i)
}

/// G2 Clockwise arc
///
/// May or may not include whitespace separators.
///
/// G2X94.838Y81.705F9000
/// G2 X94.838Y81.705 F9000 ; comment text
///
/// NB - The command is dropped and cannot be recovered.
///
/// # Errors
///   When match fails.
fn parse_g3(i: &str) -> IResult<&str, Command> {
    preceded(
        (tag("G3"), space0),
        map_res(arc_many, |vals: Vec<ArcVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f64>.
            let hs = HashSet::from_iter(vals);
            let mut has_ij = false;
            let mut has_r = false;
            for val in &hs {
                match val {
                    ArcVal::I(_) | ArcVal::J(_) => {
                        // If I or J is present, then we have a "IJ" form.
                        has_ij = true;
                    }
                    ArcVal::R(_) => {
                        // If R is present, then we have a "R" form.
                        has_r = true;
                    }
                    _ => {}
                }
            }
            // Checks (I,J) and R are mutually exclusive.
            // If both are present then the command is invalid.
            // If neither is present then the command is invalid.
            match (has_ij, has_r) {
                (true, false) => Ok(Command::G3(ArcForm::IJ(hs))),
                (false, true) => Ok(Command::G3(ArcForm::R(hs))),
                _ => {
                    // Invalid G2 command: must have either I,J or R but not both,
                    Err("Invalid G2 command: must have either I,J or R but not both")
                }
            }
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
        (tag("G92"), space0),
        map(pos_many, |vals: Vec<PosVal>| {
            // Paranoid: deduplication.
            // eg. There can be only one E<f63> value.
            let hs = HashSet::from_iter(vals);
            Command::G92(hs)
        }),
    )
    .parse(i)
}

/// Extracts from 1 to 12 values from the set of `PosVal`s.
///
/// ( A, B, C, E, F, S, U, V, W, X, Y, Z )
///
/// # Errors
///   When match fails.
fn pos_many(i: &str) -> IResult<&str, Vec<PosVal>> {
    many(1..12, pos_val).parse(i)
}

/// Extracts from 1 to 12 values from the set of `PosVal`s.
///
/// ( A, B, C, E, F, S, U, V, W, X, Y, Z )
///
/// # Errors
///   When match fails.
fn arc_many(i: &str) -> IResult<&str, Vec<ArcVal>> {
    many(1..16, arc_val).parse(i)
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

///
/// # Errors
///   When match fails.
fn arc_val(i: &str) -> IResult<&str, ArcVal> {
    alt((
        parse_arc_a,
        parse_arc_b,
        parse_arc_c,
        parse_arc_e,
        parse_arc_f,
        parse_arc_i,
        parse_arc_j,
        parse_arc_s,
        parse_arc_u,
        parse_arc_v,
        parse_arc_w,
        parse_arc_x,
        parse_arc_y,
        parse_arc_z,
    ))
    .parse(i)
}

/// Drop M code - no further action
///
/// # Errors
///   When match fails.
pub fn m_drop(i: &str) -> IResult<&str, u16> {
    map_res(preceded((tag("M"), space0), digit1), str::parse).parse(i)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn comments() {
        let text_commands = [
            (
                "; perimeters extrusion width = 0.67mm\n",
                Ok((
                    "",
                    Command::Comment(String::from(" perimeters extrusion width = 0.67mm")),
                )),
            ),
            (
                // a sample of a comment with a base-64 encoded thumbnail.
                "; 7K6Ho8Q5vPBT4ZkdDGAk/t/wOw4rChXwlVJwAAAABJRU5ErkJggg==\n",
                Ok((
                    "",
                    Command::Comment(String::from(
                        " 7K6Ho8Q5vPBT4ZkdDGAk/t/wOw4rChXwlVJwAAAABJRU5ErkJggg==",
                    )),
                )),
            ),
            (
                // Header:- "assets/3dBenchy.gcode"
                // Paranoid: This test asserts "No greedy grabbing over a blank line!"
                // Input string should be interpreted a (Command::comment, Command::blank, CommandComment)
                "; generated by Slic3r 1.2.9 on 2015-10-01 at 20:51:53

; external perimeters extrusion width = 0.40mm",
                Ok((
                    "
; external perimeters extrusion width = 0.40mm",
                    Command::Comment(" generated by Slic3r 1.2.9 on 2015-10-01 at 20:51:53".into()),
                )),
            ),
        ];

        for (line, expected) in text_commands {
            let actual = Command::parse_line(line);
            assert_eq!(actual, expected, "line: {}", line);
        }
    }

    #[test]
    fn g0() {
        let text_commands = [(
            // Troublesome pattern found in "both \parts.gcode".
            "G0E-2.7F4200",
            Ok((
                "",
                Command::G0([PosVal::E(-2.7), PosVal::F(4200_f64)].into()),
            )),
        )];

        for (line, expected) in text_commands {
            let actual = Command::parse_line(line);
            assert_eq!(actual, expected, "line: {}", line);
        }
    }

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
            (
                // Must tolerate compact form without whitespace.
                "G1Z0.350F7800.000",
                Ok((
                    "",
                    Command::G1([PosVal::Z(0.350_f64), PosVal::F(7800_f64)].into()),
                )),
            ),
            (
                // Paranoid: - Initial tags has whitespace, but parameters are expressed in a compact form.
                "G1 Z0.350F7800.000",
                Ok((
                    "",
                    Command::G1([PosVal::Z(0.350_f64), PosVal::F(7800_f64)].into()),
                )),
            ),
            (
                // Paranoid: - Initial tags has whitespace, but parameters are expressed in a compact form.
                "G1X888F1000",
                Ok((
                    "",
                    Command::G1([PosVal::X(888_f64), PosVal::F(1000_f64)].into()),
                )),
            ),
            // Fails : -
            (
                // Invalid  missing parameters
                // G1 Fails falls back to a generic GDrop(1)
                "G1 ",
                Ok((" ", Command::GDrop(1))),
            ),
            // (
            //     // Supplied 13 parameters when only 12 are permitted.
            //     "G1A0A1B2C3E4F5S6U7V8W9X0Y1Z1Z2",
            //     Ok((
            //         "",
            //         Command::G1(
            //             [
            //                 PosVal::A(0_f64),
            //                 PosVal::A(1_f64),
            //                 PosVal::B(2_f64),
            //                 PosVal::C(3_f64),
            //                 PosVal::E(4_f64),
            //                 PosVal::F(5_f64),
            //                 PosVal::S(6_f64),
            //                 PosVal::U(7_f64),
            //                 PosVal::V(8_f64),
            //                 PosVal::W(9_f64),
            //                 PosVal::X(0_f64),
            //                 PosVal::Y(1_f64),
            //                 PosVal::Z(2_f64),
            //             ]
            //             .into(),
            //         ),
            //     )),
            // ),
        ];

        for (line, expected) in text_commands {
            let actual = Command::parse_line(line);
            assert_eq!(actual, expected, "line: {}", line);
        }
    }

    // G2 Clockwise arc
    //
    // ARC command come in two forms:
    //
    // "IJ" Form
    // "R" Form
    //
    // TODO add this test
    //
    // IJ Form
    // At least one of the I J parameters is required.
    // X and Y can be omitted to do a complete circle.
    // Mixing I or J with R will throw an error.
    //
    // R Form
    // R specifies the radius. X or Y is required.
    // Omitting both X and Y will throw an error.
    // Mixing R with I or J will throw an error.
    //
    // source https://marlinfw.org/docs/gcode/G002-G003.html
    #[test]
    fn g2() {
        // let default = PosPayload::<f64>::default();

        let text_commands = [
            (
                "G2 X125 Y32 I10.5 J10.5; arc",
                Ok((
                    "; arc",
                    Command::G2(ArcForm::IJ(
                        [
                            ArcVal::X(125_f64),
                            ArcVal::Y(32_f64),
                            ArcVal::I(10.5),
                            ArcVal::J(10.5),
                        ]
                        .into(),
                    )),
                )),
            ),
            (
                "G2 I20 J20; X and Y can be omitted to do a complete circle.",
                Ok((
                    "; X and Y can be omitted to do a complete circle.",
                    Command::G2(ArcForm::IJ([ArcVal::I(20_f64), ArcVal::J(20_f64)].into())),
                )),
            ),
        ];

        for (line, expected) in text_commands {
            let actual = Command::parse_line(line);
            assert_eq!(actual, expected, "line: {}", line);
        }
    }
    // // G3 X2 Y7 R5
    #[test]
    fn g3() {
        let text_commands = [
            (
                "G2 X125 Y32 I10.5 J10.5; arc",
                Ok((
                    "; arc",
                    Command::G2(ArcForm::IJ(
                        [
                            ArcVal::X(125_f64),
                            ArcVal::Y(32_f64),
                            ArcVal::I(10.5),
                            ArcVal::J(10.5),
                        ]
                        .into(),
                    )),
                )),
            ),
            (
                "G2 I20 J20; X and Y can be omitted to do a complete circle.",
                Ok((
                    "; X and Y can be omitted to do a complete circle.",
                    Command::G2(ArcForm::IJ([ArcVal::I(20_f64), ArcVal::J(20_f64)].into())),
                )),
            ),
        ];
        for (line, expected) in text_commands {
            let actual = Command::parse_line(line);
            assert_eq!(actual, expected, "line: {}", line);
        }
    }

    #[test]
    const fn parse_g_drop() {}
}
