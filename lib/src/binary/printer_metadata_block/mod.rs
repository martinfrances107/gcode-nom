use core::fmt::Display;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    default_params::param_parser,
    default_params::Param,
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
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
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
        writeln!(f, "## PrinterMetadataBlock")?;
        writeln!(f, "### Params")?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static PRINTER_METADATA_BLOCK_ID: u16 = 3u16;
pub fn printer_metadata_parser_with_checksum(input: &[u8]) -> IResult<&[u8], PrinterMetadataBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for PRINTER_METADATA_BLOCK_ID {PRINTER_METADATA_BLOCK_ID} {block_type} cond {}",
                *block_type == PRINTER_METADATA_BLOCK_ID
            );
            *block_type == PRINTER_METADATA_BLOCK_ID
        }),
        block_header_parser,
    ).parse(input)?;

    log::info!("Found printer metadata block id.");
    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
    } = header.clone();

    let (after_param, param) = param_parser(after_block_header)?;

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
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(compressed_size.unwrap())(after_param)?;
            // Must decompress here
            todo!()
        }
    };

    let (after_checksum, checksum) = le_u32(after_data)?;

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
        // panic!("printer metadata block failed checksum");
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
