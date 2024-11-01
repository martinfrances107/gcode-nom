use std::hash::Hash;
use std::hash::Hasher;

use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;

#[derive(Clone, Debug)]
pub(crate) enum PosVal {
    /// Axis set { A, B, C }
    A(f64),
    B(f64),
    C(f64),
    // Extruder
    E(f64),
    /// sets the federate for all subsequent moved.
    F(f64),
    G(f64),
    /// Sets the laser power for the move
    S(f64),

    /// Axis set { U, V }
    U(f64),
    V(f64),

    W(f64),
    X(f64),
    Y(f64),
    Z(f64),
}

impl Eq for PosVal {}

/// Ignore numerical value.
impl PartialEq for PosVal {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::A(_), Self::A(_))
                | (Self::B(_), Self::B(_))
                | (Self::C(_), Self::C(_))
                | (Self::E(_), Self::E(_))
                | (Self::F(_), Self::F(_))
                | (Self::G(_), Self::G(_))
                | (Self::S(_), Self::S(_))
                | (Self::U(_), Self::U(_))
                | (Self::V(_), Self::V(_))
                | (Self::W(_), Self::W(_))
                | (Self::X(_), Self::X(_))
                | (Self::Y(_), Self::Y(_))
                | (Self::Z(_), Self::Z(_))
        )
    }
}
impl Hash for PosVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::A(_) => "A".hash(state),
            Self::B(_) => "B".hash(state),
            Self::C(_) => "C".hash(state),
            Self::E(_) => "E".hash(state),
            Self::F(_) => "F".hash(state),
            Self::G(_) => "G".hash(state),
            Self::S(_) => "S".hash(state),
            Self::U(_) => "U".hash(state),
            Self::V(_) => "V".hash(state),
            Self::W(_) => "W".hash(state),
            Self::X(_) => "X".hash(state),
            Self::Y(_) => "Y".hash(state),
            Self::Z(_) => "Z".hash(state),
        }
    }
}

// TODO: can I use a macro here!!
pub fn parse_a(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("A"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::A cannot decode number.");
        PosVal::A(number)
    })(i)
}

pub fn parse_b(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("B"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::B cannot decode number.");
        PosVal::B(number)
    })(i)
}

pub fn parse_c(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("C"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::C cannot decode number.");
        PosVal::C(number)
    })(i)
}

pub fn parse_e(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("E"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::E cannot decode number.");
        PosVal::E(number)
    })(i)
}

pub fn parse_f(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("F"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::F cannot decode number.");
        PosVal::F(number)
    })(i)
}

pub fn parse_s(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("S"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::S cannot decode number.");
        PosVal::S(number)
    })(i)
}
pub fn parse_u(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("U"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::U cannot decode number.");
        PosVal::U(number)
    })(i)
}
pub fn parse_v(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("V"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::V cannot decode number.");
        PosVal::V(number)
    })(i)
}

pub fn parse_w(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("W"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::W cannot decode number.");
        PosVal::W(number)
    })(i)
}
pub fn parse_x(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("X"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::X cannot decode number.");
        PosVal::X(number)
    })(i)
}
pub fn parse_y(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("Y"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::Y cannot decode number.");
        PosVal::Y(number)
    })(i)
}
pub fn parse_z(i: &str) -> IResult<&str, PosVal> {
    map(preceded(tag("Z"), digit1), |v: &str| {
        let number = v.parse::<f64>().expect("PosVal::Z cannot decode number.");
        PosVal::Z(number)
    })(i)
}
