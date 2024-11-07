//! The new binarized G-code file consists of a file header followed by an ordered succession of blocks, in the following sequence:
//!
//! File Header
//! File Metadata Block (optional)
//! Printer Metadata Block
//! Thumbnails Blocks (optional)
//! Print Metadata Block
//! Slicer Metadata Block
//! G-code Blocks
//!
//! All of the multi-byte integers are encoded in little-endian byte ordering.
//!
//! <https://github.com/prusa3d/libbgcode/blob/main/doc/specifications.md>
//!
//! using this file as a example of good binary parsing
//!
//! <https://github.com/rust-av/flavors/blob/master/src/parser.rs>
//!
mod block_header;
mod compression_type;
mod file_handler;
mod file_metadata_block;
mod gcode;
mod printer_metadata_block;
mod sm;
mod thumb;

use std::fmt::Display;

use file_handler::{file_header_parser, FileHeader};
use file_metadata_block::{file_metadata_parser_with_checksum, FileMetadataBlock};
use nom::{
    combinator::{map, opt},
    sequence::tuple,
    IResult,
};

use compression_type::CompressionType;
use printer_metadata_block::printer_metadata_parser_with_checksum;
use printer_metadata_block::PrinterMetadataBlock;

/// Structure of the binary file.
///
/// extension .bgcode
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bgcode {
    fh: FileHeader,
    file_metadata: Option<FileMetadataBlock>,
    printer_metadata: PrinterMetadataBlock,
    // thumbnail: Option<Vec<ThumbnailBlock>>,
    // print: PrinterMetadataBlock,
    // slicer: SlicerMetadataBlock,
    // gcode: Vec<GCodeBlock>,
}

impl Display for Bgcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.fh)?;
        if let Some(file_metadata) = &self.file_metadata {
            writeln!(f, "{}", file_metadata)?;
        } else {
            writeln!(f, "No optional file metadata block")?;
        }
        writeln!(f, "{}", &self.printer_metadata)?;
        // TODO add more sections
        Ok(())
    }
}

/// Parses a binary gcode
///
/// # Errors
///   When the bytes stream is not a valid file.
pub fn bgcode_parser(input: &[u8]) -> IResult<&[u8], Bgcode> {
    map(
        tuple((
            file_header_parser,
            opt(file_metadata_parser_with_checksum),
            printer_metadata_parser_with_checksum,
        )),
        |(fh, file_metadata, printer_metadata)| Bgcode {
            fh,
            file_metadata,
            printer_metadata,
        },
    )(input)
}
