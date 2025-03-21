use core::fmt::Display;

use nom::combinator::map_res;
use nom::error::{Error, ErrorKind};
use nom::Parser;
use nom::{number::streaming::le_u16, IResult};

// Default Parameter encoding
//
// A empty placeholder, currently only the `GCodeBlock`
// uses an extended set of this encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Param {
    pub(super) encoding: Encoding,
}

pub(super) fn param_parser(input: &[u8]) -> IResult<&[u8], Param> {
    map_res(le_u16, |value| {
        Encoding::try_from(value).map_or_else(
            |_| Err(Error::new(value, ErrorKind::Alt)),
            |encoding| Ok(Param { encoding }),
        )
    })
    .parse(input)
}

// Encoding
//
// 0 = No encoding
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Encoding {
    None,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Encoding ini")
    }
}
impl TryFrom<u16> for Encoding {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0u16 => Self::None,
            bad_value => {
                let msg = format!("Discarding version {bad_value:?}");
                log::error!("{}", &msg);
                return Err(msg);
            }
        })
    }
}
