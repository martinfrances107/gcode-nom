use core::fmt::Display;
use std::{borrow::Cow, fmt::Write};

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    BlockError,
};
use inflate::inflate_bytes_zlib;
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

use crate::binary::Markdown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThumbnailBlock<'a> {
    header: BlockHeader,
    pub param: Param,
    /// binary data.
    pub data: Cow<'a, [u8]>,
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
        };
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
            let (remain, data) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: Error<_>| {
                    BlockError::Decompression(format!(
                        "thumbnail: Compression None - Failed to extract data block: {e:#?}"
                    ))
                })
            })?;
            (remain, Cow::from(data))
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param).map_err(|e| {
                e.map(|e: Error<_>| {
                    BlockError::Decompression(format!(
                        "thumbnail: Compression Deflate - Failed to extract data block: {e:#?}"
                    ))
                })
            })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, Cow::from(decoded)),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(format!(
                        "thumbnail: Compression Deflate - Failed to decode data block: {msg}"
                    ))));
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: Error<_>| {
                        BlockError::Decompression(format!(
                        "thumbnail: Compression HeatShrink11 - Failed to extract data block: {e:#?}"
                    ))
                    })
                })?;
            log::info!("TODO: Must implement HeatShrink11");
            unimplemented!("HeatShrink11 is not yet implemented");
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: Error<_>| {
                        BlockError::Decompression(format!(
                        "thumbnail: Compression HeatShrink12 - Failed to extract data block: {e:#?}"
                    ))
                    })
                })?;
            log::info!("TODO: Must implement heatshrink12");
            unimplemented!("HeatShrink11 is not yet implemented");
        }
    };

    let (after_checksum, checksum) = le_u32(after_data).map_err(|e| {
        e.map(|e: Error<_>| {
            BlockError::Checksum(format!(
                "thumbnail: Failed to decode checksum block: {e:#?}"
            ))
        })
    })?;

    let param_size = 6;
    let block_size = header.size_in_bytes() + param_size + header.payload_size_in_bytes();
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!("thumbnail checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("failed checksum");
        return Err(nom::Err::Error(BlockError::Checksum(format!(
            "thumbnail: checksum mismatch: expected 0x{checksum:04x} computed 0x{computed_checksum:04x}"
        ))));
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
