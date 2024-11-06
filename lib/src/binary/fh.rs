use core::fmt::Display;

use nom::error::ErrorKind;
use nom::number::streaming::{le_u16, le_u32};
use nom::sequence::preceded;
use nom::Err;
use nom::{combinator::map_res, error::Error, sequence::pair, IResult};

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File Header")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.version)?;
        writeln!(f, "{}", self.checksum_type)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum ChecksumType {
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Version(u16);

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

// First 32 bits of valid bgcode file.
static HEADER: u32 = 0x4544_4347;

// Shorthand to catch the preamble
fn found_magic(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(le_u32, |code| {
        if code == HEADER {
            println!("found binary GCODE header 0x{code:X}");
            Ok(HEADER)
        } else {
            Err(Err::Error(Error::new(input, ErrorKind::Alt)))
        }
    })(input)
}

pub fn file_header_parse(input: &[u8]) -> IResult<&[u8], FileHeader> {
    preceded(
        found_magic,
        map_res(
            pair(le_u32, le_u16),
            |(version_value, checksum_value): (u32, u16)| {
                let checksum_type = match checksum_value {
                    0 => ChecksumType::None,
                    1 => ChecksumType::CRC32,
                    bad_checksum => {
                        println!("Discarding checksum {bad_checksum:?}");
                        return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
                    }
                };
                // Discard unrecognized version.
                let version = match version_value {
                    1 => Version(1),
                    bad_version => {
                        println!("discarding version {bad_version:?}");
                        return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
                    }
                };

                Ok(FileHeader {
                    version,
                    checksum_type,
                })
            },
        ),
    )(input)
}
