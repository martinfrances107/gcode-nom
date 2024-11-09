use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::le_u16,
    Err, IResult,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum CompressionType {
    #[default]
    None = 0,
    Deflate = 1,
    // Heatshrink algorithm with window size 11 and lookahead size 4
    HeatShrink11 = 2,
    // Heatshrink algorithm with window size 12 and lookahead size 4
    HeatShrink12 = 3,
}

pub(super) fn compression_parser(input: &[u8]) -> IResult<&[u8], CompressionType> {
    map_res(le_u16, |compression: u16| {
        Ok(match compression {
            0 => CompressionType::None,
            1 => CompressionType::Deflate,
            2 => CompressionType::HeatShrink11,
            3 => CompressionType::HeatShrink12,
            _ => {
                return {
                    println!("Compression_parser bad type  failing ");
                    Err(Err::Error(Error::new(input, ErrorKind::Alt)))
                }
            }
        })
    })(input)
}
