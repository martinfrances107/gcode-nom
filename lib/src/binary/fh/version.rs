use core::fmt::Display;

use nom::error::ErrorKind;
use nom::number::streaming::le_u32;

use nom::Err;
use nom::{combinator::map_res, error::Error, IResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Version(pub(super) u16);

impl Default for Version {
    fn default() -> Self {
        Self(1u16)
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Version number {}", self.0)
    }
}

pub(super) fn version_parse(input: &[u8]) -> IResult<&[u8], Version> {
    map_res(le_u32, |value| {
        Ok(match value {
            1 => Version(1),
            bad_version => {
                println!("Discarding version {bad_version:?}");
                return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
            }
        })
    })(input)
}