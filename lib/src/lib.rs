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

/// Parsing rules for gcode commands
pub mod command;

/// Parsing rules for gcode parameters `Pos<number>`
pub mod parms;

/// Absolute or Relative positioning
#[derive(Default, Debug)]
pub enum CoordPos {
    /// As per spec `CoordPos::Absolute` is the dafault
    /// <https://marlinfw.org/docs/gcode/G090.html>
    #[default]
    Absolute,
    /// Relative positioning.
    Relative,
}
