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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlicerBlock {
    header: BlockHeader,
    param: Param,
    data: String,
    checksum: Option<u32>,
}
impl Display for SlicerBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

static SLICER_BLOCK_ID: u16 = 6u16;
pub fn slicer_parser_with_checksum(input: &[u8]) -> IResult<&[u8], SlicerBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!(
                "looking for SLICER_BLOCK_ID {SLICER_BLOCK_ID} found {block_type} cond {}",
                *block_type == SLICER_BLOCK_ID
            );
            *block_type == SLICER_BLOCK_ID
        }),
        block_header_parser,
    )(input)?;

    let BlockHeader {
        compression_type,
        uncompressed_size,
        ..
    } = header.clone();
    println!("about to check param ");
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

    let data = match param.encoding {
        0 => String::from_utf8(data_raw.to_vec()).expect("raw data error"),
        2u16 => String::from("A meatpacked string with comments handling"),
        _ => {
            panic!("bad encoding");
        }
    };

    let (after_checksum, checksum_value) = le_u32(after_data)?;

    Ok((
        after_checksum,
        SlicerBlock {
            header,
            param,
            data,
            checksum: Some(checksum_value),
        },
    ))
}
