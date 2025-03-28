use std::sync::LazyLock;

use heatshrink::decode;
use heatshrink::Config;
use meatpack::MeatPackResult;
use meatpack::Unpacker;
use nom::bytes::streaming::take;
use nom::Err::Error;
use nom::IResult;

use super::block_header::BlockHeader;
use super::default_params::Encoding;
use super::BlockError;
use super::CompressionType;

use inflate::inflate_bytes_zlib;

static CONFIG_W12_L4: LazyLock<Config> =
    LazyLock::new(|| Config::new(12, 4).expect("Failed to configure HeatshrinkW11L4 decoder"));

pub fn decompress_data_block<'a>(
    data: &'a [u8],
    encoding: &Encoding,
    header: &BlockHeader,
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
            let (remain, encoded) = take::<_, _, BlockError>(header.compressed_size.unwrap())(data)
                .map_err(|e| {
                    e.map(|e| {
                        BlockError::Decompression(format!(
                            "gcode_block: HeatShrink12 - Failed to extract raw data: {e:?}"
                        ))
                    })
                })?;

            // TODO Figure out why size is is off by 1 -  crashes with buffer was not large enough.
            let mut scratch = vec![0u8; 1 + header.uncompressed_size as usize];

            let data = match decode(encoded, &mut scratch, &CONFIG_W12_L4) {
                Ok(decoded_hs) => match encoding {
                    Encoding::None => scratch,
                    Encoding::MeatPackAlgorithm => {
                        log::error!("Must decode with standard meat packing algorithm");
                        unimplemented!("Decoding with the meatpacking algorithm is not yet support please create an issue.");
                    }
                    Encoding::MeatPackModifiedAlgorithm => {
                        let mut data = vec![];
                        let mut unpacker = Unpacker::<64>::default();
                        for b in decoded_hs {
                            match unpacker.unpack(b) {
                                Ok(MeatPackResult::WaitingForNextByte) => {
                                    // absorb byte and continue
                                }
                                Ok(MeatPackResult::Line(line)) => {
                                    data.extend_from_slice(line);
                                }
                                Err(e) => {
                                    let msg = format!("Failed running the deflate MeatPackModifiedAlgorithm 'unpack()' algorithm {e:?}");
                                    log::error!("{msg}");
                                    return Err(nom::Err::Error(BlockError::Decompression(msg)));
                                }
                            }
                        }
                        data
                    }
                },
                Err(e) => {
                    let msg = format!("GCodeBlock:  Failed running the deflate MeatPackModifiedAlgorithm 'decode()' algorithm {e:?}");
                    log::error!("{msg}");
                    return Err(nom::Err::Error(BlockError::Decompression(msg)));
                }
            };

            (remain, data)
        }
    };

    Ok((after_data, data))
}
