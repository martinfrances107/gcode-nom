use core::fmt::Display;
use std::sync::LazyLock;

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
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

use meatpack::{MeatPackResult, Unpacker};
use param::param_parser;
use param::Encoding;

mod param;

static CONFIG_W12_L4: LazyLock<Config> =
    LazyLock::new(|| Config::new(12, 4).expect("Failed to configure HeatshrinkW11L4 decoder"));

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GCodeBlock {
    header: BlockHeader,
    encoding: Encoding,
    data: String,
    checksum: Option<u32>,
}

impl Display for GCodeBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "-------------------------- GCodeBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "encoding {:#?}", self.encoding)?;
        writeln!(f)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f)?;
        write!(f, "-------------------------- GCodeBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

impl Markdown for Vec<GCodeBlock> {
    fn markdown<W>(&self, f: &mut W) -> core::fmt::Result
    where
        W: core::fmt::Write,
    {
        writeln!(f, "## GCodeBlocks")?;
        writeln!(f)?;
        for (i, gcode) in self.iter().enumerate() {
            // All titles (for a given level), must be unique
            writeln!(f, "### GCodeBlock {i}")?;
            writeln!(f)?;
            gcode.headless_markdown(&mut *f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl GCodeBlock {
    /// Write to formatter a markdown block.
    pub(super) fn headless_markdown<W>(&self, mut f: W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        writeln!(f, "### Params")?;
        writeln!(f, "encoding {:#?}", self.encoding)?;
        writeln!(f)?;
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

static CODE_BLOCK_ID: u16 = 1u16;
pub fn gcode_parser_with_checksum(input: &[u8]) -> IResult<&[u8], GCodeBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for CODE_BLOCK_ID {CODE_BLOCK_ID} found {block_type} cond {}",
                *block_type == CODE_BLOCK_ID
            );
            *block_type == CODE_BLOCK_ID
        }),
        block_header_parser,
    )
    .parse(input)?;
    log::info!("Found G-code block id.");
    let BlockHeader {
        compression_type,
        compressed_size,
        uncompressed_size,
    } = header.clone();

    let (after_param, encoding) = param_parser(after_block_header)?;
    log::info!("encoding {encoding}");
    // Decompress data block
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(uncompressed_size)(after_param)?;
            let data = String::from_utf8(data_raw.to_vec()).expect("raw data error");
            (remain, data)
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param)?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => {
                    let data = String::from_utf8(decoded).expect("raw data error");
                    (remain, data)
                }
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    panic!()
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(compressed_size.unwrap())(after_param)?;
            // Must decompress here

            // use CONFIG_W11_L4 here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param)?;

            // TODO Figure out why size is is off by 1 -  crashes with buffer was not large enough.
            let mut scratch = vec![0u8; 1 + uncompressed_size as usize];

            let data = match decode(encoded, &mut scratch, &CONFIG_W12_L4) {
                Ok(decoded_hs) => match encoding {
                    Encoding::None => String::from_utf8(decoded_hs.to_vec())
                        .expect("Simple heatshrink12 output is a bad string"),
                    Encoding::MeatPackAlgorithm => {
                        log::error!("Must decode with standard meat packing algorithm");
                        panic!();
                    }
                    Encoding::MeatPackModifiedAlgorithm => {
                        let mut data = String::new();
                        let mut unpacker = Unpacker::<64>::default();
                        for b in decoded_hs {
                            match unpacker.unpack(b) {
                                Ok(MeatPackResult::WaitingForNextByte) => {
                                    // absorb byte and continue
                                }
                                Ok(MeatPackResult::Line(line)) => {
                                    let line = std::str::from_utf8(line).unwrap();
                                    data.push_str(line);
                                }
                                Err(e) => {
                                    log::error!(
                                        "{}",
                                        format!("failed to unpack meatpack data {e:#?}")
                                    );
                                    panic!();
                                }
                            }
                        }
                        data
                    }
                },
                Err(e) => {
                    log::error!("HeatShrink12: The output buffer was not large enough to hold the decompressed data {e:#?}");
                    panic!();
                }
            };

            (remain, data)
        }
    };

    let (after_checksum, checksum) = le_u32(after_data)?;

    let param_size = 2;
    let payload_size = match compression_type {
        CompressionType::None => uncompressed_size as usize,
        _ => compressed_size.unwrap() as usize,
    };
    let block_size = header.size_in_bytes() + param_size + payload_size;
    let crc_input = &input[..block_size];
    let computed_checksum = crc32fast::hash(crc_input);

    log::debug!("gcode checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
    if checksum == computed_checksum {
        log::debug!("checksum match");
    } else {
        log::error!("fail checksum");
        panic!("gcode metadata block failed checksum");
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
