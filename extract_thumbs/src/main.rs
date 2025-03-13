//! Extract thumbnails from a gcode/bgcode file.
//!
//! For example
//!
//! cargo run --release benchy.gcode
//!
//! generates creates files
//!
//! benchy.png
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

extern crate clap;

use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use clap::Parser;
use gcode_nom::binary::bgcode_parser;

// Extract thumbnails from a .gcode/bgcode
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Name of the file to convert.
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Cli::parse();

    let metadata = fs::metadata(&args.input)?;
    // TODO: Why is the buffer size half the length?
    let mut buffer = Vec::with_capacity((metadata.len() / 2) as usize);

    log::info!("Loading filename {} ... ", args.input.display());
    let mut f = File::open(args.input)?;
    log::info!("done");

    let state = f.read_to_end(&mut buffer);
    match state {
        Ok(_) => log::info!("state {state:?}"),
        Err(e) => log::error!("error {e:?}"),
    }

    match bgcode_parser(&buffer) {
        Ok((_remain, bgcode)) => {
            for (i, thumbnail_block) in bgcode.thumbnails.iter().enumerate() {
                let path_str = format!(
                    "./thumb_{i}_{}x{}.{}",
                    thumbnail_block.param.width,
                    thumbnail_block.param.height,
                    thumbnail_block.param.format
                );

                println!("writing {path_str:#?}");
                std::fs::write(path_str, &thumbnail_block.data).unwrap();
            }
        }
        Err(e) => {
            log::error!("Unhandled x error decoding file {e}");
        }
    }

    Ok(())
}
