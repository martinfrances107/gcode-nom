use core::fmt::Display;

use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::le_u16,
    IResult, Parser,
};

use crate::binary::default_params::Encoding;
