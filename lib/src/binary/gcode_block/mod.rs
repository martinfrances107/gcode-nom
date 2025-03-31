use core::fmt::Display;

use crate::binary::{default_params::param_parser, BlockError};
use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

use super::{
    block_header::{block_header_parser, BlockHeader},
    default_params::Param,
    inflate::decompress_data_block,
    Markdown,
};

/// Parser extracts Vec<GCodeBlock> from file.
pub mod extractor;
/// Converts a gcode block into a SVG file.
pub mod svg;

/// A wrapper for a series of gcode commands.
///
/// also wraps header, encoding and checksum
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GCodeBlock<'a> {
    /// Header
    pub header: BlockHeader,
    /// Param the data's encoding.
    pub param: Param,
    /// A series of gcode commands
    pub data: &'a [u8],
    checksum: Option<u32>,
}

impl Display for GCodeBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let datablock: String =
            match decompress_data_block(self.data, &self.param.encoding, &self.header) {
                Ok((_remain, data)) => String::from_utf8_lossy(&data).to_string(),
                Err(_e) => String::from("failed to decompress"),
            };
        writeln!(
            f,
            "-------------------------- GCodeBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "encoding {:#?}", self.param.encoding)?;
        writeln!(f)?;
        writeln!(f, "DataBlock {datablock:?}")?;
        writeln!(f)?;
        write!(f, "-------------------------- GCodeBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl Markdown for Vec<GCodeBlock<'_>> {
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: core::fmt::Write,
    {
        if self.is_empty() {
            return Ok(());
        }
        writeln!(f)?;
        writeln!(f, "## GCodeBlocks")?;
        for (i, gcode) in self.iter().enumerate() {
            writeln!(f)?;
            // All titles (for a given level), must be unique
            writeln!(f, "### GCodeBlock {i}")?;
            writeln!(f)?;
            gcode.headless_markdown(&mut *f)?;
        }
        Ok(())
    }
}

impl GCodeBlock<'_> {
    /// Write to formatter a markdown block.
    pub(super) fn headless_markdown<W>(&self, mut f: W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        let datablock = match decompress_data_block(self.data, &self.param.encoding, &self.header) {
            Ok((_remain, data)) => String::from_utf8_lossy(&data).to_string(),
            Err(_e) => String::from("failed to decompress"),
        };
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "encoding {:#?}", self.param.encoding)?;
        writeln!(f)?;
        writeln!(f, "<details>")?;
        writeln!(f, "<summary>DataBlock</summary>")?;
        writeln!(f, "<br>")?;
        writeln!(f, "{datablock:?}")?;
        writeln!(f, "</details>")?;
        writeln!(f)?;

        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static CODE_BLOCK_ID: u16 = 1u16;

/// Parses a gcode block without validating checksum.
///
/// See also `gcode_parser_with_checksum()`.
///
/// # Errors
///
/// When no match is found.
pub fn gcode_parser(input: &[u8]) -> IResult<&[u8], GCodeBlock, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "gcode_block: Looking for CODE_BLOCK_ID {CODE_BLOCK_ID} found {block_type} cond {}",
                *block_type == CODE_BLOCK_ID
            );
            *block_type == CODE_BLOCK_ID
        }),
        block_header_parser,
    )
    .parse(input)
    .map_err(|e| {
        log::error!("Failed to parse block header {e}");
        e.map(|_e| BlockError::FileHeader("Failed preamble version and checksum".to_string()))
    })?;

    log::info!("Found G-code block id.");
    let (after_param, param) = param_parser(after_block_header).map_err(|e| {
        log::error!("Failed to parse param {e}");
        e.map(|_e| BlockError::FileHeader("Failed to parse param".to_string()))
    })?;

    log::info!("param {param:?}");
    // Decompress data block.
    let (after_data, data) = match header.compressed_size {
        Some(size) => take(size)(after_param)?,
        None => take(header.uncompressed_size)(after_param)?,
    };

    let (after_checksum, checksum) = match le_u32::<_, BlockError>(after_data) {
        Ok((after_checksum, checksum)) => (after_checksum, checksum),
        Err(_e) => {
            let msg = "gcode_block: Failed to extract checksum".to_string();
            log::error!("{msg}");
            return Err(nom::Err::Error(BlockError::Checksum(msg)));
        }
    };

    Ok((
        after_checksum,
        GCodeBlock {
            header,
            param,
            data,
            checksum: Some(checksum),
        },
    ))
}

/// Parses a gcode block, while validating checksum.
///
/// See also `gcode_parser()`.
///
/// # Errors
///
/// When no match is found.
pub fn gcode_parser_with_checksum(input: &[u8]) -> IResult<&[u8], GCodeBlock, BlockError> {
    let (remain, gcode) = gcode_parser(input)?;
    if let Some(checksum) = gcode.checksum {
        let param_size = 2;
        let block_size =
            gcode.header.size_in_bytes() + param_size + gcode.header.payload_size_in_bytes();
        let crc_input = &input[..block_size];
        let computed_checksum = crc32fast::hash(crc_input);

        log::debug!("gcode checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
        if checksum == computed_checksum {
            log::debug!("checksum match");
        } else {
            log::error!("fail checksum");
            return Err(nom::Err::Error(BlockError::Checksum(format!(
                "slicer: checksum mismatch 0x{checksum:04x} computed 0x{computed_checksum:04x}"
            ))));
        }
    }

    Ok((remain, gcode))
}
