use core::fmt::Display;
use std::fmt::Write;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    BlockError,
};
use inflate::inflate_bytes_zlib;
use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

mod param;
use param::param_parser;
use param::Param;

use crate::binary::Markdown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThumbnailBlock {
    header: BlockHeader,
    pub param: Param,
    /// binary data.
    pub data: Vec<u8>,
    checksum: Option<u32>,
}
impl Display for ThumbnailBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- ThumbnailBlock --------------------------"
        )?;
        writeln!(f)?;
        writeln!(f, "Params")?;
        write!(f, "{}", self.param)?;
        writeln!(f, "DataBlock omitted")?;
        writeln!(f)?;
        write!(f, "-------------------------- ThumbnailBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl Markdown for Vec<ThumbnailBlock> {
    /// Write to formatter a markdown block.
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: Write,
    {
        writeln!(f, "## ThumbnailBlocks")?;
        writeln!(f)?;
        for (i, thumbnail) in self.iter().enumerate() {
            // All titles (for a given level), must be unique
            // otherwise, as per spec,  table of content block cannot be constructed.
            writeln!(f, "### ThumbnailBlock {i}")?;
            thumbnail.headless_markdown(f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl ThumbnailBlock {
    /// Write to formatter a markdown block.
    pub(super) fn headless_markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: Write,
    {
        writeln!(f, "### Params")?;
        write!(f, "{}", self.param)?;
        writeln!(f, "DataBlock omitted")?;
        writeln!(f)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static THUMBNAIL_BLOCK_ID: u16 = 5u16;
pub fn thumbnail_parser_with_checksum(input: &[u8]) -> IResult<&[u8], ThumbnailBlock, BlockError> {
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
    .map_err(|e| {
        e.map(|e| {
            BlockError::FileHeader(format!(
                "thumbnail: Failed preamble version and checksum: {e:#?}"
            ))
        })
    })?;

    log::info!("Found thumbnail block id");
    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
    } = header.clone();

    let (after_param, param) = param_parser(after_block_header).map_err(|e| {
        e.map(|e| {
            BlockError::Param(format!(
                "thumbnail: Failed to decode parameter block: {e:#?}"
            ))
        })
    })?;

    // Decompress data block
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data) = take(uncompressed_size)(after_param)?;
            (remain, data.to_vec())
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param)?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, decoded),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    panic!()
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(compressed_size.unwrap())(after_param)?;
            log::info!("TODO: Must implement HeatShrink11");
            unimplemented!("HeatShrink11 is not yet implemented");
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(compressed_size.unwrap())(after_param)?;
            log::info!("TODO: Must implement heatshrink12");
            unimplemented!("HeatShrink11 is not yet implemented");
        }
    };

    let (after_checksum, checksum) = le_u32(after_data)?;

    let param_size = 6;
    let block_size = header.size_in_bytes() + param_size + header.payload_size_in_bytes();
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!("thumbnail checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("failed checksum");
        panic!("file metadata block failed checksum");
    }

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
