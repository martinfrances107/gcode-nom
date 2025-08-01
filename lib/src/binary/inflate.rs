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

/// Return type for `decompress_data_block`.
#[derive(Debug)]
pub enum DecompressError {
    /// Unexpected length while taking data.
    None,
    /// Error decompressing, with the "`Deflate`" algorithm.
    Deflate,
    /// Error decompressing, with the "`HeatShrink11`" algorithm.
    HeatShrink11,
    /// Error decompressing, with the "`HeatShrink12`" algorithm.
    HeatShrink12,
    /// Error decompressing, with the  "`MeatPackAlgorithm`" algorithm.
    MeatPackAlgorithm,
}

/// Decompresses the data block
///
/// Using the appropriate decompression algorithm and encoding type.
///
/// # Panics
///  Some decompression algorithms are unimplemented
///
/// # Errors
///
/// When matching fails.
pub fn decompress_data_block<'a>(
    data: &'a [u8],
    encoding: &Encoding,
    header: &BlockHeader,
) -> IResult<&'a [u8], Vec<u8>, DecompressError> {
    // Decompress data-block
    let (after_data, data) = match header.compression_type {
        CompressionType::None => {
            let (remain, data_raw) = take(header.uncompressed_size)(data)
                .map_err(|e| e.map(|_e: nom::error::Error<_>| DecompressError::None))?;
            (remain, data_raw.to_vec())
        }
        CompressionType::Deflate => {
            let (remain, encoded) = take(header.compressed_size.unwrap())(data)
                .map_err(|e| e.map(|_e: nom::error::Error<_>| DecompressError::Deflate))?;

            match inflate_bytes_zlib(encoded) {
                Ok(decoded) => (remain, decoded),
                Err(msg) => {
                    log::error!("Failed to decode decompression failed {msg}");
                    return Err(Error(DecompressError::Deflate))?;
                }
            }
        }
        CompressionType::HeatShrink11 => {
            let (_remain, _data_compressed) = take(header.uncompressed_size)(data)
                .map_err(|e| e.map(|_e: nom::error::Error<_>| DecompressError::HeatShrink11))?;
            // Must decompress here
            log::info!("TODO: Must implement decompression");
            todo!()
        }
        CompressionType::HeatShrink12 => {
            let (remain, encoded) = take::<_, _, BlockError>(header.compressed_size.unwrap())(data)
                .map_err(|e| e.map(|_e| DecompressError::HeatShrink12))?;

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
                                Err(_e) => {
                                    // let msg = format!("Failed running the deflate MeatPackModifiedAlgorithm 'unpack()' algorithm {e:?}");
                                    // log::error!("{msg}");
                                    return Err(nom::Err::Error(
                                        DecompressError::MeatPackAlgorithm,
                                    ));
                                }
                            }
                        }
                        data
                    }
                },
                Err(_e) => {
                    // let msg = format!("GCodeBlock:  Failed running the deflate MeatPackModifiedAlgorithm 'decode()' algorithm {e:?}");
                    // log::error!("{msg}");
                    return Err(nom::Err::Error(DecompressError::MeatPackAlgorithm));
                }
            };

            (remain, data)
        }
    };

    Ok((after_data, data))
}
