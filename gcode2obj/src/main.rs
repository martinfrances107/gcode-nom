//! A G-code visualization tool written in [rust](https://www.rust-lang.org/)
//!
//! A nom based parser, outputs a "Wavefront Obj" file which can be imported into blender and a Bevy app for visualization
//!
//! ## How to use
//!
//! Parses `StdIn` as a gcode file - the OBJ file is send to `StdOut` :-
//!
//! ```bash
//! cargo run --release -- < ../assets/bency.gcode > benchy.obj
//! ```
#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![allow(clippy::many_single_char_names)]
use std::io::stdin;
use std::io::BufRead;

mod obj;

use obj::Obj;

use clap::ArgAction;
use clap::Parser;

// Occasionally want to apply Blender specific transform.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Blender compatibility mode.
    #[clap(long, short, action=ArgAction::SetTrue)]
    apply_blender_transform: bool,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let mut obj = stdin().lock().lines().map(|l| l.unwrap()).collect::<Obj>();
    obj.apply_blender_transform = args.apply_blender_transform;
    println!("{obj}");
}

#[cfg(test)]
mod test {
    use gcode_nom::command::Command;

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
}
