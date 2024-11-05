use std::io::{stdin, BufReader, Read};

extern crate gcode_nom;
use gcode_nom::binary::bgcode_parse;

fn main() -> std::io::Result<()> {
    let stdin = stdin();
    let s_lock = stdin.lock();
    let mut reader = BufReader::new(s_lock);
    // let mut input = reader.fill_buf()?;
    let mut buffer: [u8; 100] = [0u8; 100];
    if reader.read(&mut buffer)? != 0usize {
        match bgcode_parse(&buffer) {
            Ok((_remain, bgcode)) => {
                println!("Valid input");
                println!("{bgcode}")
            }
            Err(e) => {
                eprintln!("Unhandlled error decoding file {e}");
            }
        }
    }

    Ok(())
}
