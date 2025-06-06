//! gcode-nom
//!
#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]

use core::f64;

use crate::arc::ArcVal;
use crate::arc::Form as ArcForm;

/// Streaming for binary gcode files
pub mod binary;
/// Parsing rules for gcode commands
pub mod command;
mod double;
/// Parsing rules for G0/G1 commands
pub mod params;

/// Parsing rules for G2/G3 arc commands
pub mod arc;

/// Absolute or Relative positioning
#[derive(Default, Debug, Eq, PartialEq)]
pub enum PositionMode {
    /// As per spec `Positionmode::Absolute` is the default
    /// <https://marlinfw.org/docs/gcode/G090.html>
    #[default]
    Absolute,
    /// Relative positioning.
    Relative,
}

/// Returns values used to render an ARC
///
/// input state is current position and the raw param values
/// extracted from command.
///
#[derive(Debug)]
pub struct ArcParams {
    /// The center of the arc
    pub origin: (f64, f64),
    /// The radius of the arc
    pub radius: f64,
    /// The start angle of the arc in radians
    pub theta_start: f64,
    /// The end angle of the arc in radians
    pub theta_end: f64,
}

#[must_use] // // pub fn compute_arc(&payload) ->  ( origin, radius, theta_start, theta_end)
/// Computes the parameters of an arc given the current position and the arc form.
///
/// ArcParams contains the values in a form which can be rendered to a OBJ/SVG file.
pub fn compute_arc(current_x: f64, current_y: f64, form: &ArcForm) -> ArcParams {
    let mut i: f64 = f64::NAN;
    let mut j: f64 = f64::NAN;
    let mut r: f64 = f64::NAN;
    let mut x: f64 = f64::NAN;
    let mut y: f64 = f64::NAN;

    let radius: f64;
    let origin: (f64, f64);
    let theta_start: f64;
    let theta_end: f64;

    match form {
        ArcForm::IJ(arc_values) => {
            // I and J form
            for val in arc_values {
                match val {
                    ArcVal::X(val) => x = *val,
                    ArcVal::Y(val) => y = *val,
                    ArcVal::I(val) => i = *val,
                    ArcVal::J(val) => j = *val,
                    _ => {
                        // Ignored params
                    }
                }
            }

            radius = i.hypot(j);
            origin = (current_x + i, current_y + j);

            let delta_start_x = current_x - origin.0;
            let delta_start_y = current_y - origin.1;
            // if delta_start_x == 0.0 {
            //     panic!("Ambiguity straight up or start down - can't tell")
            // } else {
            //     theta_start = (delta_start_y).atan2(delta_start_x);
            // }
            theta_start = (delta_start_y).atan2(delta_start_x);

            let delta_end_x = x - origin.0;
            let delta_end_y = y - origin.1;
            // if delta_end_x == 0.0 {
            //     panic!("Ambiguity straight up or start down - can't tell")
            // } else {
            //     theta_end = (delta_end_y).atan2(delta_end_x);
            // }
            theta_end = (delta_end_y).atan2(delta_end_x);
        }
        ArcForm::R(arc_values) => {
            // R form
            for val in arc_values {
                match val {
                    ArcVal::X(val) => x = *val,
                    ArcVal::Y(val) => y = *val,
                    ArcVal::R(val) => r = *val,
                    _ => {
                        // Ignored params
                    }
                }
            }
            radius = r;
            // Must solve this  par of simultaneous equations
            // radius * radius = ( origin.0 - current_x ) * ( origin.0 - current_x ) + ( origin.1 - current_y ) * ( origin.1 - current_y )
            // radians * radius = (current_x - origin.0) * (current_x - origin.0) + (current_y - origin.1) * (current_y - origin.1)
            //
            // which of the two solutions is correct can be determined by realizing that we are moving clockwise or counter clockwise
            todo!();
        }
    }
    ArcParams {
        origin,
        radius,
        theta_start,
        theta_end,
    }
}

// This illustrates a counter clockwise arc, starting at [9, 6]. It can be generated either by G3 X2 Y7 I-4 J-3 or G3 X2 Y7 R5
//
// As show in this (image)[<../images/G3fog.png>]
//
// source <https://marlinfw.org/docs/gcode/G002-G003.html>
#[cfg(test)]
mod tests {
    use super::*;

    fn round_to_two_decimals(x: f64) -> f64 {
        (x * 100.0).round() / 100.0
    }

    #[test]
    fn compute_arc_ij() {
        let arc = compute_arc(
            9.0,
            6.0,
            &ArcForm::IJ(
                [
                    ArcVal::X(2.0),
                    ArcVal::Y(7.0),
                    ArcVal::I(-4.0),
                    ArcVal::J(-3.0),
                ]
                .into(),
            ),
        );
        assert_eq!(arc.origin, (5.0, 3.0));
        assert_eq!(arc.radius, 5.0);
        assert_eq!(
            round_to_two_decimals(arc.theta_start.to_degrees()),
            36.87_f64
        );
        assert_eq!(
            round_to_two_decimals(arc.theta_end.to_degrees()),
            126.87_f64
        );
    }

    #[ignore]
    #[test]
    // ignored? - Complex algorithm to be implemented involving solving a par of simultaneous equations
    fn compute_arc_r() {
        let arc = compute_arc(
            9.0,
            6.0,
            &ArcForm::R([ArcVal::X(2.0), ArcVal::Y(7.0), ArcVal::R(5.0)].into()),
        );
        assert_eq!(arc.origin, (5.0, 3.0));
        assert_eq!(arc.radius, 5.0);
    }
}
