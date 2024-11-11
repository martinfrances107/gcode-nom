use core::fmt::Display;

use nom::Err;
use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::le_u16,
    sequence::tuple,
    IResult,
};

// type    size    description
// Format	 uint16	 Image format
// Width	 uint16	 Image width
// Height	 uint16	 Image height

// Possible values for Format are:
//
// 0 = PNG format
// 1 = JPG format
// 2 = QOI format
//
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    // aka file extension
    pub format: Format,
    pub width: u16,
    pub height: u16,
}

impl Display for Param {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "format {}", self.format)?;
        writeln!(f, "width {}", self.width)?;
        writeln!(f, "height {}", self.height)
    }
}

pub(super) fn param_parser(input: &[u8]) -> IResult<&[u8], Param> {
    map_res(tuple((le_u16, le_u16, le_u16)), |(f, width, height)| {
        Format::try_from(f).map_or_else(
            |_| Err(Err::Error(Error::new(input, ErrorKind::Alt))),
            |format| {
                Ok(Param {
                    format,
                    width,
                    height,
                })
            },
        )
    })(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Format {
    Png,
    Jpg,
    Qoi,
}

impl TryFrom<u16> for Format {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Png),
            1 => Ok(Self::Jpg),
            2 => Ok(Self::Qoi),
            bad_value => {
                let msg = format!("Format Value was not recognised Ox{bad_value:02X}");
                Err(msg)
            }
        }
    }
}
impl Display for Format {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Png => write!(f, "png"),
            Self::Jpg => write!(f, "jpg"),
            Self::Qoi => write!(f, "oci"),
        }
    }
}
