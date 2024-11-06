#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Parameter {
    Encoding(EncodingVal),
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encoding(encoding_val) => {
                writeln!(f, "Paramter: Encoding {encoding_val}")?;
            }
        }

        Ok(())
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Self::Encoding(EncodingVal::Ini)
    }
}
pub(crate) fn parameters_parse(input: &[u8]) -> IResult<&[u8], Parameter> {
    match encoding_parse(input) {
        Ok((r, encoding_value)) => Ok((r, Parameter::Encoding(encoding_value))),
        Err(_) => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum EncodingVal {
    #[default]
    Ini = 0,
}

impl Display for EncodingVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Self::Ini => {
                writeln!(f, "Ini")?;
            }
        }

        Ok(())
    }
}
use core::fmt::Display;

use nom::combinator::map_res;
use nom::error::{Error, ErrorKind};
use nom::number::streaming::le_u16;
use nom::{Err, IResult};

fn encoding_parse(input: &[u8]) -> IResult<&[u8], EncodingVal> {
    map_res(le_u16, |encoding_val| {
        // header
        match encoding_val {
            1 => Ok(EncodingVal::Ini),
            _bad_encoding => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        }
    })(input)
}
