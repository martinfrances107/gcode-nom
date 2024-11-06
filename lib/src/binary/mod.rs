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
mod fh;
mod fm;
mod gcode;
mod pm;
mod sm;
mod thumb;

use std::fmt::Display;

use fh::{file_header_parser, FileHeader};
use fm::{file_metadata_parser, FileMetadataBlock};
use nom::IResult;

use compression_type::CompressionType;

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
pub fn bgcode_parser(input: &[u8]) -> IResult<&[u8], Bgcode> {
    let (mut remain, fh) = file_header_parser(input)?;
    let file_metadata = if let Ok((r, file_meta_data_actual)) = file_metadata_parser(remain) {
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
