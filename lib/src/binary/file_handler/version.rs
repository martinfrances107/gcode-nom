use core::fmt::Display;

use nom::error::ErrorKind;
use nom::number::streaming::le_u32;

use nom::Parser;
use nom::{combinator::map_res, error::Error, IResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Version(pub(super) u16);

impl Default for Version {
    fn default() -> Self {
        Self(1u16)
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Version number {}", self.0)
    }
}

pub(super) fn version_parser(input: &[u8]) -> IResult<&[u8], Version> {
    map_res(le_u32, |value| {
        Ok(match value {
            1 => Version(1),
            bad_version => {
                log::error!("Discarding file handler version type {bad_version:?}");
                return Err(Error::new(input, ErrorKind::Alt));
            }
        })
    })
    .parse(input)
}
