use crate::command::Command;

#[derive(Debug, Default, Clone)]
struct SVG {
    parts: Vec<String>,
}

struct ParseError;

impl<'a> TryFrom<Vec<Command<'a>>> for SVG {
    type Error = ParseError;

    fn try_from(commands: Vec<Command<'a>>) -> Result<Self, Self::Error> {
        let mut abs_coords = true;
        let mut svg = SVG::default();
        for command in commands {
            match command {
                Command::G1(payload) => {
                    println!("G1 - payload {payload:?}");
                }
                Command::G21 => svg.parts.push("M0,0".to_string()),
                Command::G90 => {
                    abs_coords = true;
                }
                Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
            }
        }
        Ok(svg)
    }
}
