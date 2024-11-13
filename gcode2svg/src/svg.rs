use core::fmt::Display;

use gcode_nom::command::Command;
use gcode_nom::parms::PosVal;
use gcode_nom::PositionMode;

#[derive(Debug, Clone)]
pub struct Svg {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    parts: Vec<String>,
}

impl Default for Svg {
    fn default() -> Self {
        Self {
            min_x: f64::INFINITY,
            max_x: -f64::INFINITY,
            min_y: f64::INFINITY,
            max_y: -f64::INFINITY,
            parts: Vec::default(),
        }
    }
}

impl Svg {
    fn update_view_box(&mut self, proj_x: f64, proj_y: f64) {
        // Record min max x, y
        if proj_x > self.max_x {
            self.max_x = proj_x;
        }
        if proj_x < self.min_x {
            self.min_x = proj_x;
        }
        if proj_y > self.max_y {
            self.max_y = proj_y;
        }
        if proj_y < self.min_y {
            self.min_y = proj_y;
        }
    }
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
        // viewBox="0 0 10 10"
        let width = self.max_x - self.min_x;
        let height = self.max_y - self.min_y;
        let vb = format!("{} {} {} {}", self.min_x, self.min_y, width, height);
        write!(
            f,
            "<svg height=\"{height}\" width=\"{width}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox =\"{vb}\"> <path d=\""
        )?;
        for part in &self.parts {
            write!(f, "{part}")?;
        }
        write!(
            f,
            r#""
style="fill:none;stroke:green;stroke-width:0.05" />
 </svg>"#
        )?;
        Ok(())
    }
}

/// TODO: Want to iterate over something more flexible
/// ie. Drop String for something more generic `AsRef<&str>`?
impl FromIterator<String> for Svg {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut svg = Self::default();

        let mut is_extruding = true;
        // Positioning mode for all axes (A, B, C), (U, V, W),  (X, Y, Z).
        let mut position_mode = PositionMode::default();
        // Extruder is unique - can be overridden.
        let mut e_position_mode = PositionMode::default();
        let mut z = 0_f64;
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command not parseable");
            let mut x = f64::NAN;
            let mut y = f64::NAN;

            match command {
                // A non printable move.
                Command::G0(mut payload) => {
                    let params = payload.drain();
                    for param in params {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(val) => {
                                // Negative values the extruder is "wiping"
                                // or sucking filament back into the extruder.
                                is_extruding = val > 0_f64;
                            }
                            PosVal::F(_) => {
                                // Silently drop.
                            }
                            pos_bad => {
                                eprintln!("Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }

                    // Valid `Command::G0` -  Where X and Y and undefined
                    //
                    // "G1 E2.72551 F1800.00000"
                    if !x.is_nan() && !y.is_nan() {
                        // Convert x,y,z, into projected x,y.
                        let proj_x = y / 2. + x / 2.;
                        let proj_y = -z - y / 2. + x / 2.;
                        svg.update_view_box(proj_x, proj_y);
                        match position_mode {
                            PositionMode::Absolute => {
                                svg.parts.push(format!("M{proj_x} {proj_y}"));
                            }
                            PositionMode::Relative => {
                                svg.parts.push(format!("m{proj_x} {proj_y}"));
                            }
                        }
                    }
                }
                // A printable move.
                Command::G1(mut params) => {
                    for param in params.drain() {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(val) => {
                                is_extruding = val > 0_f64;
                            }
                            PosVal::F(_) => {
                                // silently drop
                            }
                            pos_bad => {
                                eprintln!("SVG Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }
                    // Convert x,y,z, into projected x,y.
                    // TODO: Must do something better.

                    // Valid `Command::G1` -  Where X and Y and undefined
                    //
                    // "G1 E2.72551 F1800.00000"
                    if !x.is_nan() && !y.is_nan() {
                        let proj_x = y / 2. + x / 2.;
                        let proj_y = -z - y / 2. + x / 2.;
                        svg.update_view_box(proj_x, proj_y);
                        match position_mode {
                            PositionMode::Absolute => {
                                if is_extruding {
                                    svg.parts.push(format!("L{proj_x} {proj_y}"));
                                } else {
                                    svg.parts.push(format!("M{proj_x} {proj_y}"));
                                }
                            }
                            PositionMode::Relative => {
                                if is_extruding {
                                    svg.parts.push(format!("l{proj_x} {proj_y}"));
                                } else {
                                    svg.parts.push(format!("m{proj_x} {proj_y}"));
                                }
                            }
                        }
                    }
                }
                Command::G21 => svg.parts.push("M0,0".to_string()),
                Command::G90 => position_mode = PositionMode::Absolute,
                Command::G91 => position_mode = PositionMode::Relative,
                // G92- Set Current Position
                Command::G92(mut params) => {
                    // The extrude rate is going to zero
                    // enter MoveMode ..ie not laying down filament.
                    for param in params.drain() {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(val) => {
                                // Negative values the extruder is "wiping"
                                // or sucking filament back into the extruder.
                                is_extruding = val > 0_f64;
                            }
                            PosVal::F(_) => {
                                // silently drop
                            }
                            pos_bad => {
                                eprintln!("Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }

                    // Set Position is by definition a move only.
                    if !x.is_nan() && !y.is_nan() {
                        let proj_x = y / 2. + x / 2.;
                        let proj_y = -z - y / 2. + x / 2.;
                        svg.update_view_box(proj_x, proj_y);
                        match position_mode {
                            PositionMode::Absolute => {
                                svg.parts.push(format!("M{proj_x} {proj_y}"));
                            }
                            PositionMode::Relative => {
                                svg.parts.push(format!("m{proj_x} {proj_y}"));
                            }
                        }
                    }
                }
                Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
            }
        }

        svg
    }
}
