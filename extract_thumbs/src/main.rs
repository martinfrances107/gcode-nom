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
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
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
    let args = Cli::parse();

    let metadata = fs::metadata(&args.input)?;
    let len = metadata.len() as usize;
    let mut buffer = Vec::with_capacity(len / 2);

    print!("Loading filename {} ... ", args.input.clone().display());
    let mut f = File::open(args.input)?;
    println!("done");

    // let mut reader = BufReader::new(f);
    let state = f.read_to_end(&mut buffer);
    match state {
        Ok(s) => println!("state {state:?}"),
        Err(e) => println!("error {e:?}"),
    }

    let index = 0;

    match bgcode_parser(&buffer) {
        Ok((_remain, bgcode)) => {
            println!("Valid input");
            for thumbnail_block in bgcode.thumbnails {
                let path_str = format!(
                    "./thumb_{}x{}{index}.{}",
                    thumbnail_block.param.width,
                    thumbnail_block.param.height,
                    thumbnail_block.param.format
                );

                println!("path str {path_str:#?}");
                // let mut file = fs::OpenOptions::new()
                //     // .create(true) // To create a new file
                //     .write(true)
                //     // either use the ? operator or unwrap since it returns a Result
                //     .open(path_str)?;
                std::fs::write(path_str, &thumbnail_block.data).unwrap()
                // file.write_all(&thumbnail_block.data)?;
            }
        }
        Err(e) => {
            println!("Unhandlled x error decoding file {e}");
        }
    }

    Ok(())
}
