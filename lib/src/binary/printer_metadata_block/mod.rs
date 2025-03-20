use core::fmt::Display;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    default_params::{param_parser, Param},
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterMetadataBlock {
    header: BlockHeader,
    param: Param,
    data: String,
    checksum: Option<u32>,
}
impl Display for PrinterMetadataBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- PrinterMetadataBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f)?;
        write!(f, "-------------------------- PrinterMetadataBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl PrinterMetadataBlock {
    /// Write to formatter a markdown block.
    pub fn markdown<W>(&self, mut f: W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        writeln!(f)?;
        writeln!(f, "## PrinterMetadataBlock")?;
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
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static PRINTER_METADATA_BLOCK_ID: u16 = 3u16;
pub fn printer_metadata_parser_with_checksum(
    input: &[u8],
) -> IResult<&[u8], PrinterMetadataBlock, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for PRINTER_METADATA_BLOCK_ID {PRINTER_METADATA_BLOCK_ID} {block_type} cond {}",
                *block_type == PRINTER_METADATA_BLOCK_ID
            );
            *block_type == PRINTER_METADATA_BLOCK_ID
        }),
        block_header_parser,
    ).parse(input)
    .map_err(|e| {
        e.map(|e| BlockError::FileHeader(format!("Failed preamble version and checksum: {e:#?}")))
    })?;

    log::info!("Found printer metadata block id.");
    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
    } = header.clone();

    let (after_param, param) = param_parser(after_block_header).map_err(|e| {
        e.map(|e| {
            BlockError::Param(format!(
                "printer_metadata: Failed to decode parameter block: {e:#?}"
            ))
        })
    })?;

    // Decompress data block
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(uncompressed_size)(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "printer_metadata: No compression - Failed to process raw data: {e:#?}"
                    ))
                })
            })?;
            let data = String::from_utf8(data_raw.to_vec()).map_err(|e| {
                log::error!("Failed to decode raw data {e}");
                nom::Err::Error(BlockError::Decompression(format!(
                    "printer_metadata: No compression - Failed to process raw data as a utf8: {e:#?}"
                )))
            })?;
            (remain, data)
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "printer_metadata: Deflate - Failed to extract compressed data: {e:#?}"
                    ))
                })
            })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => {
                    let data = String::from_utf8(decoded).map_err(|e| {
                        log::error!("Failed to decode decompressed data {e}");
                        nom::Err::Error(BlockError::Decompression(format!(
                            "printer_metadata: Deflate - Failed to process inflated data as utf8: {e:#?}"
                        )))
                    })?;
                    (remain, data)
                }
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(format!(
                        "printer_metadata: Deflate - Failed to decode decompressed data: {msg}"
                    ))));
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: nom::error::Error<_>| {
                        BlockError::Decompression(format!(
                        "printer_metadata: HeatShrink11 - Failed to extract compressed data: {e:#?}"
                    ))
                    })
                })?;

            unimplemented!("printer_metadata_block: Decoding with the meatpacking algorithm is not yet support please create an issue.");
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) =
                take(compressed_size.unwrap())(after_param).map_err(|e| {
                    e.map(|e: nom::error::Error<_>| {
                        BlockError::Decompression(format!(
                        "printer_metadata: HeatShrink12 - Failed to extract compressed data: {e:#?}"
                    ))
                    })
                })?;
            // Must decompress here
            unimplemented!("printer_metadata_block: Decoding with the meatpacking algorithm is not yet support please create an issue.");
        }
    };

    let (after_checksum, checksum) = le_u32(after_data).map_err(|e| {
        e.map(|e: nom::error::Error<_>| {
            BlockError::Checksum(format!(
                "printer_metadata: Failed to decode checksum: {e:#?}"
            ))
        })
    })?;

    let param_size = 2;
    let block_size = header.size_in_bytes() + param_size + header.payload_size_in_bytes();
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!(
        "printer_metadata checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
    );
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("fail checksum");
        return Err(nom::Err::Error(BlockError::Checksum(format!(
            "printer_metadata: Checksum mismatch: {checksum} != {computed_checksum}"
        ))));
    }

    Ok((
        after_checksum,
        PrinterMetadataBlock {
            header,
            param,
            data,
            checksum: Some(checksum),
        },
    ))
}
