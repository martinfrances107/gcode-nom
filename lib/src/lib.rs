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

/// Streaming for binary gcode files
pub mod binary;
/// Parsing rules for gcode commands
pub mod command;
mod double;
/// Parsing rules for G0/G1 commands
pub mod params;

/// Parsing rules for G2/G3 arc commands
pub mod arc_params;

/// Absolute or Relative positioning
#[derive(Default, Debug)]
pub enum PositionMode {
    /// As per spec `Positionmode::Absolute` is the default
    /// <https://marlinfw.org/docs/gcode/G090.html>
    #[default]
    Absolute,
    /// Relative positioning.
    Relative,
}
