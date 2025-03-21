//! The new binary G-code file consists of a file header followed by an ordered succession of blocks, in the following sequence:
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

/// Public: Contains the SVG converter.
pub mod gcode_block;

mod block_header;
mod compression_type;
mod default_params;
mod file_handler;
mod file_metadata_block;
mod print_metadata_block;
mod printer_metadata_block;
mod slicer_block;
mod thumbnail_block;

use core::fmt::Display;

use file_handler::{file_header_parser, FileHeader};
use file_metadata_block::{file_metadata_parser_with_checksum, FileMetadataBlock};
use nom::{
    combinator::{eof, map, opt},
    error::{ErrorKind, ParseError},
    multi::{many0, many_till},
    IResult, Parser,
};

use compression_type::CompressionType;
use gcode_block::{gcode_parser_with_checksum, GCodeBlock};
use print_metadata_block::{print_metadata_parser_with_checksum, PrintMetadataBlock};
use printer_metadata_block::printer_metadata_parser_with_checksum;
use printer_metadata_block::PrinterMetadataBlock;
use slicer_block::{slicer_parser_with_checksum, SlicerBlock};
use thumbnail_block::thumbnail_parser_with_checksum;
use thumbnail_block::ThumbnailBlock;

/// A trait for markdown formatting.
pub trait Markdown {
    /// Write to formatter a markdown block.
    ///
    /// # Errors
    ///   When a call to write fails.
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: core::fmt::Write;
}

/// Error while parsing text into a `Bgcode` structure.
#[derive(Debug)]
pub enum BlockError {
    /// Error parsing the checksum.
    Checksum(String),
    /// A failure to decompress the data block
    Decompression(String),
    /// Error parsing the file header.
    FileHeader(String),
    /// Traps when  EOF is called
    Kind(String),
    /// Error parsing the sub sections header.
    Header(String),
    /// Error parsing the parameter section.
    Param(String),
}

impl<I> ParseError<I> for BlockError
where
    I: std::fmt::Debug,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        // This is trapping an EOF error
        let message = format!("{kind:?}:\t{input:?}\n");
        Self::Kind(message)
    }

    // if combining multiple errors, we show them one after the other
    fn append(input: I, kind: ErrorKind, other: Self) -> Self {
        match other {
            Self::FileHeader(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::FileHeader(message)
            }
            Self::Checksum(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::Checksum(message)
            }
            Self::Decompression(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::Decompression(message)
            }

            Self::Header(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::Header(message)
            }
            Self::Param(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::Param(message)
            }
            Self::Kind(message) => {
                let message = format!("{message}{kind:?}:\t{input:?}\n");
                Self::Kind(message)
            }
        }
    }

    fn from_char(input: I, c: char) -> Self {
        let message = format!("'{c}':\t{input:?}\n",);
        println!("{message}");
        // big match statement append message to existing message
        todo!()
    }

    fn or(self, _other: Self) -> Self {
        // let message = format!("{}\tOR\n{}\n", self.message, other.message);
        // println!("{message}");
        // Self::Other(message)
        todo!()
    }
}

/// Structure of the binary file.
///
/// extension .bgcode
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bgcode<'a> {
    fh: FileHeader,
    /// A file block.
    pub file_metadata: Option<FileMetadataBlock<'a>>,
    /// A file block.
    pub printer_metadata: PrinterMetadataBlock<'a>,
    /// A collection of image blocks.
    pub thumbnails: Vec<ThumbnailBlock<'a>>,
    /// A file block.
    pub print_metadata: PrintMetadataBlock<'a>,
    /// A file block.
    pub slicer: SlicerBlock<'a>,
    /// A collection of gcode blocks.
    pub gcode: Vec<GCodeBlock<'a>>,
}

impl Display for Bgcode<'_> {
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

impl Markdown for Bgcode<'_> {
    /// Write to formatter a markdown block.
    ///
    /// # Errors
    ///   When match fails.
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        self.fh.markdown(&mut *f)?;

        if let Some(file_metadata) = &self.file_metadata {
            file_metadata.markdown(&mut *f)?;
        } else {
            writeln!(f, "No optional file metadata block")?;
        }

        self.printer_metadata.markdown(&mut *f)?;

        self.thumbnails.markdown(&mut *f)?;

        self.print_metadata.markdown(&mut *f)?;

        self.slicer.markdown(&mut *f)?;

        self.gcode.markdown(f)?;

        Ok(())
    }
}

/// Parses a binary gcode
///
/// # Errors
///   When the bytes stream is not a valid file.
pub fn bgcode_parser(input: &[u8]) -> IResult<&[u8], Bgcode, BlockError> {
    map(
        (
            file_header_parser,
            opt(file_metadata_parser_with_checksum),
            printer_metadata_parser_with_checksum,
            many0(thumbnail_parser_with_checksum),
            print_metadata_parser_with_checksum,
            slicer_parser_with_checksum,
            // eof here asserts than what remains is_empty()
            many_till(gcode_parser_with_checksum, eof),
        ),
        |(
            fh,
            file_metadata,
            printer_metadata,
            thumbnail,
            print_metadata,
            slicer,
            (gcode, _remain),
        )| {
            log::info!("File has been validated");
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
    )
    .parse(input)
}
