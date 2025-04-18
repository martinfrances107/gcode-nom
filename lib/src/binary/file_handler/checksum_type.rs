use core::fmt::Display;

use nom::error::ErrorKind;
use nom::number::streaming::le_u16;
use nom::Parser;
use nom::{combinator::map_res, error::Error, IResult};

// Details if a checksum is appended to all blocks structures.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum ChecksumType {
    #[default]
    None = 0,
    CRC32 = 1,
}

impl Display for ChecksumType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::None => {
                write!(f, "0 - Blocks have no checksum")
            }
            Self::CRC32 => {
                write!(f, "1 - A CRC32 checksum is appended to all block")
            }
        }
    }
}

pub(super) fn checksum_type_parser(input: &[u8]) -> IResult<&[u8], ChecksumType> {
    map_res(le_u16, |value| {
        Ok(match value {
            0 => ChecksumType::None,
            1 => ChecksumType::CRC32,
            bad_checksum => {
                log::error!("Discarding bad checksum type {bad_checksum:?}");
                return Err(Error::new(input, ErrorKind::Alt));
            }
        })
    })
    .parse(input)
}
