use core::fmt::Display;

use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult,
};

use super::{block_header::block_header_parser, block_header::BlockHeader, CompressionType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileMetadataBlock {
    header: BlockHeader,
    // This string is a table of "key  = value" pairs
    data: String,
    checksum: Option<u32>,
}

impl Display for FileMetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FileMetadataBlock")?;
        writeln!(f,)?;
        writeln!(f, "DataBlock {}", self.data)?;
        writeln!(f,)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "{checksum}")?,
            None => write!(f, "No checksum")?,
        };
        Ok(())
    }
}

pub fn file_metadata_parser_with_checksum(input: &[u8]) -> IResult<&[u8], FileMetadataBlock> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            println!("block type{} cond {}", block_type, *block_type == 0u16);
            *block_type == 0u16
        }),
        block_header_parser,
    )(input)?;

    let BlockHeader {
        compression_type,
        uncompressed_size,
        ..
    } = header.clone();

    // Decompress datablock
    let (after_data, data_raw) = match compression_type {
        CompressionType::None => take(uncompressed_size)(after_block_header)?,
        CompressionType::Deflate => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_block_header)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_block_header)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(after_block_header)?;
            // Must decompress here
            todo!()
        }
    };

    let data = String::from_utf8(data_raw.to_vec()).expect("raw data error");

    let (after_checksum, checksum_value) = le_u32(after_data)?;

    Ok((
        after_checksum,
        FileMetadataBlock {
            header,
            data,
            checksum: Some(checksum_value),
        },
    ))
}
