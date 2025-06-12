//! Data structures associated with obj files
//!
//! In this context obj files have :-
//!
//! a list of vertices, V
//! a list of indices into V.
//!
use core::f64::consts::TAU;
use core::fmt::Display;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem;
use core::panic;

use hashbrown::HashMap;

use gcode_nom::binary::gcode_block::GCodeBlock;
use gcode_nom::binary::inflate::decompress_data_block;
use gcode_nom::command::Command;
use gcode_nom::compute_arc;
use gcode_nom::params::head::PosVal;
use gcode_nom::params::mp::MultiPartVal;
use gcode_nom::ArcParams;
use gcode_nom::PositionMode;
use gcode_nom::MM_PER_ARC_SEGMENT;

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
    vertex_store: HashMap<Vertex, usize>,
    vertex_buffer: Vec<Vertex>,

    // Multipart: One "OBJ" file can contain multiple objects.
    //
    // Keyed by the slot id.
    //
    // Each slot is a vector of point indexes representing line.
    lines_store: HashMap<i128, Vec<Vec<usize>>>,

    // Keyed by the slot id.
    name_store: HashMap<i128, String>,

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

        // If the GCODE file contains only one object omit the 'o' definition.
        //
        // Blender 'Import" script will use the filename as the object name.
        let display_object_name = self.lines_store.keys().len() != 1;

        // Write out a sequence of objects.
        for (slot_id, lines) in &self.lines_store {
            if display_object_name {
                // "o object_name"  - the name of the object.
                // The slot_id is the object name.
                if slot_id < &0 {
                    writeln!(f, "o purge_tower_{slot_id}")?;
                } else {
                    writeln!(f, "o object_{slot_id}")?;
                }
            }

            // Write out sequence of index buffers.
            for line in lines {
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

        // Multipart objects
        //
        // Each object get a unique entry in the line buffer store.

        // For gcode files that do not contain Command::M486 directives
        // the initial state MUST be slot id 0 is "in store" and active.
        //
        // Option as job can be cancelled without specifying the next object.
        let mut object_id = Some(0);
        // keyed by object_id
        let mut line_buffer_store: HashMap<i128, Vec<usize>> = HashMap::from([(0, vec![])]);

        let mut is_extruding = true;
        let mut position_mode = PositionMode::default();
        let mut next_vertex_pos = 0;

        let mut current_x = 0_f64;
        let mut current_y = 0_f64;
        let mut current_z = 0_f64;

        let mut origin_x = 0_f64;
        let mut origin_y = 0_f64;
        let mut origin_z = 0_f64;

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
                            pos_bad => {
                                log::debug!(
                                    "Obj: Unexpected param seen in Command::G1 {pos_bad:?}"
                                );
                            }
                        }
                    }

                    if !x_param.is_nan() {
                        current_x = match position_mode {
                            PositionMode::Absolute => x_param,
                            PositionMode::Relative => current_x + x_param,
                        };
                    }

                    if !y_param.is_nan() {
                        current_y = match position_mode {
                            PositionMode::Absolute => y_param,
                            PositionMode::Relative => current_y + y_param,
                        }
                    }

                    if !z_param.is_nan() {
                        current_z = match position_mode {
                            PositionMode::Absolute => z_param,
                            PositionMode::Relative => current_z + z_param,
                        }
                    }

                    let vertex = Vertex(
                        origin_x + current_x,
                        origin_y + current_y,
                        origin_z + current_z,
                    );
                    if is_extruding {
                        if let Some(v_index) = obj.vertex_store.get(&vertex) {
                            // Push record of exiting vertex to index_buffer.
                            if let Some(id) = object_id {
                                if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                                    line_buffer.push(*v_index);
                                } else {
                                    debug_assert!(
                                        false,
                                        "failed to get line buffer for object id {id}"
                                    );
                                }
                            }
                        } else {
                            // New entry in vertex_buffer and index_buffer.
                            if let Some(id) = object_id {
                                if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                                    obj.vertex_store.insert(vertex.clone(), next_vertex_pos);
                                    line_buffer.push(next_vertex_pos);
                                    obj.vertex_buffer.push(vertex);
                                    next_vertex_pos += 1;
                                } else {
                                    debug_assert!(
                                        false,
                                        "failed to get line buffer for object id {id}"
                                    );
                                }
                            }
                        }
                    } else {
                        // Not extruding
                        //
                        if let Some(id) = object_id {
                            if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                                // TODO: set the capacity of the complete_line
                                // to the last good capacity.
                                let mut complete_line = vec![];
                                mem::swap(line_buffer, &mut complete_line);
                                if let Some(obj_line_store) = obj.lines_store.get_mut(&id) {
                                    // If the line store already has a line for this part_id
                                    // then append to it.
                                    obj_line_store.push(complete_line);
                                } else {
                                    // Otherwise create a new entry in the lines_store.
                                    // obj.lines_store.insert(id, vec![complete_line]);
                                    panic!("failed to get line buffer for object id {id}");
                                }

                                // The first entry in the new line buffer is current position.
                                if let Some(v_index) = obj.vertex_store.get(&vertex) {
                                    // Push record of exiting vertex to index_buffer.
                                    line_buffer.push(*v_index);
                                } else {
                                    // New entry in vertex_buffer and index_buffer.
                                    obj.vertex_store.insert(vertex.clone(), next_vertex_pos);
                                    line_buffer.push(next_vertex_pos);
                                    obj.vertex_buffer.push(vertex);
                                    next_vertex_pos += 1;
                                }
                            } else {
                                debug_assert!(
                                    false,
                                    "failed to get line buffer for object id {id}"
                                );
                                panic!("failed to get line buffer for object id {id}");
                            }
                        }
                    }
                }
                Command::G2(arc_form) => {
                    if let Some(id) = object_id {
                        if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                            // Clockwise arc
                            let ArcParams {
                                center,
                                radius,
                                mut theta_start,
                                theta_end,
                            } = compute_arc(current_x, current_y, &arc_form);

                            // Regarding the Ambiguity/Equivalence  of the angles 0 and 2PI
                            // All values here are in the range 0<=theta<2PI
                            // We are rotating clockwise
                            // in this cased the start angle of 0 should be read as 2PI
                            if theta_start == 0_f64 {
                                theta_start = TAU;
                            }

                            let delta_theta = if theta_start < theta_end {
                                // Adjust for zero crossing
                                // say 115 -> 304 degrees
                                // delta_theta = 115 + (360 - 304 ) = 170
                                theta_start + (TAU - theta_end)
                            } else {
                                theta_start - theta_end
                            };
                            let total_arc_length = delta_theta * radius;
                            // n_steps must be a number > 0
                            let n_steps = (total_arc_length / MM_PER_ARC_SEGMENT).ceil();
                            let theta_step = delta_theta / n_steps;

                            // x,y are the position of the head in absolute units.
                            let mut x = f64::NAN;
                            let mut y = f64::NAN;

                            // For loop: f64 has a problem with numerical accuracy
                            // specifically the comparing limit.
                            // rust idiomatically insists on indexed here
                            for i in 0..=n_steps as u64 {
                                let theta = (theta_start - (i as f64 * theta_step)) % TAU;
                                x = center.0 + radius * theta.cos();
                                y = center.1 + radius * theta.sin();
                                let vertex =
                                    Vertex(origin_x + x, origin_y + y, origin_z + current_z);

                                // This command is always extruding.
                                if let Some(v_index) = obj.vertex_store.get(&vertex) {
                                    // Push record of exiting vertex to index_buffer.
                                    line_buffer.push(*v_index);
                                } else {
                                    // New entry in vertex_buffer and index_buffer.
                                    obj.vertex_store.insert(vertex.clone(), next_vertex_pos);
                                    line_buffer.push(next_vertex_pos);
                                    obj.vertex_buffer.push(vertex);
                                    next_vertex_pos += 1;
                                }
                            }

                            current_x = x;
                            current_y = y;
                        } else {
                            panic!("failed to get line buffer for object id {id}");
                        }
                    }
                }
                Command::G3(arc_form) => {
                    if let Some(id) = object_id {
                        if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                            // Counter-clockwise arc
                            let ArcParams {
                                center,
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

                            let delta_theta = if theta_start > theta_end {
                                // Adjust for zero crossing
                                // say 306 -> 115 degrees
                                // delta_theta = (360 - 305 ) + 115 = 170
                                TAU - theta_start + theta_end
                            } else {
                                theta_end - theta_start
                            };
                            let total_arc_length = delta_theta * radius;
                            // n_steps must be a number > 0
                            let n_steps = (total_arc_length / MM_PER_ARC_SEGMENT).ceil();
                            let theta_step = delta_theta / n_steps;

                            // x,y are the position of the head in absolute units.
                            let mut x = f64::NAN;
                            let mut y = f64::NAN;

                            // For loop: f64 has a problem with numerical accuracy
                            // specifically the comparing limit.
                            // rust idiomatically insists on indexed here
                            for i in 0..=n_steps as u64 {
                                let theta = (theta_start + (i as f64 * theta_step)) % TAU;
                                x = center.0 + radius * theta.cos();
                                y = center.1 + radius * theta.sin();
                                let vertex =
                                    Vertex(origin_x + x, origin_y + y, origin_z + current_z);

                                // This command is always extruding.
                                if let Some(v_index) = obj.vertex_store.get(&vertex) {
                                    // Push record of exiting vertex to index_buffer.
                                    line_buffer.push(*v_index);
                                } else {
                                    // New entry in vertex_buffer and index_buffer.
                                    obj.vertex_store.insert(vertex.clone(), next_vertex_pos);
                                    line_buffer.push(next_vertex_pos);
                                    obj.vertex_buffer.push(vertex);
                                    next_vertex_pos += 1;
                                }
                            }

                            current_x = x;
                            current_y = y;
                        } else {
                            panic!("failed to get line buffer for object id {id}");
                        }
                    }
                }
                // G90 and G91 set the position mode.
                Command::G90 => position_mode = PositionMode::Absolute,
                Command::G91 => position_mode = PositionMode::Relative,
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
                                    if let Some(id) = object_id {
                                        if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                                            let mut complete_line = vec![];
                                            mem::swap(line_buffer, &mut complete_line);
                                            // obj.lines.push(complete_line);
                                            if let Some(obj_line_store) =
                                                obj.lines_store.get_mut(&id)
                                            {
                                                // If the line store already has a line for this part_id
                                                // then append to it.
                                                obj_line_store.push(complete_line);
                                            } else {
                                                // Otherwise create a new entry in the lines_store.
                                                obj.lines_store.insert(id, vec![complete_line]);
                                            }
                                        }
                                    }
                                } else {
                                    // Starting to extrude
                                    // debug_assert!(line_buffer.is_empty());
                                    is_extruding = true;
                                }
                            }
                            PosVal::X(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_x = current_x - val;
                                    current_x = val;
                                }

                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            PosVal::Y(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_y = current_x - val;
                                    current_y = val;
                                }
                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            PosVal::Z(val) => match position_mode {
                                PositionMode::Absolute => {
                                    origin_z = current_z - val;
                                    current_z = val;
                                }
                                PositionMode::Relative => {
                                    unimplemented!("Relative position mode origin adjust ");
                                }
                            },
                            bad => {
                                // Dropping unexpected params
                                log::debug!("G92 unhandled set position code. P{bad:#?}");
                            }
                        }
                    }
                }
                Command::M486(val) => {
                    match val {
                        MultiPartVal::A(new_name) => {
                            if let Some(id) = object_id {
                                if let Some(name) = obj.name_store.get_mut(&id) {
                                    *name = new_name;
                                }
                            }
                        }
                        MultiPartVal::C => {
                            // Cancel job
                            object_id = None;
                        }
                        MultiPartVal::P(_) => {
                            object_id = None;
                        }
                        MultiPartVal::S(val, n) => {
                            // TODO used name once parsing is implemented.
                            // Start and un-cancel are the same action.
                            object_id = Some(val);
                            // Initialize both the entry in the line_buffer_store and the
                            // entry in the global lines_store.
                            line_buffer_store.entry(val).or_insert(vec![]);
                            obj.lines_store.entry(val).or_insert(vec![]);

                            if let Some(name) = n {
                                obj.name_store.insert(val, name);
                            } else {
                                // If no name is given then use the default.
                                obj.name_store.insert(val, format!("object_{val}"));
                            }
                        }
                        MultiPartVal::T(_) => {
                            // Set max job.
                            //
                            // The intent is to allow the LCD on the printer to display
                            // the number of objects in the job.
                        }
                        MultiPartVal::U(val) => {
                            // Start and un-cancel are the same action.
                            object_id = Some(val);
                            // Initialize both the entry in the line_buffer_store and the
                            // entry in the global lines_store.
                            line_buffer_store.entry(val).or_insert(vec![]);
                            obj.lines_store.entry(val).or_insert(vec![]);
                        }
                    }
                }
                _ => {
                    // println!("Dropping command {command:#?}");
                }
            }
        }

        if let Some(id) = object_id {
            if let Some(line_buffer) = line_buffer_store.get_mut(&id) {
                if !line_buffer.is_empty() {
                    // Print head is still extruding at end.

                    if let Some(obj_line_store) = obj.lines_store.get_mut(&id) {
                        // If the line store already has a line for this part_id
                        // then append to it.
                        obj_line_store.push(line_buffer.clone());
                    } else {
                        // Otherwise create a new entry in the lines_store.
                        obj.lines_store.insert(id, vec![line_buffer.clone()]);
                    }
                }
            }
        }

        obj
    }
}
