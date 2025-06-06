use core::hash::Hash;
use core::hash::Hasher;
use std::collections::HashSet;

use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// G2/G3 Arc Command
///
/// Payload has two forms,
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Form {
    /// Arc with center offset in I and J
    IJ(HashSet<ArcVal>),
    /// Arc with radius R
    R(HashSet<ArcVal>),
}

/// Parameters for `Command::G2` and `Command::G3`
///
/// Similar to `PosVal` but with additional parameters
/// I,J, P, R.
#[derive(Clone, Debug)]
pub enum ArcVal {
    /// Axis A
    A(f64),
    /// Axis B
    B(f64),
    /// Axis C
    C(f64),

    /// Extruder
    E(f64),
    /// sets the federate for all subsequent moved.
    F(f64),

    /// Sets the laser power for the move
    S(f64),

    /// Arc center offset in the I direction
    I(f64),
    /// Arc center offset in the J direction
    J(f64),

    /// `P<Count>` number of complete circles.
    P(f64),
    /// Radius of the arc
    R(f64),

    /// Axis U
    U(f64),
    /// Axis V
    V(f64),

    /// Axis X
    X(f64),
    /// Axis Y
    Y(f64),
    /// Axis Z
    Z(f64),
    /// Axis W
    W(f64),
}

impl Eq for ArcVal {}

/// Bit wise comparison cant' compare directly [NAN and inf]
///
/// N.B. Equality is not used in production code -  assertion testing only.
impl PartialEq for ArcVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::A(x), Self::A(y))
            | (Self::B(x), Self::B(y))
            | (Self::C(x), Self::C(y))
            | (Self::E(x), Self::E(y))
            | (Self::F(x), Self::F(y))
            | (Self::I(x), Self::I(y))
            | (Self::J(x), Self::J(y))
            | (Self::P(x), Self::P(y))
            | (Self::R(x), Self::R(y))
            | (Self::S(x), Self::S(y))
            | (Self::U(x), Self::U(y))
            | (Self::V(x), Self::V(y))
            | (Self::W(x), Self::W(y))
            | (Self::X(x), Self::X(y))
            | (Self::Y(x), Self::Y(y))
            | (Self::Z(x), Self::Z(y)) => x.to_bits() == y.to_bits(),
            _ => false,
        }
    }
}

/// Hash is used to determine if an entry should be added to the Sets
///
/// malformed commands with duplicate parameters will be rejected here.
///
/// G1 X95.110 X96.233 E2.07708
///
/// By ignoring the f64 in hashing the parsed Command will only have one
/// X value.
impl Hash for ArcVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::A(_) => "A".hash(state),
            Self::B(_) => "B".hash(state),
            Self::C(_) => "C".hash(state),
            Self::E(_) => "E".hash(state),
            Self::F(_) => "F".hash(state),
            Self::I(_) => "I".hash(state),
            Self::J(_) => "J".hash(state),
            Self::P(_) => "P".hash(state),
            Self::R(_) => "R".hash(state),
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

// A macro to make parse_a generic
//
// BUGFIX: using double_no_exponent from double.rs
// as parsing a float with an exponent conflicts
// with the E parameter in GCode.
// use nom::number::complete::double;
macro_rules! parse_arc_val {
    ($name:ident, $tag:literal, $variant:ident) => {
        #[doc = "Extracts"]
        #[doc = stringify!($tag)]
        #[doc = " parameter"]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = "  When match fails."]
        pub fn $name(i: &str) -> IResult<&str, ArcVal> {
            map(
                preceded((space0, tag($tag)), crate::double::double_no_exponent),
                ArcVal::$variant,
            )
            .parse(i)
        }
    };
}

parse_arc_val!(parse_arc_a, "A", A);
parse_arc_val!(parse_arc_b, "B", B);
parse_arc_val!(parse_arc_c, "C", C);
parse_arc_val!(parse_arc_e, "E", E);

parse_arc_val!(parse_arc_i, "I", I);
parse_arc_val!(parse_arc_j, "J", J);
parse_arc_val!(parse_arc_p, "P", P);
parse_arc_val!(parse_arc_r, "R", R);

parse_arc_val!(parse_arc_f, "F", F);
parse_arc_val!(parse_arc_s, "S", S);
parse_arc_val!(parse_arc_u, "U", U);
parse_arc_val!(parse_arc_v, "V", V);

parse_arc_val!(parse_arc_w, "W", W);
parse_arc_val!(parse_arc_x, "X", X);
parse_arc_val!(parse_arc_y, "Y", Y);
parse_arc_val!(parse_arc_z, "Z", Z);

#[cfg(test)]
mod test {
    use super::*;

    // Test the macro
    #[test]
    fn parse_a_macro() {
        assert_eq!(parse_arc_a("A95.110"), Ok(("", ArcVal::A(95.110))));
    }

    // Offsets and extrusions can be negative
    #[test]
    fn parse_negative_value() {
        assert_eq!(parse_arc_e("E-1.1"), Ok(("", ArcVal::E(-1.1))));
        assert_eq!(parse_arc_i("I-10.10"), Ok(("", ArcVal::I(-10.10))));
        assert_eq!(parse_arc_j("J-100.100"), Ok(("", ArcVal::J(-100.100))));
    }

    #[test]
    fn pos_value_equality() {
        // Pass: - parameter wrapper and inner value match.
        assert!(ArcVal::A(95.0) == ArcVal::A(95.0));

        // Fail: -  A = A but inner value is different.
        assert!(ArcVal::A(95.0) != ArcVal::B(9.0));

        // FAIL: - A != B but with identical inner value.
        assert!(ArcVal::A(95.0) != ArcVal::B(95.0));
    }
}
