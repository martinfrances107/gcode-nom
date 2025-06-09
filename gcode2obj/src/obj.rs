//! Data structures associated with obj files
//!
//! In this context obj files have :-
//!
//! a list of vertices, V
//! a list of indices into V.
//!
use core::fmt::Display;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem;

use gcode_nom::compute_arc;
use gcode_nom::compute_step_params;
use gcode_nom::ArcParams;
use gcode_nom::StepParams;
use hashbrown::HashMap;

use gcode_nom::binary::gcode_block::GCodeBlock;
use gcode_nom::binary::inflate::decompress_data_block;
use gcode_nom::command::Command;
use gcode_nom::params::PosVal;
use gcode_nom::PositionMode;

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
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Obj {
    /// De-duplicating structure.
    //   Given a point return the position in the vertex_buffer
    index_store: HashMap<Vertex, usize>,
    vertex_buffer: Vec<Vertex>,

    // A collection of lines
    // A collection index_buffers representing line.
    lines: Vec<Vec<usize>>,

    // Blender axes compatible mode.
    pub apply_blender_transform: bool,
}

// Display the object

// Blender's obj importer applies a non-standard transform
//
// Blender's red +X axis =>> Obj +X axis
// Blender's green +Y axis =>> Obj -Z axis
// Blender's blue +Z axis =>> Obj +Y axis
//
// "This is intentional since most OBJ files have a different UP to whats used
//  in blender, blender switches the axis on import/export intentionally."
//
// Campbell Barton
impl Display for Obj {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Write out vertex buffer
        // "List of geometric vertices, with (x, y, z, [w]) coordinates, w is optional and defaults to 1.0."
        // [spec](<https://en.wikipedia.org/wiki/Wavefront_.obj_file>)
        if self.apply_blender_transform {
            for Vertex(x, y, z) in &self.vertex_buffer {
                writeln!(f, "v {x} {z} {y}")?;
            }
        } else {
            for Vertex(x, y, z) in &self.vertex_buffer {
                writeln!(f, "v {x} {y} {z}")?;
            }
        }

