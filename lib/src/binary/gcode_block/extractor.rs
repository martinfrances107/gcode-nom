use nom::IResult;
use nom::Parser;
use nom::combinator::eof;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many_till;
use nom::multi::many0;
use nom::sequence::preceded;

use crate::binary::BlockError;
use crate::binary::file_handler::file_header_parser;
use crate::binary::file_metadata_block::file_metadata_parser;
use crate::binary::print_metadata_block::print_metadata_parser;
use crate::binary::printer_metadata_block::printer_metadata_parser;
use crate::binary::slicer_block::slicer_parser;
use crate::binary::thumbnail_block::thumbnail_parser;

use super::GCodeBlock;
use super::gcode_parser;

/// Extracts gcode block from a binary gcode file.
///
/// # Errors
///
/// When the parsing fails.
///
pub fn extract_gcode(input: &[u8]) -> IResult<&[u8], Vec<GCodeBlock<'_>>, BlockError> {
    map(
        preceded(
            (
                file_header_parser,
                opt(file_metadata_parser),
                printer_metadata_parser,
                many0(thumbnail_parser),
                print_metadata_parser,
                slicer_parser,
            ),
            // eof here asserts than what remains is_empty()
            many_till(gcode_parser, eof),
        ),
        |(gcode, _remain)| gcode,
    )
    .parse(input)
}
