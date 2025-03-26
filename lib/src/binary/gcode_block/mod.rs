use core::fmt::Display;
use std::{borrow::Cow, sync::LazyLock};

use crate::binary::BlockError;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    Markdown,
};

use heatshrink::{decode, Config};
use inflate::inflate_bytes_zlib;
use nom::{
    bytes::streaming::take,
    combinator::verify,
    error::Error,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

use meatpack::{MeatPackResult, Unpacker};
use param::param_parser;
use param::Encoding;

mod param;
/// Converts a gcode block into a SVG file.
pub mod svg;

static CONFIG_W12_L4: LazyLock<Config> =
    LazyLock::new(|| Config::new(12, 4).expect("Failed to configure HeatshrinkW11L4 decoder"));

/// A wrapper for a series of gcode commands.
///
/// also wraps header, encoding and checksum
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GCodeBlock<'a> {
    header: BlockHeader,
    encoding: Encoding,
    /// A series of gcode commands
    pub data: Cow<'a, [u8]>,
    checksum: Option<u32>,
}

impl Display for GCodeBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- GCodeBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "encoding {:#?}", self.encoding)?;
        writeln!(f)?;
        writeln!(f, "DataBlock {:?}", String::from_utf8_lossy(&self.data))?;
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
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "encoding {:#?}", self.encoding)?;
        writeln!(f)?;
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

static CODE_BLOCK_ID: u16 = 1u16;
pub(crate) fn gcode_parser_with_checksum(input: &[u8]) -> IResult<&[u8], GCodeBlock, BlockError> {
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
        e.map(|e| BlockError::FileHeader(format!("Failed preamble version and checksum: {e:?}")))
    })?;

    log::info!("Found G-code block id.");
    let BlockHeader {
        compression_type,
        compressed_size,
        uncompressed_size,
    } = header.clone();

    let (after_param, encoding) = param_parser(after_block_header).map_err(|e| {
        log::error!("Failed to parse param {e}");
        e.map(|e| BlockError::FileHeader(format!("Failed to parse param {e:?}")))
    })?;

    log::info!("encoding {encoding}");
    // Decompress data block.
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            // Take the raw data block.
            let (remain, data_raw) = take::<_, _, Error<&[u8]>>(uncompressed_size)(after_param)
                .map_err(|e| {
                    e.map(|e: nom::error::Error<_>| {
                        BlockError::Decompression(format!(
                            "gcode_block: No compression - Failed to process raw data: {e:?}"
                        ))
                    })
                })?;
            (remain, Cow::from(data_raw))
        }
        CompressionType::Deflate => {
            // Take the raw data block.
            let (remain, encoded) = take::<_, _, BlockError>(compressed_size.unwrap())(after_param)
                .map_err(|e| {
                    e.map(|e| {
                        BlockError::Decompression(format!(
                            "gcode_block: Deflate - Failed to process raw data: {e:?}"
                        ))
                    })
                })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, Cow::from(decoded)),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(format!(
                        "gcode block: Deflate - Failed to process inflated data as utf8: {msg}"
                    ))));
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) =
                take::<_, _, BlockError>(compressed_size.unwrap())(after_param).map_err(|e| {
                    log::error!("gcode_block: Failed to decode decompressed data {e}");
                    nom::Err::Error(BlockError::Decompression(format!(
                    "gcode_block: HeatShrink11 - Failed to process inflated data as utf8: {e:?}"
                )))
                })?;

            // Must decompress here

            // use CONFIG_W11_L4 here
            log::info!("gcode_block: TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (remain, encoded) = take::<_, _, BlockError>(compressed_size.unwrap())(after_param)
                .map_err(|e| {
                    e.map(|e| {
                        BlockError::Decompression(format!(
                            "gcode_block: HeatShrink12 - Failed to extract raw data: {e:?}"
                        ))
                    })
                })?;

            // TODO Figure out why size is is off by 1 -  crashes with buffer was not large enough.
            let mut scratch = vec![0u8; 1 + uncompressed_size as usize];

            let data = match decode(encoded, &mut scratch, &CONFIG_W12_L4) {
                Ok(decoded_hs) => match encoding {
                    Encoding::None => Cow::from(scratch),
                    Encoding::MeatPackAlgorithm => {
                        log::error!("Must decode with standard meat packing algorithm");
                        unimplemented!("Decoding with the meatpacking algorithm is not yet support please create an issue.");
                    }
                    Encoding::MeatPackModifiedAlgorithm => {
                        let mut data = vec![];
                        let mut unpacker = Unpacker::<64>::default();
                        for b in decoded_hs {
                            match unpacker.unpack(b) {
                                Ok(MeatPackResult::WaitingForNextByte) => {
                                    // absorb byte and continue
                                }
                                Ok(MeatPackResult::Line(line)) => {
                                    data.extend_from_slice(line);
                                }
                                Err(e) => {
                                    let msg = format!("Failed running the deflate MeatPackModifiedAlgorithm 'unpack()' algorithm {e:?}");
                                    log::error!("{msg}");
                                    return Err(nom::Err::Error(BlockError::Decompression(msg)));
                                }
                            }
                        }
                        Cow::from(data)
                    }
                },
                Err(e) => {
                    let msg = format!("GCodeBlock:  Failed running the deflate MeatPackModifiedAlgorithm 'decode()' algorithm {e:?}");
                    log::error!("{msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(msg)));
                }
            };

            (remain, data)
        }
    };

    let (after_checksum, checksum) = match le_u32::<_, BlockError>(after_data) {
        Ok((after_checksum, checksum)) => (after_checksum, checksum),
        Err(e) => {
            let msg = format!("gcode_block: Failed to extract checksum {e}");
            log::error!("{}", msg);
            return Err(nom::Err::Error(BlockError::Checksum(msg)));
        }
    };

    let param_size = 2;
    let payload_size = match compression_type {
        CompressionType::None => uncompressed_size as usize,
        _ => compressed_size.unwrap() as usize,
    };
    let block_size = header.size_in_bytes() + param_size + payload_size;
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!(
        "gcode_block: checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
    );
    if checksum == computed_checksum {
        log::debug!("gcode_block: checksum match");
    } else {
        log::error!("fail checksum");
        return Err(nom::Err::Error(BlockError::Checksum(format!(
            "FAIL: checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ",
        ))));
    }

    Ok((
        after_checksum,
        GCodeBlock {
            header,
            encoding,
            data,
            checksum: Some(checksum),
        },
    ))
}
