use std::io::{stdin, BufReader, Read};

extern crate gcode_nom;
use gcode_nom::binary::bgcode_parser;

fn main() -> std::io::Result<()> {
    let stdin = stdin();
    let s_lock = stdin.lock();
    let mut reader = BufReader::new(s_lock);

    // TODO Must stream properly.
    const N: usize = 3471042;
    let mut buffer: [u8; N] = [0; N];
    if reader.read(&mut buffer)? != 0usize {
        match bgcode_parser(&buffer) {
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
