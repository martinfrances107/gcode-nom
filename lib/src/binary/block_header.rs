use nom::{combinator::map, number::streaming::le_u32, sequence::tuple, IResult};

use super::compression_type::compression_parser;
use super::compression_type::CompressionType;

/// Block header
///
/// The block header is the same for all blocks. It is defined as:
///
/// type                 size  description
/// Type                 u16   2 bytes Block type
/// Compression          u16   2 bytes Compression algorithm
/// Uncompressed size    u32   4 bytes Size of the data when uncompressed
/// Compressed size      u32   4 bytes Size of the data when compressed
///
/// The size in bytes of the block header is 8 when Compression = 0 and 12 in all other cases.
///
/// <https://github.com/prusa3d/libbgcode/blob/main/doc/specifications.md#block-header>
///
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BlockHeader {
    // Compression algorithm
    pub(super) compression_type: CompressionType,
    // Size of data when uncompressed
    pub(super) uncompressed_size: u32,
    // Size of data when compressed
    pub(super) compressed_size: Option<u32>,
}

// In memory "Block Header" is a variable sized structure.
//
/// "The size in bytes of the block header is 8 when Compression = 0 and 12 in all other cases."
pub(super) fn block_header_parser(input: &[u8]) -> IResult<&[u8], BlockHeader> {
    let (remain, compression_type) = compression_parser(input)?;
    if compression_type == CompressionType::None {
        map(le_u32, |uncompressed_size| {
            // hh
            BlockHeader {
                compression_type: CompressionType::None,
                uncompressed_size,
                compressed_size: None,
            }
        })(remain)
    } else {
        map(
            tuple((le_u32, le_u32)),
            |(uncompressed_size, compressed_size)| {
                // hh
                BlockHeader {
                    compression_type: compression_type.clone(),
                    uncompressed_size,
                    compressed_size: Some(compressed_size),
                }
            },
        )(remain)
    }
}

// Utils used for CRC checking.
impl BlockHeader {
    /// The size in bytes of the block header is 8 when Compression = 0 and 12 in all other cases.
    pub(super) const fn size_in_bytes(&self) -> usize {
        match self.compression_type {
            CompressionType::None => 8,
            _ => 12,
        }
    }

    pub(super) const fn payload_size_in_bytes(&self) -> usize {
        match self.compressed_size {
            Some(size) => size as usize,
            None => self.uncompressed_size as usize,
        }
    }
}
