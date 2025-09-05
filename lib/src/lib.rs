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

/// G2/G3 Arc commands.
/// Used in step size calculations
pub static MM_PER_ARC_SEGMENT: f64 = 1_f64;

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
    pub center: (f64, f64),
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
/// `ArcParams` contains the values in a form which can be rendered to a OBJ/SVG file.
pub fn compute_arc(current_x: f64, current_y: f64, form: &ArcForm) -> ArcParams {
    // Unspecified relative offsets default to zero.
    let mut i: f64 = 0_f64;
    let mut j: f64 = 0_f64;

    let mut x: f64 = f64::NAN;
    let mut y: f64 = f64::NAN;

    let radius: f64;
    let center: (f64, f64);
    let mut theta_start: f64;
    let mut theta_end: f64;

    match form {
        ArcForm::IJ(arc_values) => {
            // I and J form
            for val in arc_values {
                match val {
                    ArcVal::X(val) => x = *val,
                    ArcVal::Y(val) => y = *val,
                    ArcVal::I(val) => i = *val,
                    ArcVal::J(val) => j = *val,
                    _ => {}
                }
            }

            // x and y MUST be extracted from command
            debug_assert!(x.is_finite());
            debug_assert!(y.is_finite());

            radius = i.hypot(j);
            center = (current_x + i, current_y + j);

            let delta_start_x = current_x - center.0;
            let delta_start_y = current_y - center.1;

            theta_start = (delta_start_y).atan2(delta_start_x);
            // atan2 returns a value in the range [ -PI, PI].
            // Want a range to be [0,2PI]
            if theta_start < 0_f64 {
                theta_start += 2_f64 * f64::consts::PI;
            }

            let delta_end_x = x - center.0;
            let delta_end_y = y - center.1;
            theta_end = (delta_end_y).atan2(delta_end_x);
            // atan2 returns a value in the range [ -PI, PI].
            // Want a range to be [0,2PI]
            if theta_end < 0_f64 {
                theta_end += 2_f64 * f64::consts::PI;
            }
        }
        ArcForm::R(arc_values) => {
            let mut radius: f64 = f64::NAN;
            // R form

            for val in arc_values {
                match val {
                    ArcVal::X(val) => x = *val,
                    ArcVal::Y(val) => y = *val,
                    ArcVal::R(val) => radius = *val,
                    _ => {
                        // Ignored params
                    }
                }
            }
            debug_assert!(x.is_finite());
            debug_assert!(y.is_finite());
            // r Must be specified from command.
            debug_assert!(radius.is_finite());
            // radius = r;
            // Must solve this  par of simultaneous equations
            // radius * radius = ( center.0 - current_x ) * ( center.0 - current_x ) + ( center.1 - current_y ) * ( center.1 - current_y )
            // radians * radius = (current_x - center.0) * (current_x - center.0) + (current_y - center.1) * (current_y - center.1)
            //
            // which of the two solutions is correct can be determined by realizing that we are moving clockwise or counter clockwise
            todo!();
        }
    }
    ArcParams {
        center,
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
        assert_eq!(arc.center, (5.0, 3.0));
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

    #[test]
    fn troublesome_arc_ij() {
        let arc = compute_arc(
            0.0,
            5.0,
            &ArcForm::IJ(
                [
                    ArcVal::X(5.0),
                    ArcVal::Y(0.0),
                    ArcVal::I(5.0),
                    ArcVal::J(0.0),
                ]
                .into(),
            ),
        );
        assert_eq!(arc.center, (5.0, 5.0));
        assert_eq!(arc.radius, 5.0);
        assert_eq!(round_to_two_decimals(arc.theta_start.to_degrees()), 180_f64);
        assert_eq!(round_to_two_decimals(arc.theta_end.to_degrees()), 270_f64);
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
        assert_eq!(arc.center, (5.0, 3.0));
        assert_eq!(arc.radius, 5.0);
    }
}
