use core::fmt::Display;
use std::fmt::Write;

use super::{
    block_header::{block_header_parser, BlockHeader},
    BlockError,
};

use nom::{
    bytes::streaming::take,
    combinator::verify,
    error::Error,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

mod param;
use param::param_parser;
use param::Param;

use crate::binary::{CompressionType, Markdown};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThumbnailBlock<'a> {
    header: BlockHeader,
    pub param: Param,
    pub data: &'a [u8],
    checksum: Option<u32>,
}
impl Display for ThumbnailBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- ThumbnailBlock --------------------------"
        )?;
        writeln!(f)?;
        writeln!(f, "Params")?;
        writeln!(f, "{}", self.param)?;
        writeln!(f, "DataBlock omitted")?;
        writeln!(f)?;
        write!(f, "-------------------------- ThumbnailBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        }
        Ok(())
    }
}

impl Markdown for Vec<ThumbnailBlock<'_>> {
    /// Write to formatter a markdown block.
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: Write,
    {
        writeln!(f)?;
        writeln!(f, "## ThumbnailBlocks")?;
        for (i, thumbnail) in self.iter().enumerate() {
            // All titles (for a given level), must be unique
            // otherwise, as per spec,  table of content block cannot be constructed.
            writeln!(f)?;
            writeln!(f, "### ThumbnailBlock {i}")?;
            writeln!(f)?;
            thumbnail.headless_markdown(f)?;
        }
        Ok(())
    }
}

impl ThumbnailBlock<'_> {
    /// Write to formatter a markdown block.
    pub(super) fn headless_markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: Write,
    {
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "{}", self.param)?;
        writeln!(f, "DataBlock omitted")?;
        writeln!(f)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        }
        Ok(())
    }
}

static THUMBNAIL_BLOCK_ID: u16 = 5u16;
pub fn thumbnail_parser(input: &[u8]) -> IResult<&[u8], ThumbnailBlock, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for THUMBNAIL_BLOCK_ID {THUMBNAIL_BLOCK_ID} found {block_type} cond {}",
                *block_type == THUMBNAIL_BLOCK_ID
            );
            *block_type == THUMBNAIL_BLOCK_ID
        }),
        block_header_parser,
    )
    .parse(input)
    .map_err(|e| e.map(|_e| BlockError::Thumbnail))?;

    log::info!("Found thumbnail block id");

    let (after_param, param) =
        param_parser(after_block_header).map_err(|e| e.map(|_e| BlockError::Thumbnail))?;

    // Decompress data block
    let (after_data, data) = match header.compressed_size {
        Some(size) => take(size)(after_param)?,
        None => take(header.uncompressed_size)(after_param)?,
    };

    let (after_checksum, checksum) =
        le_u32(after_data).map_err(|e| e.map(|_e: Error<_>| BlockError::Thumbnail))?;

    Ok((
        after_checksum,
        ThumbnailBlock {
            header,
            param,
            data,
            checksum: Some(checksum),
        },
    ))
}

pub fn thumbnail_parser_with_checksum(input: &[u8]) -> IResult<&[u8], ThumbnailBlock, BlockError> {
    let (remain, thumbnail) = thumbnail_parser(input)?;

    if let Some(checksum) = thumbnail.checksum {
        let param_size = 6;
        let payload_size = match thumbnail.header.compression_type {
            CompressionType::None => thumbnail.header.uncompressed_size as usize,
            _ => thumbnail.header.compressed_size.unwrap() as usize,
        };
        let block_size = thumbnail.header.size_in_bytes() + param_size + payload_size;
        let crc_input = &input[..block_size];
        let computed_checksum = crc32fast::hash(crc_input);

        log::debug!(
            "thumbnail checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
        );
        if checksum == computed_checksum {
            log::debug!("checksum match");
        } else {
            log::error!("fail checksum");
            return Err(nom::Err::Error(BlockError::Thumbnail));
        }
    }

    Ok((remain, thumbnail))
}
