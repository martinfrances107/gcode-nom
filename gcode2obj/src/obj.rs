use core::fmt::Display;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

use gcode_nom::command::Command;
use gcode_nom::parms::PosVal;
use gcode_nom::CoordPos;

#[derive(Debug, Clone)]
struct VERTEX(f64, f64, f64);

impl Eq for VERTEX {}
impl PartialEq for VERTEX {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
            && self.1.to_bits() == other.1.to_bits()
            && self.2.to_bits() == other.2.to_bits()
    }
}

impl Hash for VERTEX {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
        self.1.to_bits().hash(state);
        self.2.to_bits().hash(state)
    }
}

///  Structure to compute a index and vertex like buffers.
#[derive(Debug, Default, Clone)]
pub struct Obj {
    /// De-duplicating structure.
    //   Given a point return the postion in the vertex_buffer
    index_store: HashMap<VERTEX, usize>,
    vertex_buffer: Vec<VERTEX>,

    index_buffer: Vec<usize>,
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
        for VERTEX(x, y, z) in &self.vertex_buffer {
            println!("v {} {} {}", x, y, z);
        }

        // l 1 2 3  as list of vertex indicies.
        // '+1' convert from zero based counting.
        // The first index is '1'.
        print!("l");
        for i in &self.index_buffer {
            print!(" {}", i + 1);
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

        let mut abs_coords = CoordPos::default();
        let mut z = 0_f64;
        let mut next_vertex_pos = 0;
        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command not parseable");
            let mut x = f64::NAN;
            let mut y = f64::NAN;
            match command {
                // A non printable move.
                Command::G0(mut payload) => {
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
                            PosVal::E(_) | PosVal::F(_) => {
                                // silently drop
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
                    if !x.is_nan() && !y.is_nan() {
                        let vertex = VERTEX(x, y, z);
                        match obj.index_store.get(&vertex) {
                            Some(index) => {
                                // Push record of exiting vertex to index_buffer.
                                obj.index_buffer.push(*index);
                            }
                            None => {
                                // New entry in vertex_buffer and index_buffer.
                                obj.index_store.insert(vertex.clone(), next_vertex_pos);
                                obj.index_buffer.push(next_vertex_pos);
                                obj.vertex_buffer.push(vertex);
                                next_vertex_pos = next_vertex_pos + 1;
                            }
                        }
                    }
                }
                // Command::G21 => obj.parts.push("M0,0".to_string()),
                Command::G90 => abs_coords = CoordPos::Absolute,
                Command::G91 => abs_coords = CoordPos::Relative,
                Command::G92(_) | Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
                Command::G21 => { // Dropping home},
                }
            }
        }
        obj
    }
}
