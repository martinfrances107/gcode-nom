use core::fmt::Display;

use crate::command::Command;
use crate::params::PosVal;
use crate::{compute_arc, ArcParams, PositionMode};

/// SVG representation of a G-Code file.
///
/// wraps the min and max x, y values of the SVG.
#[derive(Debug, Clone, PartialEq)]
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
        let width = self.max_x - self.min_x;
        let height = self.max_y - self.min_y;
        let vb = format!("{} {} {} {}", self.min_x, self.min_y, width, height);
        write!(
            f,
            "<svg height=\"{height}\" width=\"{width}\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{vb}\"> <path d=\""
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

/// Returns a SVG given a collection of G-Code commands.
///
/// TODO: Want to iterate over something more flexible
/// ie. Drop String for something more generic `AsRef<&str>`?
impl FromIterator<String> for Svg {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut svg = Self::default();

        // Invalid if the <path>'s d string does not start with a move.
        svg.parts.push("M0 0".to_string());

        let mut is_extruding = false;
        // Positioning mode for all axes (A, B, C), (U, V, W),  (X, Y, Z).
        let mut position_mode = PositionMode::default();
        let mut z = 0_f64;
        let mut current_x = 0_f64;
        let mut current_y = 0_f64;
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command is not parsable");
            let mut x = f64::NAN;
            let mut y = f64::NAN;

            match command {
                // Treat G0 and G1 command identically.
                //
                // A G0 is a non-printing move but E is present in files seen in the wild.
                // (In the assets directory see the gears and benchy2 files.)
                Command::G0(mut payload) | Command::G1(mut payload) => {
                    // Treat G0 and G1 command identically.
                    //
                    // A G0 is a non-printing move but E is present in files seen in the wild.
                    // (In the assets directory see the gears and benchy2 files.)
                    for param in payload.drain() {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(val) => {
                                is_extruding = val > 0_f64;
                            }
                            _ => {}
                        }
                    }

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
                                    svg.parts.push(format!("L{proj_x:.3} {proj_y:.3}"));
                                } else {
                                    svg.parts.push(format!("M{proj_x:.3} {proj_y:.3}"));
                                }
                                current_x = x;
                                current_y = y;
                            }
                            PositionMode::Relative => {
                                if is_extruding {
                                    svg.parts.push(format!("l{proj_x:.3} {proj_y:.3}"));
                                } else {
                                    svg.parts.push(format!("m{proj_x:.3} {proj_y:.3}"));
                                }
                                current_x += x;
                                current_y += y;
                            }
                        }
                    }
                }
                Command::G2(arc_form) => {
                    // Clockwise arc
                    let ArcParams {
                        origin,
                        radius,
                        theta_start,
                        theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // TODO: FIXME
                    // initially move in 4 steps
                    // This is be replaced by a more sophisticated approach
                    // use the config constant MM_PER_ARC_SEGMENT
                    // to determine the number of steps.
                    let delta_theta = (theta_end - theta_start) / 4.0;

                    // For loop with f64 have a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=4 {
                        // Subtraction here is the only point in this function
                        // that implies clockwise rotation.
                        let theta = theta_start - (i as f64 * delta_theta);
                        let x = origin.0 + radius * theta.cos();
                        let y = origin.1 + radius * theta.sin();

                        let proj_x = y / 2. + x / 2.;
                        let proj_y = -z - y / 2. + x / 2.;
                        svg.update_view_box(proj_x, proj_y);
                        match position_mode {
                            PositionMode::Absolute => {
                                svg.parts.push(format!("L{proj_x:.3} {proj_y:.3}"));
                            }
                            PositionMode::Relative => {
                                svg.parts.push(format!("l{proj_x:.3} {proj_y:.3}"));
                            }
                        }
                    }
                }
                Command::G3(arc_form) => {
                    // Anti-Clockwise arc
                    let ArcParams {
                        origin,
                        radius,
                        theta_start,
                        theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // TODO: FIXME
                    // initially move in 4 steps
                    // This is be replaced by a more sophisticated approach
                    // use the config constant MM_PER_ARC_SEGMENT
                    // to determine the number of steps.
                    let delta_theta = (theta_end - theta_start) / 4.0;

                    // For loop with f64 have a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=4 {
                        // Addition here is the only point in this function
                        // that implies anticlockwise rotation.
                        let theta = theta_start + (i as f64 * delta_theta);
                        let x = origin.0 + radius * theta.cos();
                        let y = origin.1 + radius * theta.sin();

                        let proj_x = y / 2. + x / 2.;
                        let proj_y = -z - y / 2. + x / 2.;
                        svg.update_view_box(proj_x, proj_y);
                        match position_mode {
                            PositionMode::Absolute => {
                                svg.parts.push(format!("L{proj_x:.3} {proj_y:.3}"));
                            }
                            PositionMode::Relative => {
                                svg.parts.push(format!("l{proj_x:.3} {proj_y:.3}"));
                            }
                        }
                    }
                }
                Command::G21 => svg.parts.push("M0 0".to_string()),
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
                            _ => { /* Silently drop. */ }
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
                Command::Comment(_) | Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
            }
        }

        svg
    }
}

// This illustrates a counter clockwise arc, starting at [9, 6]. It can be generated by G3 X2 Y7 I-4 J-3
//
// As show in this (image)[<../images/G3fog.png>]
//
// source <https://marlinfw.org/docs/gcode/G002-G003.html>
#[test]
fn svg_anti_clockwise_arc() {
    // SNAPSHOT values
    // projection was removed the values compared to the diagram, projection was restored and the
    // values copied over.
    let expected_parts: Vec<String> = vec![
        "M0 0".into(),
        "M7.500 1.500".into(),
        "L7.500 1.500".into(),
        "L7.425 0.123".into(),
        "L6.828 -1.121".into(),
        "L5.801 -2.042".into(),
        "L4.500 -2.500".into(),
    ];

    let actual = Svg::from_iter(vec![
        String::from("G0 X9 Y6; set start point"),
        String::from("G3 X2 Y7 I-4 J-3"),
    ]);
    assert_eq!(actual.parts, expected_parts);
}
