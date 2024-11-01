use std::collections::HashSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::preceded;
use nom::IResult;

use crate::pos::PosVal;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Command<'a> {
    /// linear move
    G1(HashSet<PosVal>),
    /// Home all axes
    G21,
    /// use absolute coordinates.
    G90,
    /// Drop - ie no further action.
    GDrop(&'a str),
    MDrop(&'a str),

    /// No Operation
    Nop,
}

impl<'a> Command<'a> {
    pub(crate) fn parse_line(line: &str) -> IResult<&str, Command> {
        // Most common first.
        alt((
            parse_g1,
            map(tag("G21 "), |_| Command::G21),
            map(tag("G90 "), |_| Command::G90),
            map(g_drop, Command::GDrop),
            map(m_drop, Command::MDrop),
            map(tag(";"), |_| Command::Nop),
            map(tag(""), |_| Command::Nop),
        ))(line)
    }
}

// G commands that require no further action
fn g_drop(i: &str) -> IResult<&str, &str> {
    preceded(tag("G "), digit1)(i)
}

fn parse_g1(i: &str) -> IResult<&str, Command> {
    preceded(
        tag("G1 "),
        map(pos_many, |val: Vec<PosVal>| {
            let mut hs: HashSet<PosVal> = HashSet::new();
            for item in val {
                hs.insert(item);
            }

            Command::G1(hs)
        }),
    )(i)
}

fn pos_many(i: &str) -> IResult<&str, Vec<PosVal>> {
    separated_list1(tag(" "), pos_val)(i)
}

fn pos_val(i: &str) -> IResult<&str, PosVal> {
    alt((
        parse_a, parse_b, parse_c, parse_e, parse_f, parse_g, parse_s, parse_u, parse_v, parse_w,
        parse_x, parse_y, parse_z,
    ))(i)
}

// TODO can I use a macro here!!
fn parse_a(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("A"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::A cannot decode number.");
        PosVal::A(number)
    })(i)
}

fn parse_b(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("B"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::B cannot decode number.");
        PosVal::B(number)
    })(i)
}

fn parse_c(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("C"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::C cannot decode number.");
        PosVal::C(number)
    })(i)
}

fn parse_e(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("E"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::E cannot decode number.");
        PosVal::E(number)
    })(i)
}

fn parse_f(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("F"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::F cannot decode number.");
        PosVal::F(number)
    })(i)
}
fn parse_g(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("G"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::G cannot decode number.");
        PosVal::G(number)
    })(i)
}
// here
fn parse_s(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("S"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::S cannot decode number.");
        PosVal::S(number)
    })(i)
}
fn parse_u(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("U"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::U cannot decode number.");
        PosVal::U(number)
    })(i)
}
fn parse_v(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("V"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::V cannot decode number.");
        PosVal::V(number)
    })(i)
}

fn parse_w(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("W"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::W cannot decode number.");
        PosVal::W(number)
    })(i)
}
fn parse_x(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("X"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::X cannot decode number.");
        PosVal::X(number)
    })(i)
}
fn parse_y(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("Y"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::Y cannot decode number.");
        PosVal::Y(number)
    })(i)
}
fn parse_z(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("Z"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::Z cannot decode number.");
        PosVal::Z(number)
    })(i)
}

fn m_drop(i: &str) -> IResult<&str, &str> {
    preceded(tag("M"), digit1)(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_comment() {}

    // #[test]
    // fn many() {
    //     let input = "E5";
    //     let a = pos_many(input);
    //     println!("a {a:#?}");

    //     let b = parse_g1("G1 E5");
    //     println!("b {b:#?}");

    //     let c = parse_g1("G1 E5 F3 ; lift nozzle");
    //     println!("c {c:#?}");
    // }

    #[test]
    fn g1() {
        // let default = PosPayload::<f64>::default();

        let text_commands = [
            ("G1 Z5", Command::G1([PosVal::Z(5_f64)].into())),
            (
                "G1 Z5 F5000 ; lift nozzle",
                Command::G1([PosVal::Z(5_f64), PosVal::F(5000_f64)].into()),
            ),
            // (
            //     "G1 E1.00000 F1800.00000 ; text",
            //     Command::G1([PosVal::E(1.0_f64), PosVal::F(1800_f64)].into()),
            // ),
            // (
            //     "G1 Z0.350 F7800.000",
            //     Command::G1([PosVal::Z(0.350_f64), PosVal::F(7800_f64)].into()),
            // ),
        ];

        for (line, expected) in text_commands {
            let actual = parse_g1(line);
            println!("expected  {expected:?}");
            println!("actual  {actual:?}");
            match actual {
                Ok((_, actual)) => {
                    assert_eq!(actual, expected);
                }
                Err(_) => {
                    assert!(false);
                }
            }
        }
    }
    #[test]
    fn parse_g_drop() {}
}
