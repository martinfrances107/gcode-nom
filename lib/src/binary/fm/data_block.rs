use core::fmt::Display;

use nom::{combinator::map, IResult};

use super::param::parameters_parser;
use super::param::Parameter;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct DataBlock(Parameter);

impl Display for DataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Block")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.0)?;

        Ok(())
    }
}

pub(super) fn data_parser(input: &[u8]) -> IResult<&[u8], DataBlock> {
    map(parameters_parser, DataBlock)(input)
}
