mod data_block;
mod param;

use core::fmt::Display;

use data_block::{data_parse, DataBlock};
use nom::{
    bytes::streaming::take,
    combinator::verify,
    error::{Error, ErrorKind},
    number::streaming::le_u16,
    sequence::preceded,
    Err, IResult,
};

use super::{block_header_parse, BlockHeader, CompressionType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileMetadataBlock {
    header: BlockHeader,
    data: DataBlock,
    checksum: Option<u32>,
}

impl Display for FileMetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FileMetadataBlock")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.data)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "{checksum}")?,
            None => write!(f, "No checksum")?,
        };
        Ok(())
    }
}

pub fn file_metadata_parse(input: &[u8]) -> IResult<&[u8], FileMetadataBlock> {
    let (remain, header) = preceded(
        verify(le_u16, |block_type| *block_type == 0u16),
        block_header_parse,
    )(input)?;

    let BlockHeader {
        compression_type,
        uncompressed_size,
        ..
    } = header.clone();

    // Decompress datablock
    let (remain, data_raw) = match compression_type {
        CompressionType::None => take(uncompressed_size)(remain)?,
        CompressionType::Deflate => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
    };

    let Ok((zero_block, data)) = data_parse(data_raw) else {
        return Err(Err::Error(Error::new(input, ErrorKind::Alt)));
    };

    // zero_block MUST be empty because we have .take()'n a fixed size block.
    assert!(zero_block.is_empty());

    Ok((
        remain,
        FileMetadataBlock {
            header,
            data,
            // TODO must handle checksum
            checksum: None,
        },
    ))
}
