use core::f64;
use core::f64::consts::TAU;
use core::fmt::Display;

use crate::command::Command;
use crate::params::head::PosVal;
use crate::{compute_arc, ArcParams, PositionMode, MM_PER_ARC_SEGMENT};

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
        // An empty gcode file will not change min_x/y or max_x/y from
        // its default of +/-INF respectively.
        //
        let parameters = if width.is_finite() && height.is_finite() {
            format!(
                "width=\"{width}\" height=\"{height}\" viewBox=\"{} {} {} {}\"",
                self.min_x, self.min_y, width, height
            )
        } else {
            // In this case silently fail by returning a empty SVG element, without a viewBox parameter.
            String::new()
        };
        writeln!(
            f,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" {parameters} >"
        )?;
        write!(f, "  <path d=\"")?;

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

        let mut part_id = Some(0);
        let mut is_extruding = false;
        // Positioning mode for all axes (A, B, C), (U, V, W),  (X, Y, Z).
        let mut position_mode = PositionMode::default();
        // X and Y position of tool head (before projection).
        let mut current_x = 0_f64;
        let mut current_y = 0_f64;
        let mut current_z = 0_f64;

        let mut origin_x = 0_f64;
        let mut origin_y = 0_f64;
        let mut origin_z = 0_f64;

        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command is not parsable");

            match command {
                // Treat G0 and G1 command identically.
                //
                // A G0 is a non-printing move but E is present in files seen in the wild.
                // (In the assets directory see the gears and benchy2 files.)
                Command::G0(mut payload) | Command::G1(mut payload) => {
                    // Candidate value of params X<number> Y<number>
                    let mut x_param = f64::NAN;
                    let mut y_param = f64::NAN;
                    let mut z_param = f64::NAN;

                    for param in payload.drain() {
                        match param {
                            PosVal::X(val) => x_param = val,
                            PosVal::Y(val) => y_param = val,
                            PosVal::Z(val) => z_param = val,
                            // Negative values the extruder is "wiping"
                            // or sucking filament back into the extruder.
                            PosVal::E(val) => is_extruding = val > 0_f64,
                            _ => {}
                        }
                    }

                    if !x_param.is_nan() {
                        current_x = match position_mode {
                            PositionMode::Absolute => x_param,
                            PositionMode::Relative => current_x + x_param,
                        }
                    };

                    if !y_param.is_nan() {
                        current_y = match position_mode {
                            PositionMode::Absolute => y_param,
                            PositionMode::Relative => current_y + y_param,
                        }
                    }

                    if !z_param.is_nan() {
                        current_z = match position_mode {
                            PositionMode::Absolute => z_param,
                            PositionMode::Relative => current_z + z_param,
                        };
                    }

                    let proj_x = (origin_y + current_y) / 2. + (origin_x + current_x) / 2.;
                    let proj_y = -(origin_z + current_z) - (origin_y + current_y) / 2.
                        + (origin_x + current_x) / 2.;
                    svg.update_view_box(proj_x, proj_y);

                    if is_extruding && part_id.is_some() {
                        svg.parts.push(format!("L{proj_x:.3} {proj_y:.3}"));
                    } else {
                        svg.parts.push(format!("M{proj_x:.3} {proj_y:.3}"));
                    }
                }
                Command::G2(arc_form) => {
                    // Clockwise arc
                    let ArcParams {
                        center,
                        radius,
                        mut theta_start,
                        theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                    // All values here are in the range 0<=theta<2PI
                    // We are rotating clockwise
                    // in this cased the start angle of 0 should be read as 2PI
                    if theta_start == 0_f64 {
                        theta_start = TAU;
                    }

                    let delta_theta = if theta_start < theta_end {
                        // Adjust for zero crossing
                        // say 115 -> 304 degrees
                        // delta_theta = 115 + (360 - 304 ) = 170
                        theta_start + (TAU - theta_end)
                    } else {
                        theta_start - theta_end
                    };
                    let total_arc_length = delta_theta * radius;
                    // n_steps must be a number > 0
                    let n_steps = (total_arc_length / MM_PER_ARC_SEGMENT).ceil();
                    let theta_step = delta_theta / n_steps;

                    // x,y are the position of the head in absolute units.
                    let mut x = f64::NAN;
                    let mut y = f64::NAN;
                    // For loop: f64 has a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=n_steps as u64 {
                        let theta = (theta_start - (i as f64 * theta_step)) % TAU;
                        x = center.0 + radius * theta.cos();
                        y = center.1 + radius * theta.sin();

                        let proj_x = (origin_x + x + origin_y + y) / 2_f64;
                        let proj_y =
                            -(origin_z + current_z) - (origin_y + y) / 2. + (origin_x + x) / 2.;
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

                    current_x = x;
                    current_y = y;
                }
                Command::G3(arc_form) => {
                    // Anti-Clockwise arc
                    let ArcParams {
                        center,
                        radius,
                        theta_start,
                        mut theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                    // All values here are in the range 0<=theta<2PI
                    // We are rotating anticlockwise
                    // in this cased the final angle of 0 should be read as 2PI
                    if theta_end == 0_f64 {
                        theta_end = TAU;
                    }

                    let delta_theta = if theta_start > theta_end {
                        // Adjust for zero crossing
                        // say 306 -> 115 degrees
                        // delta_theta = (360 - 305 ) + 115 = 170
                        TAU - theta_start + theta_end
                    } else {
                        theta_end - theta_start
                    };
                    let total_arc_length = delta_theta * radius;
                    // n_steps must be a number > 0
                    let n_steps = (total_arc_length / MM_PER_ARC_SEGMENT).ceil();
                    let theta_step = delta_theta / n_steps;

                    // For loop with f64 have a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    let mut x = f64::NAN;
                    let mut y = f64::NAN;
                    for i in 0..=n_steps as u64 {
                        let theta = (theta_start + (i as f64 * theta_step)) % TAU;
                        x = center.0 + radius * theta.cos();
                        y = center.1 + radius * theta.sin();

                        let proj_x = (origin_x + x + origin_y + y) / 2.;
                        let proj_y =
                            -(origin_z + current_z) - (origin_y + y) / 2. + (origin_x + x) / 2.;
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

                    current_x = x;
                    current_y = y;
                }
                Command::G90 => position_mode = PositionMode::Absolute,
                Command::G91 => position_mode = PositionMode::Relative,

                // If the current position is at X=4 and G92 X7 is programmed,
                //  the current position is redefined as X=7, effectively
                // moving the origin of the coordinate system -3 units in X.""
                Command::G92(mut params) => {
                    // The extrude rate is going to zero
                    // enter MoveMode ..ie not laying down filament.
                    for param in params.drain() {
                        match param {
                            PosVal::E(val) => {
                                // Negative values the extruder is "wiping"
                                // or sucking filament back into the extruder.
                                is_extruding = val > 0_f64;
                            }
                            PosVal::X(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_x = current_x - val;
                                    current_x = val;
                                }
                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            PosVal::Y(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_y = current_x - val;
                                    current_y = val;
                                }
                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            PosVal::Z(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_z = current_z - val;
                                    current_z = val;
                                }
                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            _ => { /* Silently drop. */ }
                        }
                    }

                    // Set Position is by definition a move only.
                    let proj_x = (origin_x + current_x + origin_y + current_y) / 2.;
                    let proj_y = -(origin_z + current_z) - (origin_y + current_y) / 2.
                        + (origin_x + current_x) / 2.;
                    svg.update_view_box(proj_x, proj_y);

                    svg.parts.push(format!("M{proj_x} {proj_y}"));
                }
                _ => {}
            }
        }

        svg
    }
}

