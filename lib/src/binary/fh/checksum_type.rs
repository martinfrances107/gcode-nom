use core::fmt::Display;

use nom::error::ErrorKind;
use nom::number::streaming::le_u16;
use nom::Err;
use nom::{combinator::map_res, error::Error, IResult};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum ChecksumType {
    #[default]
    None = 0,
    CRC32 = 1,
}

impl Display for ChecksumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => {
                write!(f, "0 - No Checksum")
            }
            Self::CRC32 => {
                write!(f, "1 - CRC32")
            }
        }
    }
}

pub(super) fn checksum_parse(input: &[u8]) -> IResult<&[u8], ChecksumType> {
    map_res(le_u16, |value| {
        Ok(match value {
            0 => ChecksumType::None,
            1 => ChecksumType::CRC32,
            bad_checksum => {
                println!("Discarding checksum {bad_checksum:?}");
                return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
            }
        })
    })(input)
}
