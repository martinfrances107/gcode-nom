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

use gcode_nom::binary::bgcode_parser;
use std::io::{stdin, BufReader, Read};

fn main() -> std::io::Result<()> {
    env_logger::init();
    let stdin = stdin();
    let s_lock = stdin.lock();
    let mut reader = BufReader::new(s_lock);

    let mut buffer = vec![];
    if reader.read_to_end(&mut buffer)? != 0usize {
        match bgcode_parser(&buffer) {
            Ok((_remain, bgcode)) => {
                log::info!("parser succeeded: Valid input");
                println!("{bgcode}");
            }
            Err(e) => {
                log::error!("Unhandled error decoding file {e}");
                panic!("Unhandled error decoding file {e}");
            }
        }
    }

    Ok(())
}
