use core::hash::Hash;

use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::complete;
use nom::combinator::map;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// Parameters used in M486 Commands
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum MultiPartVal {
    /// A Integrity check
    /// used to assert this name is equal to be equivalent embedded stl file name.
    A(String),
    /// C   ; Cancel the current object (use with care!)
    C,
    /// P10 ; Cancel object with index 10 (the 11th object)
    P(i128),
    /// M486 S3                ; Indicate that the 4th object is starting now
    /// M486 S3 A"cube copy 3" ; Indicate that the 4th object is starting now and name it
    /// S-1 ; Indicate a non-object, purge tower, or other global feature
    S(i128, Option<String>),
    /// T12 ; Total of 12 objects (otherwise the firmware must count)
    T(i128),
    /// U2  ; Un-cancel object with index 2 (the 3rd object)
    U(i128),
}

macro_rules! parse_mp_val {
    ($name:ident, $tag:literal, $variant:ident) => {
        #[doc = "Extracts multipart"]
        #[doc = stringify!($tag)]
        #[doc = " parameter"]
        #[doc = ""]
        #[doc = "# Errors"]
        #[doc = "  When match fails."]
        pub fn $name(i: &str) -> IResult<&str, MultiPartVal> {
            map(
                preceded((space0, tag($tag)), nom::character::complete::i128),
                MultiPartVal::$variant,
            )
            .parse(i)
        }
    };
}

/// Extract multipart A parameter
///
/// TODO: Should I used `line_ending` instead of newline?
///
/// # Errors
///   when match fails.
pub fn parse_mp_a(i: &str) -> IResult<&str, MultiPartVal> {
    map(
        complete(delimited(tag("A"), take_until("\n"), char('\n'))),
        |s: &str| MultiPartVal::A(s.to_string()),
    )
    .parse(i)
}

/// Extract multipart C parameter
///
/// # Errors
///   when match fails.
pub fn parse_mp_c(i: &str) -> IResult<&str, MultiPartVal> {
    map((space0, tag("C")), |_| MultiPartVal::C).parse(i)
}

/// Extract multipart S parameter
///
/// # Errors
///   when match fails.
pub fn parse_mp_s(i: &str) -> IResult<&str, MultiPartVal> {
    map(
        preceded(
            (space0::<&str, _>, tag("S")),
            (
                // This is broken when a name is supplied, it is not recognized!!!
                nom::character::complete::i128,
                nom::combinator::opt(preceded(
                    space0,
                    delimited(char('"'), take_until("\""), char('"')),
                )),
            ),
        ),
        |(s, name)| MultiPartVal::S(s, name.map(std::string::ToString::to_string)),
    )
    .parse(i)
}

parse_mp_val!(parse_mp_p, "P", P);
// TODO must implement [ S<i> A"<name>" ]
// M486 S3 A"cube copy 3" ; Indicate that the 4th object is starting now and name it

parse_mp_val!(parse_mp_t, "T", T);
parse_mp_val!(parse_mp_u, "U", U);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multipart_value_equality() {
        // Pass: - parameter wrapper and inner value match.
        assert_eq!(
            parse_mp_a("Aa.stl\n"),
            Ok(("", MultiPartVal::A(String::from("a.stl"))))
        );

        // Pass: - parameter wrapper and inner value match.
        assert_eq!(parse_mp_c("C"), Ok(("", MultiPartVal::C)));

        // Pass: - parameter wrapper and inner value match.
        assert_eq!(parse_mp_s("S-1"), Ok(("", MultiPartVal::S(-1, None))));
    }
}
