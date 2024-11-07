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
use param::Param;
use param::{param_parser, Format};

static THUMBNAIL_BLOCK_ID: u16 = 5u16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThumbnailBlock {
    header: BlockHeader,
    param: Param,
    data: Vec<u8>,
    checksum: Option<u32>,
}
impl Display for ThumbnailBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "-------------------------- ThumbnailBlock --------------------------"
        )?;
        writeln!(f)?;
        writeln!(f, "Params")?;
        write!(f, "{}", self.param)?;
        writeln!(f, "DataBlock omitted")?;
        writeln!(f)?;
        write!(f, "-------------------------- ThumbnailBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Ckecksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        };
        Ok(())
    }
}

pub fn thumbnail_parser_with_checksum(input: &[u8]) -> IResult<&[u8], ThumbnailBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!(
                "looking for THUMBNAIL_BLOCK_ID {} cond {}",
                block_type,
                *block_type == THUMBNAIL_BLOCK_ID
            );
            *block_type == THUMBNAIL_BLOCK_ID
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
    eprintln!("Param value -- {param:#?}");
    eprintln!("uncompressed_size -- {uncompressed_size:#?}");
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

    let data = data_raw.to_vec();
    match param.format {
        Format::Qoi => std::fs::write("thumb.qoi", &data).unwrap(),
        Format::Jpg => std::fs::write("thumb.jpg", &data).unwrap(),
        Format::Png => std::fs::write("thumb.png", &data).unwrap(),
    }

    let (after_checksum, checksum_value) = le_u32(after_data)?;

    Ok((
        after_checksum,
        ThumbnailBlock {
            header,
            param,
            data,
            checksum: Some(checksum_value),
        },
    ))
}
