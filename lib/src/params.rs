use core::hash::Hash;
use core::hash::Hasher;

use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

// BUGFIX: using double_no_exponent from double.rs
// as parsing a float with an exponent conflicts
// with the E parameter in GCode.
// use nom::number::complete::double;
use crate::double::double_no_exponent as double;

/// Parameters for `Command::G0` and `Command::G1`
#[derive(Clone, Debug)]
pub enum PosVal {
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

impl Eq for PosVal {}

/// Bit wise comparison cant' compare directly [NAN and inf]
///
/// N.B. Equality is not used in production code -  assertion testing only.
impl PartialEq for PosVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::A(x), Self::A(y))
            | (Self::B(x), Self::B(y))
            | (Self::C(x), Self::C(y))
            | (Self::E(x), Self::E(y))
            | (Self::F(x), Self::F(y))
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
impl Hash for PosVal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::A(_) => "A".hash(state),
            Self::B(_) => "B".hash(state),
            Self::C(_) => "C".hash(state),
            Self::E(_) => "E".hash(state),
            Self::F(_) => "F".hash(state),
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

// A macro to make a parse_X() function where X is a parameter in a G0/G1 command
//
// BUGFIX: using double_no_exponent from double.rs
// as parsing a float where an exponent conflicts
// with the E parameter in GCode.
// a replacement for nom::number::complete::double;
macro_rules! parse_val {
    ($name:ident, $tag:literal, $variant:ident) => {
        /// Extracts $tag parameter
        ///
        /// # Errors
        ///   When match fails.
        pub fn $name(i: &str) -> IResult<&str, PosVal> {
            map(
                preceded((space0, tag($tag)), crate::double::double_no_exponent),
                PosVal::$variant,
            )
            .parse(i)
        }
    };
}

parse_val!(parse_a, "A", A);
parse_val!(parse_b, "B", B);
parse_val!(parse_c, "C", C);
// /// Extracts A parameter
parse_val!(parse_e, "E", E);
parse_val!(parse_f, "F", F);
parse_val!(parse_s, "S", S);
// ///
parse_val!(parse_u, "U", U);
parse_val!(parse_v, "V", V);
parse_val!(parse_w, "W", W);
// /// # Errorsparse_val!(parse_e, "E", E);
parse_val!(parse_x, "X", X);
parse_val!(parse_y, "Y", Y);
parse_val!(parse_z, "Z", Z);
// ///   When match fails.parse_val!(parse_s, "S", S);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pos_value_equality() {
        // Pass: - parameter wrapper and inner value match.
        assert!(PosVal::A(95.0) == PosVal::A(95.0));

        // Fail: -  A = A but inner value is different.
        assert!(PosVal::A(95.0) != PosVal::B(9.0));

        // FAIL: - A != B but with identical inner value.
        assert!(PosVal::A(95.0) != PosVal::B(95.0));
    }
}
