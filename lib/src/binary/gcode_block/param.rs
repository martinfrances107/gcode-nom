use core::fmt::Display;

use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::le_u16,
    Err, IResult,
};

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
pub(super) fn param_parser(input: &[u8]) -> IResult<&[u8], Encoding> {
    map_res(le_u16, |value| {
        Ok(match value {
            0u16 => Encoding::None,
            1u16 => Encoding::MeatPackAlgorithm,
            2u16 => Encoding::MeatPackModifiedAlgorithm,
            bad => {
                log::error!("Discarding bad encoding  {bad:?}");
                return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
            }
        })
    })(input)
}
