use nom::error::ErrorKind;
use nom::number::streaming::le_u32;

use nom::Err;
use nom::{combinator::map_res, error::Error, IResult};

// First 32 bits of valid bgcode file.
static HEADER: u32 = 0x4544_4347;

// Shorthand to catch the file preamble
pub(super) fn preamble(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(le_u32, |code| {
        if code == HEADER {
            println!("found binary GCODE header 0x{code:X}");
            Ok(HEADER)
        } else {
            Err(Err::Error(Error::new(input, ErrorKind::Alt)))
        }
    })(input)
}
