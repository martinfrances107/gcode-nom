use core::fmt::Display;

use nom::combinator::map_res;
use nom::error::{Error, ErrorKind};
use nom::Parser;
use nom::{number::streaming::le_u16, IResult};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Param {
    // possible values :-
    // 0
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
// 1 = MeatPack algorithm
// 2 = MeatPack algorithm modified to keep comment lines
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Encoding {
    None,
    // MeatPack,
    // MeatPackWithComments,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Encoding ini")
    }
}
impl TryFrom<u16> for Encoding {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0u16 => Self::None,
            // 1u16 => Encoding::MeatPack,
            // 2u16 => Encoding::MeatPackWithComments,
            bad_value => {
                log::error!("Discarding version {bad_value:?}");
                return Err("Invalid encoding type could not decode {bad_value:?}");
            }
        })
    }
}
