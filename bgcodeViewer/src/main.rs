//! Produces a report on a binary encoded gcode file.
//!
//! ( Files with extension *.bgcode ).
//!
//! For example run
//!
//! ```bash
//! RUST_BACKTRACE=1 RUST_LOG=trace cargo run  < ../assets/both\ parts.bgcode
//! ````
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

extern crate env_logger;
extern crate gcode_nom;

use gcode_nom::binary::bgcode_parser_with_checksum;
use gcode_nom::binary::Markdown;
use std::io::{stdin, BufReader, Read};

fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut buffer = vec![];
    if BufReader::new(stdin().lock()).read_to_end(&mut buffer)? != 0usize {
        match bgcode_parser_with_checksum(&buffer) {
            Ok((_remain, bgcode)) => {
                log::info!("parser succeeded: Valid input");
                let mut out = String::new();
                bgcode
                    .markdown(&mut out)
                    .expect("failed to generate markdown");
                println!("{}", &out);
            }
            Err(e) => {
                log::error!("Unhandled error decoding file {e}");
                panic!("Unhandled error decoding file {e}");
            }
        }
    }

    Ok(())
}
