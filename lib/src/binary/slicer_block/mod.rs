use core::fmt::{Display, Write};

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

use super::default_params::param_parser;
use super::default_params::Param;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicerBlock {
    header: BlockHeader,
    param: Param,
    data: String,
    checksum: Option<u32>,
}
impl Display for SlicerBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- SlicerBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f)?;
        write!(f, "-------------------------- SlicerBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl SlicerBlock {
    /// Write to formatter a markdown block.
    pub fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: Write,
    {
        writeln!(f)?;
        writeln!(f, "## SlicerBlock")?;
        writeln!(f)?;
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f, "<details>")?;
        writeln!(f, "<summary>DataBlock</summary>")?;
        writeln!(f, "<br>")?;
        writeln!(f, "{}", self.data)?;
        writeln!(f, "</details>")?;
        writeln!(f)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static SLICER_BLOCK_ID: u16 = 2u16;
pub fn slicer_parser_with_checksum(input: &[u8]) -> IResult<&[u8], SlicerBlock, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for SLICER_BLOCK_ID {SLICER_BLOCK_ID} found {block_type} cond {}",
                *block_type == SLICER_BLOCK_ID
            );
            *block_type == SLICER_BLOCK_ID
        }),
        block_header_parser,
    )
    .parse(input)
    .map_err(|e| {
        e.map(|e| {
            BlockError::FileHeader(format!(
                "Slicer: Failed preamble version and checksum: {e:#?}"
            ))
        })
    })?;

    log::info!("Found slicer block id");
    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
        ..
    } = header.clone();

    let (after_param, param) = param_parser(after_block_header).map_err(|e| {
        e.map(|e: nom::error::Error<_>| {
            BlockError::Param(format!("slider: Failed to decode parameter block: {e:#?}"))
        })
    })?;

    // Decompress data block
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "slicer: Compression None - Failed to extract data block: {e:#?}"
                    ))
                })
            })?;
            let data = String::from_utf8(data_raw.to_vec()).map_err(|e| {
                nom::Err::Error(BlockError::Decompression(format!(
                    "slicer: Compression None Failed to decode data block as utf8: {e:#?}"
                )))
            })?;
            (remain, data)
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "slicer: Compression Deflate - Failed to extract raw(compressed) data block: {e:#?}"
                    ))
                })
            })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => {
                    let data = String::from_utf8(decoded).map_err(|e| {
                        nom::Err::Error(BlockError::Decompression(format!(
                            "slicer: Compression Deflate - Failed to decode data block as utf8: {e:#?}"
                        )))
                    })?;
                    (remain, data)
                }
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(format!(
                        "slicer: Compression Deflate - Failed to decode data block: {msg}"
                    ))));
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: nom::error::Error<_>| {
                        BlockError::Decompression(format!(
                        "slicer: Compression HeatShrink11 - Failed to extract data block: {e:#?}"
                    ))
                    })
                })?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: nom::error::Error<_>| {
                        BlockError::Decompression(format!(
                        "slicer: Compression HeatShrink12 - Failed to extract data block: {e:#?}"
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
            BlockError::Checksum(format!("slicer: Failed to extract checksum: {e:#?}"))
        })
    })?;

    let param_size = 2;
    let payload_size = match compression_type {
        CompressionType::None => uncompressed_size as usize,
        _ => compressed_size.unwrap() as usize,
    };
    let block_size = header.size_in_bytes() + param_size + payload_size;
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!("slicer checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("fail checksum");
        return Err(nom::Err::Error(BlockError::Checksum(format!(
            "slicer: checksum mismatch 0x{checksum:04x} computed 0x{computed_checksum:04x}"
        ))));
    }

    Ok((
        after_checksum,
        SlicerBlock {
            header,
            param,
            data,
            checksum: Some(checksum),
        },
    ))
}
