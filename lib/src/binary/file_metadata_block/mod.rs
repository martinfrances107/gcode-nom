use core::fmt::Display;
use std::borrow::Cow;

use inflate::inflate_bytes_zlib;
use nom::Err::Error;
use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

use crate::binary::default_params::param_parser;
use crate::binary::default_params::Param;

use super::{
    block_header::{block_header_parser, BlockHeader},
    BlockError, CompressionType,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileMetadataBlock<'a> {
    header: BlockHeader,
    param: Param,
    // This string is a table of "key  = value" pairs
    data: Cow<'a, [u8]>,
    checksum: Option<u32>,
}

impl Display for FileMetadataBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- FileMetadataBlock --------------------------"
        )?;
        writeln!(f)?;
        write!(f, "Params")?;
        writeln!(f, "params 0x{:?}", self.param)?;
        writeln!(f, "DataBlock {}", String::from_utf8_lossy(&self.data))?;
        writeln!(f)?;

        write!(f, "-------------------------- FileMetadataBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl FileMetadataBlock<'_> {
    /// Write to formatter a markdown block.
    pub fn markdown<W>(&self, mut f: W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        writeln!(f)?;
        writeln!(f, "## FileMetadataBlock")?;
        writeln!(f)?;
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "params 0x{:?}", self.param)?;
        writeln!(f, "<details>")?;
        writeln!(f, "<summary>DataBlock</summary>")?;
        writeln!(f, "<br>")?;
        writeln!(f, "{:?}", String::from_utf8_lossy(&self.data))?;
        writeln!(f, "</details>")?;
        writeln!(f)?;

        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static FILE_METADATA_BLOCK_ID: u16 = 0u16;
pub fn file_metadata_parser_with_checksum(
    input: &[u8],
) -> IResult<&[u8], FileMetadataBlock, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for FILE_METADATA_BLOCK_ID {FILE_METADATA_BLOCK_ID} found {block_type} cond {}",
                *block_type == FILE_METADATA_BLOCK_ID
            );
            *block_type == FILE_METADATA_BLOCK_ID
        }),
        block_header_parser,
    ).parse(input).map_err(|e| {
        e.map(|e| BlockError::FileHeader(format!("file_metadata: Failed preamble version and checksum: {e:#?}")))
    })?;
    log::info!("Found file metadata block id.");
    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
    } = header.clone();

    let (after_param, param) = param_parser(after_block_header).map_err(|e| {
        e.map(|e| {
            BlockError::Param(format!(
                "file_metadata: Failed to decode parameter block: {e:#?}"
            ))
        })
    })?;

    // Decompress data-block
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "file_metadata: Failed to extract raw(uncompressed) data block: {e:#?}"
                    ))
                })
            })?;
            (remain, Cow::from(data_raw))
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "file_metadata: Compression::Deflate - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, Cow::from(decoded)),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(Error(BlockError::Decompression(format!(
                        "file_metadata: Compression: Deflate - Failed to decompress: {msg}"
                    ))))?;
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "file_metadata: Compression::HeatShrink11 - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "file_metadata: Compression::HeatShrink12 - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
    };

    let (after_checksum, checksum) = le_u32(after_data).map_err(|e| {
        e.map(|e: nom::error::Error<_>| {
            BlockError::Checksum(format!("file_metadata: Failed to decode checksum: {e:#?}"))
        })
    })?;

    let param_size = 2;
    let block_size = header.size_in_bytes() + param_size + header.payload_size_in_bytes();
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!(
        "file_metadata checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
    );
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("failed checksum");
        return Err(Error(BlockError::Checksum(format!(
            "file_metadata: Checksum mismatch: expected 0x{checksum:04x} got 0x{computed_checksum:04x}"
        ))));
    }

    Ok((
        after_checksum,
        FileMetadataBlock {
            param,
            header,
            data,
            checksum: Some(checksum),
        },
    ))
}
