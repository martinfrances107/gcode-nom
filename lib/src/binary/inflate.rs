use super::compression_type::CompressionType;
use nom::{bytes::streaming::take, IResult};

pub(super) fn inflate(
    data: &[u8],
    compression_type: CompressionType,
    uncompressed_size: u32,
) -> IResult<&[u8], &[u8]> {
    // Decompress data block
    let (remain, data_inflated) = match compression_type {
        CompressionType::None => take(uncompressed_size)(data)?,
        CompressionType::Deflate => {
            let (remain, encoded) = take(compressed_size.unwrap())(after_param)?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => {
                    let data = String::from_utf8(decoded).expect("raw data error");
                    (remain, data)
                }
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    panic!()
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(compressed_size)(data)?;

            log::error!("TODO: Must implement HeatShrink11");
            todo!("This file contains a heatshrink11 encoded data block")
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(compressed_size)(data)?;
            log::error!("TODO: Must implement HeatShrink12");
            todo!("This file contains a heatshrink11 encoded data block")
        }
    };

    Ok((remain, data_inflated))
}
