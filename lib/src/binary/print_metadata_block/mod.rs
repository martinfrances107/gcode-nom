use core::fmt::Display;

use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, InputTake,
};

mod param;

use super::{block_header::block_header_parser, block_header::BlockHeader, CompressionType};
use param::param_parser;
use param::Param;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintMetadataBlock {
    header: BlockHeader,
    param: Param,
    // This string is a table of "key  = value" pairs
    data: String,
    checksum: Option<u32>,
}

impl Display for PrintMetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "-------------------------- PrintMetadataBlock --------------------------"
        )?;
        writeln!(f)?;
        write!(f, "Params")?;
        writeln!(f, "params 0x{:?}", self.param)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f)?;

        write!(f, "-------------------------- PrintMetadataBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

static PRINT_METADATA_BLOCK_ID: u16 = 4u16;
pub fn print_metadata_parser_with_checksum(input: &[u8]) -> IResult<&[u8], PrintMetadataBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!(
                "Looking for PRINT_METADATA_BLOCK_ID {PRINT_METADATA_BLOCK_ID} found {block_type} cond {}",
                *block_type == PRINT_METADATA_BLOCK_ID
            );
            *block_type == PRINT_METADATA_BLOCK_ID
        }),
        block_header_parser,
    )(input)?;

    let BlockHeader {
        compression_type,
        uncompressed_size,
        compressed_size,
    } = header.clone();

    println!("uncompressed_size -- {uncompressed_size:#?}");
    println!("compressed_size -- {compressed_size:#?}");
    println!("compression_type -- {compression_type:#?}");
    let (after_param, param) = param_parser(after_block_header)?;

    println!("extracted param {param:?}");
    // Decompress datablock
    let (after_data, data) = match compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(uncompressed_size)(after_param)?;
            let data = String::from_utf8(data_raw.to_vec()).expect("raw data error");
            (remain, data)
        }
        CompressionType::Deflate => {
            let (remain, _data_compressed) = take(compressed_size.unwrap())(after_param)?;
            // let mut d = GzDecoder::new(data_compressed);
            // let mut data = String::new();
            // d.read_to_string(&mut data).unwrap();
            let data = String::from("contains compressed data");
            (remain, data)

            // take(uncompressed_size)(after_param)?
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

    let (after_checksum, checksum) = le_u32(after_data)?;

    let param_size = 2;
    let payload_size = match compression_type {
        CompressionType::None => uncompressed_size as usize,
        _ => compressed_size.unwrap() as usize,
    };
    let block_size = header.size_in_bytes() + param_size + payload_size;
    let crc_input: Vec<u8> = input.take(block_size).to_vec();
    let computed_checksum = crc32fast::hash(&crc_input);

    print!("print_metadata checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} ");
    if checksum == computed_checksum {
        println!(" match");
    } else {
        println!(" fail");
        panic!("print metadata block failed checksum");
    }

    Ok((
        after_checksum,
        PrintMetadataBlock {
            param,
            header,
            data,
            checksum: Some(checksum),
        },
    ))
}
