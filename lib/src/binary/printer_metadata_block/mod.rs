use core::fmt::Display;

use super::{
    block_header::{block_header_parser, BlockHeader},
    compression_type::CompressionType,
    default_params::param_parser,
    default_params::Param,
};
use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, InputTake,
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

static PRINTER_METADATA_BLOCK_ID: u16 = 3u16;
pub fn printer_metadata_parser_with_checksum(input: &[u8]) -> IResult<&[u8], PrinterMetadataBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!(
                "looking for PRINTER_METADATA_BLOCK_ID {PRINTER_METADATA_BLOCK_ID} {block_type} cond {}",
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
    println!("printer_metadata about to check param ");
    let (after_param, param) = param_parser(after_block_header)?;
    println!("Param value -- {param:#?}");
    println!("uncompressed_size -- {uncompressed_size:#?}");
    // Decompress datablock
    let (after_data, data_raw) = match compression_type {
        CompressionType::None => take(uncompressed_size)(after_param)?,
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

    let (after_checksum, checksum) = le_u32(after_data)?;

    let param_size = 2;
    let block_size = header.size_in_bytes() + param_size + header.payload_size_in_bytes();
    let crc_input: Vec<u8> = input.take(block_size).to_vec();
    let computed_checksum = crc32fast::hash(&crc_input);

    print!(
        "printer_metadata checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
    );
    if checksum == computed_checksum {
        println!(" match");
    } else {
        println!(" fail");
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
