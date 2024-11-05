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
mod fm;
mod gcode;
mod pm;
mod sm;
mod thumb;

use fm::FileMetadataBlock;
use gcode::GCodeBlock;
use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::be_u16,
    Err, IResult,
};
use pm::PrinterMetadataBlock;
use sm::SlicerMetadataBlock;
use thumb::ThumbnailBlock;

// Structure of the binary file.
//
// extension .bgcode
struct Bgcode {
    fh: FileHeader,
    file_metadata: Option<FileMetadataBlock>,
    printer_metadata: PrinterMetadataBlock,
    thumbnail: Option<Vec<ThumbnailBlock>>,
    print: PrinterMetadataBlock,
    slicer: SlicerMetadataBlock,
    gcode: Vec<GCodeBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileHeader {
    header: BlockHeader,
}

/// Block header
///
/// The block header is the same for all blocks. It is defined as:
///
/// type                 size  description
/// Type                 u16   2 bytes Block type
/// Compression          u16   2 bytes Compression algorithm
/// Uncompressed size    u32   4 bytes Size of the data when uncompressed
/// Compressed size      u32   4 bytes Size of the data when compressed
///
/// <https://github.com/prusa3d/libbgcode/blob/main/doc/specifications.md#block-header>
///
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlockHeader {
    block_type: u16,
    // Compression algorithm
    compression: u16,
    // Size of data when uncompressed
    uncompressed_size: u32,
    // Size of data whehn compressed
    compressed_size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BlockType {
    FileMetadata = 0,
    GCode = 1,
    SlicerMetadata = 2,
    PrinterMetadata = 3,
    PrintMetadata = 4,
    Thumbnail = 5,
}

fn block_type_parse(input: &[u8]) -> IResult<&[u8], BlockType> {
    map_res(be_u16, |block_type: u16| {
        // help
        Ok(match block_type {
            0 => BlockType::FileMetadata,
            1 => BlockType::GCode,
            2 => BlockType::SlicerMetadata,
            3 => BlockType::PrinterMetadata,
            4 => BlockType::PrintMetadata,
            5 => BlockType::Thumbnail,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Compression {
    NoCompression = 0,
    Deflate = 1,
    // Heatshrink algorithm with window size 11 and lookahead size 4
    HeatShrink11 = 2,
    // Heatshrink algorithm with window size 12 and lookahead size 4
    HeatShrink12 = 3,
}

fn compression_parse(input: &[u8]) -> IResult<&[u8], Compression> {
    map_res(be_u16, |compression: u16| {
        // help
        Ok(match compression {
            0 => Compression::NoCompression,
            1 => Compression::Deflate,
            2 => Compression::HeatShrink11,
            3 => Compression::HeatShrink12,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}
