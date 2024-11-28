use super::compression_type::CompressionType;
use nom::{bytes::streaming::take, IResult};

pub(super) fn inflate(
    data: &[u8],
    compression_type: CompressionType,
    uncompressed_size: u32,
) -> IResult<&[u8], &[u8]> {
    // Decompress datablock
    let (remain, data_inflated) = match compression_type {
        CompressionType::None => take(uncompressed_size)(data)?,
        CompressionType::Deflate => {
            let (_remain, _data_compressed) = take(uncompressed_size)(data)?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(data)?;
            // Must decompress here
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(data)?;
            // Must decompress here
            todo!()
        }
    };

    Ok((remain, data_inflated))
}
