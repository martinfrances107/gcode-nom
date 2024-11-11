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
mod default_params;
mod file_handler;
mod file_metadata_block;
mod gcode_block;
mod print_metadata_block;
mod printer_metadata_block;
mod slicer_block;
mod thumbnail_block;

use core::fmt::Display;

use file_handler::{file_header_parser, FileHeader};
use file_metadata_block::{file_metadata_parser_with_checksum, FileMetadataBlock};
use nom::{
    combinator::{eof, map, opt},
    multi::{many0, many_till},
    sequence::tuple,
    IResult,
};

use compression_type::CompressionType;
use gcode_block::{gcode_parser_with_checksum, GCodeBlock};
use print_metadata_block::{print_metadata_parser_with_checksum, PrintMetadataBlock};
use printer_metadata_block::printer_metadata_parser_with_checksum;
use printer_metadata_block::PrinterMetadataBlock;
use slicer_block::{slicer_parser_with_checksum, SlicerBlock};
use thumbnail_block::thumbnail_parser_with_checksum;
use thumbnail_block::ThumbnailBlock;

/// Structure of the binary file.
///
/// extension .bgcode
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bgcode {
    fh: FileHeader,
    /// A file block.
    pub file_metadata: Option<FileMetadataBlock>,
    /// A file block.
    pub printer_metadata: PrinterMetadataBlock,
    /// A colleciton of image blocks.
    pub thumbnails: Vec<ThumbnailBlock>,
    /// A file block.
    pub print_metadata: PrintMetadataBlock,
    /// A file block.
    pub slicer: SlicerBlock,
    /// A collection of gcode blocks.
    pub gcode: Vec<GCodeBlock>,
}

impl Display for Bgcode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", self.fh)?;
        if let Some(file_metadata) = &self.file_metadata {
            writeln!(f, "{file_metadata}")?;
        } else {
            writeln!(f, "No optional file metadata block")?;
        }
        writeln!(f, "{}", &self.printer_metadata)?;
        if self.thumbnails.is_empty() {
            writeln!(f, "No optional thumbnail block")?;
        } else {
            for thumb in &self.thumbnails {
                writeln!(f, "{thumb}")?;
            }
        }

        writeln!(f, "{}", self.print_metadata)?;
        writeln!(f, "{}", self.slicer)?;
        if self.gcode.is_empty() {
            writeln!(f, "No optional thumbnail block")?;
        } else {
            for g in &self.gcode {
                writeln!(f, "{g}")?;
            }
        }
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
            many0(thumbnail_parser_with_checksum),
            print_metadata_parser_with_checksum,
            slicer_parser_with_checksum,
            // eof here asserts than what remains is_empty()
            many_till(gcode_parser_with_checksum, eof),
        )),
        |(
            fh,
            file_metadata,
            printer_metadata,
            thumbnail,
            print_metadata,
            slicer,
            (gcode, _remain),
        )| {
            println!("File has been validated");
            Bgcode {
                fh,
                file_metadata,
                printer_metadata,
                thumbnails: thumbnail,
                print_metadata,
                slicer,
                gcode,
            }
        },
    )(input)
}
