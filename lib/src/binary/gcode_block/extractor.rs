use nom::{
    combinator::{eof, map, opt},
    multi::{many0, many_till},
    sequence::preceded,
    IResult, Parser,
};

use crate::binary::{
    file_handler::file_header_parser, file_metadata_block::file_metadata_parser,
    print_metadata_block::print_metadata_parser, printer_metadata_block::printer_metadata_parser,
    slicer_block::slicer_parser, thumbnail_block::thumbnail_parser, BlockError,
};

use super::{gcode_parser, GCodeBlock};

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
