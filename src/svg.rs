use core::marker::PhantomData;

use crate::command::Command;

#[derive(Debug, Default, Clone)]
struct Svg<'a, T> {
    pd: PhantomData<&'a T>,
    parts: Vec<String>,
}

impl<'a, T> FromIterator<Command<'a>> for Svg<'a, T>
where
    T: Default,
{
    fn from_iter<I: IntoIterator<Item = Command<'a>>>(iter: I) -> Self {
        let mut svg: Svg<'a, T> = Self::default();

        let mut abs_coords = true;
        let mut svg = Self::default();
        for command in iter {
            match command {
                Command::G1(payload) => {
                    println!("G1 - payload {payload:?}");
                }
                Command::G21 => svg.parts.push("M0,0".to_string()),
                Command::G90 => {
                    abs_coords = true;
                }
                Command::G91 => todo!(),
                Command::G92(_) => {
                    todo!();
                }
                Command::GDrop(_) | Command::MDrop(_) | Command::Nop => {}
            }
        }

        svg
    }
}
