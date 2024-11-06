use core::fmt::Display;

use nom::{
    error::{Error, ErrorKind},
    Err, IResult,
};

use super::param::parameters_parse;
use super::param::Parameter;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DataBlock(Parameter);

pub(crate) fn data_parse(input: &[u8]) -> IResult<&[u8], DataBlock> {
    match parameters_parse(input) {
        Ok((r, parameter)) => Ok((r, DataBlock(parameter))),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

impl Display for DataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Block")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.0)?;

        Ok(())
    }
}
