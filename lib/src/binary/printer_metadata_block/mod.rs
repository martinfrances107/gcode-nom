use core::fmt::Display;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
};
use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult,
};

mod param;
use param::param_parser;
use param::Param;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrinterMetadataBlock {
    header: BlockHeader,
    param: Param,
    data: String,
    checksum: Option<u32>,
}
impl Display for PrinterMetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "-------------------------- PrinterMetadataBlock --------------------------"
        )?;
        writeln!(f,)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f,)?;
        write!(f, "-------------------------- PrinterMetadataBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static PRINTER_METADATA_BLOCK_ID: u16 = 3u16;
pub fn printer_metadata_parser_with_checksum(input: &[u8]) -> IResult<&[u8], PrinterMetadataBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!(
                "looking for PRINTER_METADATA_BLOCK_ID {} cond {}",
                block_type,
                *block_type == PRINTER_METADATA_BLOCK_ID
            );
            *block_type == PRINTER_METADATA_BLOCK_ID
        }),
        block_header_parser,
    )(input)?;

    let BlockHeader {
        compression_type,
        uncompressed_size,
        ..
    } = header.clone();
    eprintln!("about to check param ");
    let (after_param, param) = param_parser(after_block_header)?;
    eprintln!("Param value -- {:#?}", param);
    eprintln!("uncompressed_size -- {:#?}", uncompressed_size);
    // Decompress datablock
    let (after_data, data_raw) = match compression_type {
        CompressionType::None => take(uncompressed_size)(after_block_header)?,
        CompressionType::Deflate => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_param)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_param)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_param)?;
            // Must decompress here
            todo!()
        }
    };

    let data = String::from_utf8(data_raw.to_vec()).expect("raw data error");

    let (after_checksum, checksum_value) = le_u32(after_data)?;

    Ok((
        after_checksum,
        PrinterMetadataBlock {
            header,
            param,
            data,
            checksum: Some(checksum_value),
        },
    ))
}
