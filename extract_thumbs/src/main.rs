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
extern crate clap;

use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use clap::Parser;
use gcode_nom::binary::bgcode_parser;
use log::log;

// Extract thumbnails from a .gcode/bgcode
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Name of the file to convert.
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let metadata = fs::metadata(&args.input)?;
    let len = metadata.len() as usize;
    let mut buffer = Vec::with_capacity(len / 2);

    log::info!("Loading filename {} ... ", args.input.clone().display());
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
                std::fs::write(path_str, &thumbnail_block.data).unwrap()
            }
        }
        Err(e) => {
            log::error!("Unhandled x error decoding file {e}");
        }
    }

    Ok(())
}
