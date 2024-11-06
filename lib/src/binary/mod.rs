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
mod fh;
mod fm;
mod gcode;
mod pm;
mod sm;
mod thumb;

use std::fmt::Display;

use fh::{file_header_parse, FileHeader};
use fm::{file_metadata_parse, FileMetadataBlock};
use nom::{
    combinator::map_res,
    error::{Error, ErrorKind},
    number::streaming::{le_u16, le_u32},
    sequence::tuple,
    Err, IResult,
};

/// Structure of the binary file.
///
/// extension .bgcode
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bgcode {
    fh: FileHeader,
    file_metadata: Option<FileMetadataBlock>,
    // printer_metadata: PrinterMetadataBlock,
    // thumbnail: Option<Vec<ThumbnailBlock>>,
    // print: PrinterMetadataBlock,
    // slicer: SlicerMetadataBlock,
    // gcode: Vec<GCodeBlock>,
}

impl Display for Bgcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.fh)?;
        writeln!(f, "{:?}", self.file_metadata)?;

        // TODO add more sections
        Ok(())
    }
}

/// Parses a binary gcode
///
/// # Errors
///   When the bytes stream is not a valid file.
pub fn bgcode_parse(input: &[u8]) -> IResult<&[u8], Bgcode> {
    let (mut remain, fh) = file_header_parse(input)?;
    let file_metadata = if let Ok((r, file_meta_data_actual)) = file_metadata_parse(remain) {
        remain = r;
        Some(file_meta_data_actual)
    } else {
        None
    };

    Ok((
        remain,
        Bgcode {
            fh,
            file_metadata,
            // printer_metadata: todo!(),
            // thumbnail: todo!(),
            // print: todo!(),
            // slicer: todo!(),
            // gcode: todo!(),
        },
    ))
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BlockHeader {
    // block_type: u16,
    // Compression algorithm
    compression: Compression,
    // Size of data when uncompressed
    uncompressed_size: u32,
    // Size of data when compressed
    compressed_size: u32,
}

pub(crate) fn block_header_parse(input: &[u8]) -> IResult<&[u8], BlockHeader> {
    match tuple((compression_parse, le_u32, le_u32))(input) {
        Ok((remain, (compression, uncompressed_size, compressed_size))) => Ok((
            remain,
            BlockHeader {
                compression,
                uncompressed_size,
                compressed_size,
            },
        )),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

// #[derive(Clone, Debug, PartialEq, Eq)]
// enum BlockType {
//     FileMetadata = 0,
//     GCode = 1,
//     SlicerMetadata = 2,
//     PrinterMetadata = 3,
//     PrintMetadata = 4,
//     Thumbnail = 5,
// }

// fn block_type_parse(input: &[u8]) -> IResult<&[u8], BlockType> {
//     map_res(le_u16, |block_type: u16| {
//         // help
//         Ok(match block_type {
//             0 => BlockType::FileMetadata,
//             1 => BlockType::GCode,
//             2 => BlockType::SlicerMetadata,
//             3 => BlockType::PrinterMetadata,
//             4 => BlockType::PrintMetadata,
//             5 => BlockType::Thumbnail,
//             _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
//         })
//     })(input)
// }

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Compression {
    #[default]
    None = 0,
    Deflate = 1,
    // Heatshrink algorithm with window size 11 and lookahead size 4
    HeatShrink11 = 2,
    // Heatshrink algorithm with window size 12 and lookahead size 4
    HeatShrink12 = 3,
}

fn compression_parse(input: &[u8]) -> IResult<&[u8], Compression> {
    map_res(le_u16, |compression: u16| {
        // help
        Ok(match compression {
            0 => Compression::None,
            1 => Compression::Deflate,
            2 => Compression::HeatShrink11,
            3 => Compression::HeatShrink12,
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        })
    })(input)
}
