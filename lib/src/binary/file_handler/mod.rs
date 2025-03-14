use core::fmt::Display;

use nom::combinator::map;

use nom::sequence::preceded;

use nom::Parser;
use nom::{sequence::pair, IResult};

mod checksum_type;
mod preamble;
mod version;

use checksum_type::{checksum_type_parser, ChecksumType};
use preamble::preamble;
use version::{version_parser, Version};

use super::Markdown;

//  Current value for Version is 1
//
// Possible values for Checksum type are:
//
// 0 = None
// 1 = CRC32
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileHeader {
    version: Version,
    checksum_type: ChecksumType,
}

impl Display for FileHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "File Header")?;
        writeln!(f)?;
        writeln!(f, "{}", self.version)?;
        writeln!(f, "{}", self.checksum_type)?;

        Ok(())
    }
}

impl Markdown for FileHeader {
    /// Write to formatter a markdown block.
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        writeln!(f, "## File Header")?;
        writeln!(f)?;
        writeln!(f, "{}", self.version)?;
        writeln!(f, "{}", self.checksum_type)?;

        Ok(())
    }
}

pub fn file_header_parser(input: &[u8]) -> IResult<&[u8], FileHeader> {
    preceded(
        preamble,
        map(
            pair(version_parser, checksum_type_parser),
            |(version, checksum_type)| FileHeader {
                version,
                checksum_type,
            },
        ),
    )
    .parse(input)
}
