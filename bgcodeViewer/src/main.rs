extern crate gcode_nom;

use std::io::{stdin, BufReader, Read};

use env_logger::Builder;
use gcode_nom::binary::bgcode_parser;
use log::LevelFilter;

fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Debug).init();

    let stdin = stdin();
    let s_lock = stdin.lock();
    let mut reader = BufReader::new(s_lock);

    let mut buffer = vec![];
    if reader.read_to_end(&mut buffer)? != 0usize {
        match bgcode_parser(&buffer) {
            Ok((_remain, bgcode)) => {
                log::info!("parser succeeded: Valid input");
                println!("{bgcode}")
            }
            Err(e) => {
                log::error!("Unhandled error decoding file {e}");
                panic!("Unhandled error decoding file {e}");
            }
        }
    }

    Ok(())
}
