use core::fmt::Display;
use core::mem;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use gcode_nom::command::Command;
use gcode_nom::parms::PosVal;
use gcode_nom::CoordPos;

#[derive(Debug, Clone)]
struct Vertex(f64, f64, f64);

impl Eq for Vertex {}
impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
            && self.1.to_bits() == other.1.to_bits()
            && self.2.to_bits() == other.2.to_bits()
    }
}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
        self.1.to_bits().hash(state);
        self.2.to_bits().hash(state);
    }
}

///  Structure to compute a index and vertex like buffers.
#[derive(Debug, Default, Clone)]
pub struct Obj {
    /// De-duplicating structure.
    //   Given a point return the postion in the vertex_buffer
    index_store: HashMap<Vertex, usize>,
    vertex_buffer: Vec<Vertex>,

    // A collection of lines
    // A collection index_buffers reperesenting line.
    lines: Vec<Vec<usize>>,
}

// A line could not be decoded as an G-Code command
// #[derive(Debug, Clone)]
// struct GCodeError;

// impl std::fmt::Display for GCodeError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "invalid g-code statement")
//     }
// }

impl Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Output vertex buffer
        // "List of geometric vertices, with (x, y, z, [w]) coordinates, w is optional and defaults to 1.0."
        // [spec](<https://en.wikipedia.org/wiki/Wavefront_.obj_file>)
        for Vertex(x, y, z) in &self.vertex_buffer {
            writeln!(f, "v {x} {y} {z}")?;
        }

        for line in &self.lines {
            // line "l 1 2 3"  list of vertex indicies.
            write!(f, "l")?;
            for i in line {
                // '+1' convert from zero based counting.
                // The first index is '1'.
                write!(f, " {}", i + 1)?;
            }
        }
        Ok(())
    }
}

/// TODO: Want to iterate over something more flexible
/// ie. Drop String for something more generic `AsRef<&str>`?
impl FromIterator<String> for Obj {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut obj = Self::default();

        let mut is_extruding = true;
        let mut abs_coords = CoordPos::default();
        let mut z = 0_f64;
        let mut next_vertex_pos = 0;
        let mut line_buffer = vec![];
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command not parseable");
            let mut x = f64::NAN;
            let mut y = f64::NAN;
            match command {
                // A non printable move.
                Command::G0(_) => {
                    todo!();
                }
                // A printable move.
                Command::G1(mut payload) => {
                    let params = payload.drain();
                    for param in params {
                        match param {
                            PosVal::X(val) => x = val,
                            PosVal::Y(val) => y = val,
                            PosVal::Z(val) => z = val,
                            PosVal::E(val) => is_extruding = val != 0_f64,
                            PosVal::F(_) => {
                                // Silently drop feedrate adjustment.
                            }
                            pos_bad => {
                                eprintln!("Unexpected param seen in Command::G1 {pos_bad:?}");
                            }
                        }
                    }

                    // TODO Must handle abs and relative position state.
                    // "abs_coords"

                    // Valid `Command::G1` -  Where X and Y and undefined
                    //
                    // "G1 E2.72551 F1800.00000"
                    if !x.is_nan() && !y.is_nan() && is_extruding {
                        let vertex = Vertex(x, y, z);
                        if let Some(index) = obj.index_store.get(&vertex) {
                            // Push record of exiting vertex to index_buffer.
                            line_buffer.push(*index);
                        } else {
                            // New entry in vertex_buffer and index_buffer.
                            obj.index_store.insert(vertex.clone(), next_vertex_pos);
                            line_buffer.push(next_vertex_pos);
                            obj.vertex_buffer.push(vertex);
                            next_vertex_pos += 1;
                        }
                    }
                }
                Command::G90 => abs_coords = CoordPos::Absolute,
                Command::G91 => abs_coords = CoordPos::Relative,
                Command::G92(mut params) => {
                    for p in params.drain() {
                        match p {
                            PosVal::E(e) => {
                                // TODO What about mode Abs or relative?
                                if e == 0_f64 {
                                    // The extrude rate is going to zero
                                    // enter MoveMode ..ie not laying down fibre.
                                    is_extruding = false;
                                    // For Visualisation we start a new line.
                                    //
                                    // TODO: set the capacity of the complete_line
                                    // to the last good capacity.
                                    let mut complete_line = vec![];
                                    mem::swap(&mut line_buffer, &mut complete_line);
                                    obj.lines.push(complete_line);
                                }
                            }
                            bad => {
                                // Dropping unexpected params
                                eprintln!("G92 unhandled set position code. P{bad:#?}");
                            }
                        }
                    }
                }
                Command::GDrop(_) | Command::MDrop(_) | Command::Nop | _ => {}
            }
        }
        obj
    }
}
