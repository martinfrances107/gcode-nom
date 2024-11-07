use nom::{
    combinator::{map, map_res},
    number::streaming::le_u16,
    IResult,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Param {
    // possible values :-
    // 1
    encoding: u16,
}

pub(super) fn param_parser(input: &[u8]) -> IResult<&[u8], Param> {
    map(le_u16, |(encoding)| Param { encoding })(input)
}
