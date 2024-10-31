use std::io::stdin;
use std::io::BufRead;
use std::io::Result;

mod sentence;

use crate::sentence::Sentence;

fn main() -> Result<()> {
    let stdin = stdin();
    for result in stdin.lock().lines() {
        let line = result?;
        let decoded = Sentence::parse_line(&line);
        match decoded {
            Ok((payload, Sentence::G1)) => {
                println!("G1 payload {payload}");
            }
            Ok((payload, Sentence::G21)) => {
                println!("G21 {payload}");
            }
            Ok((payload, Sentence::GDrop(code))) => {
                println!("G_Drop code {code:#?}, payload {payload}, ");
            }
            Ok((payload, Sentence::MDrop(code))) => {
                println!("G_Drop code {code:#?}, payload {payload}, ");
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to decode sentence {e}");
            }
        };
    }
    Ok(())
}