#[cfg(test)]
mod svg {
    use super::*;
    use crate::command::Command;
    use insta::assert_debug_snapshot;

    // The first few lines of assets/3dBench.gcode
    static INPUT: &str = r"
; generated by Slic3r 1.2.9 on 2015-10-01 at 20:51:53

; external perimeters extrusion width = 0.40mm
; perimeters extrusion width = 0.67mm
; infill extrusion width = 0.67mm
; solid infill extrusion width = 0.67mm
; top infill extrusion width = 0.67mm

M107
M190 S65 ; set bed temperature
M104 S205 ; set temperature
G28 ; home all axes
G1 Z5 F5000 ; lift nozzle
M109 S205 ; wait for temperature to be reached
G21 ; set units to millimeters
G90 ; use absolute coordinates
M82 ; use absolute distances for extrusion
G92 E0
G1 E-1.00000 F1800.00000
G92 E0
G1 Z0.350 F7800.000
";

    #[test]
    fn nothing_unhandled() {
        // The first few lines of the benchy file must be recognized.
        for line in INPUT.lines() {
            assert!(Command::parse_line(line).is_ok());
        }
    }

    #[test]
    fn arc_clockwise() {
        // SNAPSHOT tests
        //
        // Simple pattern essential for code coverage
        //
        // Ensures calculated theta values are in the range 0..360
        // as measured in anticlockwise from the positive x-axis.
        //
        // 0 and 360 are the same point
        // This test asserts that the cases where 360 must be used are correct.
        let buffer = include_str!("../../../../assets/g3_box_rounded_anticlockwise.gcode");
        let svg = buffer.lines().map(|l| l.to_string()).collect::<Svg>();
        assert_debug_snapshot!(svg);
    }

    #[test]
    fn arc_anti_clockwise() {
        // SNAPSHOT tests
        //
        // Simple pattern essential for code coverage
        //
        // Ensures calculated theta values are in the range 0..360
        // as measured in anticlockwise from the positive x-axis.
        //
        // 0 and 360 are the same point
        // This test asserts that the cases where 360 must be used are correct.
        let buffer = include_str!("../../../../assets/g2_box_nibble_clockwise.gcode");
        let svg = buffer.lines().map(|l| l.to_string()).collect::<Svg>();
        assert_debug_snapshot!(svg);
    }

    #[test]
    fn arc_demo() {
        // SNAPSHOT tests
        let buffer = include_str!("../../../../assets/arc_demo.gcode");
        let svg = buffer.lines().map(|l| l.to_string()).collect::<Svg>();
        assert_debug_snapshot!(svg);
    }

    #[test]
    fn zero_crossing() {
        // SNAPSHOT tests
        //
        // Complex model with lots of curves.
        //
        // NB This is the only test that covers both clockwise and anticlockwise
        // zero crossings.
        let buffer = include_str!("../../../../assets/both.gcode");
        let svg = buffer.lines().map(|l| l.to_string()).collect::<Svg>();
        assert_debug_snapshot!(svg);
    }
}
