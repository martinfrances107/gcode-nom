use core::fmt::Display;

use crate::command::Command;
use crate::params::PosVal;
use crate::{compute_arc, compute_step_params, ArcParams, PositionMode, StepParams};

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

        let mut is_extruding = false;
        // Positioning mode for all axes (A, B, C), (U, V, W),  (X, Y, Z).
        let mut position_mode = PositionMode::default();
        let mut z = 0_f64;
        // X and Y position of tool head (before projection).
        let mut current_x = 0_f64;
        let mut current_y = 0_f64;
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command is not parsable");
            // Candidate value of params X<number> Y<number>
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

                    // Regarding X and Y at least one must be specified.
                    //
                    // If any value is unspecified the current value is used.
                    let xy_valid = match (x.is_nan(), y.is_nan()) {
                        (false, false) => {
                            // X and Y passed as parameters.
                            true
                        }

                        (false, true) => {
                            // X is passed as a parameter
                            // Y is unspecified
                            y = match position_mode {
                                PositionMode::Absolute => current_y,
                                PositionMode::Relative => 0_f64,
                            };
                            true
                        }
                        (true, false) => {
                            // X is unspecified
                            // Y is passed as a parameter
                            x = match position_mode {
                                PositionMode::Absolute => current_x,
                                PositionMode::Relative => 0_f64,
                            };
                            true
                        }

                        (true, true) => {
                            // Cannot proceed: both X and Y are unspecified
                            // Silently handle error by dropping further action.
                            // TODO: Leave a log in the debug output
                            // once debug strategy is worked developed.
                            false
                        }
                    };

                    if xy_valid {
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
                        mut theta_start,
                        theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                    // All values here are in the range 0<=theta<2PI
                    // We are rotating clockwise
                    // in this cased the start angle of 0 should be read as 2PI
                    if theta_start == 0_f64 {
                        theta_start = 2_f64 * std::f64::consts::PI;
                    }

                    let StepParams {
                        n_steps,
                        theta_step,
                    } = compute_step_params(theta_start, theta_end, radius);

                    // For loop: f64 has a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=n_steps as u64 {
                        let theta = theta_start + (i as f64 * theta_step);
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
                        mut theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                    // All values here are in the range 0<=theta<2PI
                    // We are rotating anticlockwise
                    // in this cased the final angle of 0 should be read as 2PI
                    if theta_end == 0_f64 {
                        theta_end = 2_f64 * std::f64::consts::PI;
                    }

                    let StepParams {
                        n_steps,
                        theta_step,
                    } = compute_step_params(theta_start, theta_end, radius);

                    // For loop with f64 have a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=n_steps as u64 {
                        let theta = theta_start + (i as f64 * theta_step);
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
}
