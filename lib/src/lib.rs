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
pub mod pos;
