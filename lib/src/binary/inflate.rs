use nom::bytes::streaming::take;
use nom::Err::Error;
use nom::IResult;

use super::block_header::BlockHeader;
use super::BlockError;
use super::CompressionType;

use inflate::inflate_bytes_zlib;

pub(crate) fn decompress_data_block<'a>(
    header: &BlockHeader,
    data: &'a [u8],
) -> IResult<&'a [u8], Vec<u8>, BlockError> {
    // Decompress data-block
    let (after_data, data) = match header.compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(header.uncompressed_size)(data).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "Failed to extract raw(uncompressed) data block: {e:#?}"
                    ))
                })
            })?;
            (remain, data_raw.to_vec())
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(header.compressed_size.unwrap())(data).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "Compression::Deflate - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, decoded),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(Error(BlockError::Decompression(format!(
                        "Compression: Deflate - Failed to decompress: {msg}"
                    ))))?;
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(header.uncompressed_size)(data).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "Compression::HeatShrink11 - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(header.uncompressed_size)(data).map_err(|e| {
                e.map(|e: nom::error::Error<_>| {
                    BlockError::Decompression(format!(
                        "Compression::HeatShrink12 - Failed to extract compressed data block: {e:#?}"
                    ))
                })
            })?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
    };

    Ok((after_data, data))
}
