use core::fmt::Display;

use nom::combinator::map_res;
use nom::error::{Error, ErrorKind};
use nom::Parser;
use nom::{number::streaming::le_u16, IResult};

// Default Parameter encoding
//
// A empty placeholder, currently only the `GCodeBlock`
// uses an extended set of this encoding.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Param {
    pub encoding: Encoding,
}

pub(super) fn param_parser(input: &[u8]) -> IResult<&[u8], Param> {
    map_res(le_u16, |value| {
        Ok(match value {
            0u16 => Param {
                encoding: Encoding::None,
            },
            1u16 => Param {
                encoding: Encoding::MeatPackAlgorithm,
            },
            2u16 => Param {
                encoding: Encoding::MeatPackModifiedAlgorithm,
            },
            bad => {
                log::error!("Discarding bad encoding  {bad:?}");
                return Err(Error::new(input, ErrorKind::Alt));
            }
        })
    })
    .parse(input)
}

// Only GCodeblock does anything but None
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Encoding {
    // 0 = No encoding
    #[default]
    None,
    // 1 = MeatPack algorithm
    MeatPackAlgorithm,
    // 2 = MeatPack algorithm modified to keep comment lines
    MeatPackModifiedAlgorithm,
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

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => {
                writeln!(f, "0 - No encoding")
            }
            Self::MeatPackAlgorithm => {
                writeln!(f, "1 = MeatPack algorithm")
            }
            Self::MeatPackModifiedAlgorithm => {
                writeln!(f, "2 = MeatPack algorithm modified to keep comment lines")
            }
        }
    }
}
