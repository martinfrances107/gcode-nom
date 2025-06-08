//! gcode2svg
//!
//! A streaming parser
//!
//! Pass a gcode file into stdin a obj file will be output to `StdOut`
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
use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use gcode_nom::binary::gcode_block::extractor::extract_gcode;
use gcode_nom::binary::gcode_block::svg::Svg;
use gcode_nom::binary::inflate::decompress_data_block;
use log::info;

// Occasionally want to apply Blender specific transform.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the file to convert.
    file: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::parse();

    if let Some(file) = args.file {
        info!("File: {file:?}");
        if file.exists() {
            if let Some(ext) = file.extension() {
                if ext == "gcode" {
                    info!("Reading gcode file");
                    let file = File::open(file)?;
                    let buffer = BufReader::new(file);
                    let svg = buffer.lines().map(|l| l.unwrap()).collect::<Svg>();
                    println!("{svg}");
                } else if ext == "bgcode" {
                    info!("Reading bgcode file");
                    let file = File::open(file)?;
                    let mut reader = BufReader::new(file);
                    let mut buffer = vec![];
                    if reader.read_to_end(&mut buffer)? != 0usize {
                        // match bgcode_parser(&buffer) {
                        match extract_gcode(&buffer) {
                            Ok((_remain, gcode_blocks)) => {
                                log::info!("parser succeeded: Valid input");
                                let svg = &gcode_blocks
                                    .iter()
                                    .map(|gcode| {
                                        let (_remain, data) = decompress_data_block(
                                            gcode.data,
                                            &gcode.param.encoding,
                                            &gcode.header,
                                        )
                                        .expect("fail to decompress data block");
                                        String::from_utf8_lossy(&data).to_string()
                                    })
                                    .collect::<String>()
                                    .lines()
                                    .map(std::string::ToString::to_string)
                                    .collect::<Svg>();
                                println!("{svg}");
                            }
                            Err(e) => {
                                log::error!("Unhandled error decoding file {e}");
                                panic!("Unhandled error decoding file {e}");
                            }
                        }
                    }
                } else {
                    eprintln!("File extension is not supported");
                }
            } else {
                eprintln!("File must have an extension");
            }
        } else {
            eprintln!("File does not exist");
        }
    } else {
        info!("Reading from stdin");
        let svg = stdin().lock().lines().map(|l| l.unwrap()).collect::<Svg>();
        println!("{svg}");
    }

    Ok(())
}
