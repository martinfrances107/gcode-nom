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
/// <https://github.com/prusa3d/libbgcode/blob/main/doc/specifications.md#block-header>
///
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct BlockHeader {
    // block_type: u16,
    // Compression algorithm
    pub(super) compression_type: CompressionType,
    // Size of data when uncompressed
    pub(super) uncompressed_size: u32,
    // Size of data when compressed
    pub(super) compressed_size: u32,
}

pub(super) fn block_header_parser(input: &[u8]) -> IResult<&[u8], BlockHeader> {
    map(
        tuple((compression_parser, le_u32, le_u32)),
        |(compression_type, uncompressed_size, compressed_size)| {
            //ehe
            BlockHeader {
                compression_type,
                uncompressed_size,
                compressed_size,
            }
        },
    )(input)
}