        // Write out sequence of index buffers.
        for line in &self.lines {
            // line "l 1 2 3"  list of vertex indices.
            if line.len() > 1 {
                write!(f, "l")?;
                for i in line {
                    // '+1' convert from zero based counting.
                    // The first index is '1'.
                    write!(f, " {}", i + 1)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl<'a> FromIterator<GCodeBlock<'a>> for Obj {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = GCodeBlock<'a>>,
    {
        iter.into_iter()
            .flat_map(|gcode| {
                let (_remain, data) =
                    decompress_data_block(gcode.data, &gcode.param.encoding, &gcode.header)
                        .expect("fail to decompress data block");

                String::from_utf8_lossy(&data)
                    .to_string()
                    .lines()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .collect::<Self>()
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
        let mut position_mode = PositionMode::default();
        let mut next_vertex_pos = 0;
        let mut line_buffer = vec![];
        let mut current_x = 0_f64;
        let mut current_y = 0_f64;
        let mut current_z = 0_f64;

        for line in iter {
            let (_, command) = Command::parse_line(&line).expect("Command not parsable");
            match command {
                // Treat G0 and G1 command identically.
                //
                // A G0 is a non-printing move but E is present in files seen in the wild.
                // (In the assets directory see the gears and benchy2 files.)
                Command::G1(mut payload) | Command::G0(mut payload) => {
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
                            PosVal::F(_) => {
                                // Silently drop feed-rate adjustment.
                            }
                            pos_bad => {
                                log::debug!(
                                    "Obj: Unexpected param seen in Command::G1 {pos_bad:?}"
                                );
                            }
                        }
                    }

                    let x = if x_param.is_nan() {
                        current_x
                    } else {
                        match position_mode {
                            PositionMode::Absolute => x_param,
                            PositionMode::Relative => current_x + x_param,
                        }
                    };

                    let y = if y_param.is_nan() {
                        current_y
                    } else {
                        match position_mode {
                            PositionMode::Absolute => y_param,
                            PositionMode::Relative => current_y + y_param,
                        }
                    };

                    let z = if z_param.is_nan() {
                        current_z
                    } else {
                        match position_mode {
                            PositionMode::Absolute => z_param,
                            PositionMode::Relative => current_z + z_param,
                        }
                    };

                    let vertex = Vertex(x, y, z);
                    if is_extruding {
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
                    } else {
                        // Not extruding
                        //
                        // TODO: set the capacity of the complete_line
                        // to the last good capacity.
                        let mut complete_line = vec![];
                        mem::swap(&mut line_buffer, &mut complete_line);
                        obj.lines.push(complete_line);

                        // The first entry in the new line buffer is current position.
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
                    current_x = x;
                    current_y = y;
                    current_z = z;
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

                    // x,y are the position of the head in absolute units.
                    let mut x = f64::NAN;
                    let mut y = f64::NAN;

                    // For loop: f64 has a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=n_steps as u64 {
                        let theta = theta_start + (i as f64 * theta_step);
                        x = origin.0 + radius * theta.cos();
                        y = origin.1 + radius * theta.sin();
                        let vertex = Vertex(x, y, current_z);

                        // This command is always extruding.
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

                    current_x = x;
                    current_y = y;
                }
                Command::G3(arc_form) => {
                    // Counter-clockwise arc
                    let ArcParams {
                        origin,
                        radius,
                        theta_start,
                        mut theta_end,
                    } = compute_arc(current_x, current_y, &arc_form);

                    // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                    // All values here are in the range 0<=theta<2PI
                    // We are rotating clockwise
                    // in this cased the start angle of 0 should be read as 2PI
                    if theta_end == 0_f64 {
                        theta_end = 2_f64 * std::f64::consts::PI;
                    }

                    let StepParams {
                        n_steps,
                        theta_step,
                    } = compute_step_params(theta_start, theta_end, radius);

                    // x,y are the position of the head in absolute units.
                    let mut x = f64::NAN;
                    let mut y = f64::NAN;

                    // For loop: f64 has a problem with numerical accuracy
                    // specifically the comparing limit.
                    // rust idiomatically insists on indexed here
                    for i in 0..=n_steps as u64 {
                        let theta = theta_start + (i as f64 * theta_step);
                        x = origin.0 + radius * theta.cos();
                        y = origin.1 + radius * theta.sin();
                        let vertex = Vertex(x, y, current_z);

                        // This command is always extruding.
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

                    current_x = x;
                    current_y = y;
                }
                // G90 and G91 set the position mode.
                Command::G90 => {
                    position_mode = PositionMode::Absolute;
                }
                Command::G91 => {
                    position_mode = PositionMode::Relative;
                }
                // G92 Set Current Position
                Command::G92(mut params) => {
                    for p in params.drain() {
                        match p {
                            PosVal::E(e) => {
                                // Negative values the extruder is "wiping"
                                // or sucking filament back into the extruder.
                                if e <= 0_f64 {
                                    // The extrude rate is going to zero
                                    // enter "move mode" ..ie not laying down filament.
                                    is_extruding = false;
                                    // For Visualization we start a new line.
                                    //
                                    // TODO: set the capacity of the complete_line
                                    // to the last good capacity.
                                    let mut complete_line = vec![];
                                    mem::swap(&mut line_buffer, &mut complete_line);
                                    obj.lines.push(complete_line);
                                }
                            }
                            PosVal::X(x) => {
                                // Set the current X position.
                                if position_mode == PositionMode::Absolute {
                                    current_x = x;
                                } else {
                                    current_x += x;
                                }
                            }
                            PosVal::Y(y) => {
                                // Set the current Y position.
                                if position_mode == PositionMode::Absolute {
                                    current_y = y;
                                } else {
                                    current_y += y;
                                }
                            }
                            PosVal::Z(z) => {
                                // Set the current Z position.
                                if position_mode == PositionMode::Absolute {
                                    current_z = z;
                                } else {
                                    current_z += z;
                                }
                            }
                            bad => {
                                // Dropping unexpected params
                                log::debug!("G92 unhandled set position code. P{bad:#?}");
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        obj
    }
}
