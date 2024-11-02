use core::fmt::Display;

use crate::{command::Command, pos::PosVal};

#[derive(Debug, Default, Clone)]
pub struct Svg {
    parts: Vec<String>,
}

// A line could not be decoded as an G-Code command
// #[derive(Debug, Clone)]
// struct GCodeError;

// impl std::fmt::Display for GCodeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "invalid g-code statement")
//     }
// }

impl Display for Svg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<svg height="210" width="400" xmlns="http://www.w3.org/2000/svg">
<path d=""#
        )?;
        for part in &self.parts {
            write!(f, "{part}")?;
        }
        write!(
            f,
            r#""
style="fill:none;stroke:green;stroke-width:3" />
 </svg>"#
        )?;
        Ok(())
    }
}

/// Absolute or Relative positioning
#[derive(Default)]
enum CoordPos {
    /// As per spec `CoordPos::Absolute` is the dafault
    /// <https://marlinfw.org/docs/gcode/G090.html>
    #[default]
    Absolute,
    Relative,
}

/// TODO: Want to iterate over something more flexible
/// ie. Drop String for something more generic `AsRef<&str>`?
impl FromIterator<String> for Svg {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut svg = Self::default();

        let mut abs_coords = CoordPos::default();
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command not parseable");
            // Error if x,y are missing .. Z default to 0!!
            let mut x = f64::NAN;
            let mut y = f64::NAN;
            let mut z = 0.;
            match command {
                // A non printable move.
                Command::G0(mut payload) => {
                    let params = payload.drain();
                    for param in params {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(_) | PosVal::F(_) => {
                                // silently drop
                            }
                            pos_bad => {
                                eprintln!("Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }
                    // Convert x,y,z, into projected x,y.
                    // TODO: Must do something better.
                    let proj_x = x + z / 2.;
                    let proj_y = y + z / 2.;
                    match abs_coords {
                        CoordPos::Absolute => {
                            svg.parts.push(format!("M{proj_x} {proj_y}"));
                        }
                        CoordPos::Relative => {
                            svg.parts.push(format!("m{proj_x} {proj_y}"));
                        }
                    }
                }
                // A printable move.
                Command::G1(mut payload) => {
                    let params = payload.drain();
                    for param in params {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(_) | PosVal::F(_) => {
                                // silently drop
                            }
                            pos_bad => {
                                eprintln!("Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }
                    // Convert x,y,z, into projected x,y.
                    // TODO: Must do something better.

                    // Valid `Command::G1` -  Where X and Y and undefined
                    //
                    // "G1 E2.72551 F1800.00000"
                    if !x.is_nan() && !y.is_nan() {
                        let proj_x = x + z / 2.;
                        let proj_y = y + z / 2.;
                        match abs_coords {
                            CoordPos::Absolute => {
                                svg.parts.push(format!("L{proj_x} {proj_y}"));
                            }
                            CoordPos::Relative => {
                                svg.parts.push(format!("l{proj_x} {proj_y}"));
                            }
                        }
                    }
                }
                Command::G21 => svg.parts.push("M0,0".to_string()),
                Command::G90 => abs_coords = CoordPos::Absolute,
                Command::G91 => abs_coords = CoordPos::Relative,
                Command::G92(_) | Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
            }
        }

        svg
    }
}
