use core::fmt::Display;

use nom::{
    bytes::streaming::take,
    combinator::{map_res, verify},
    error::Error,
    number::streaming::le_u16,
    sequence::preceded,
    IResult,
};

use super::{block_header_parse, BlockHeader, Compression};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileMetadataBlock {
    header: BlockHeader,
    data: DataBlock,
    checksum: Option<u32>,
}

impl Display for FileMetadataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "File Header")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.data)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "{}", checksum)?,
            None => write!(f, "No checksum")?,
        };
        Ok(())
    }
}

pub(crate) fn file_metadata_parse(input: &[u8]) -> IResult<&[u8], FileMetadataBlock> {
    let (remain, header) = preceded(
        verify(le_u16, |block_type| *block_type == 0u16),
        block_header_parse,
    )(input)?;

    let BlockHeader {
        compression,
        uncompressed_size,
        ..
    } = header.clone();

    // Decompress datablock
    let (remain, data_raw) = match compression {
        Compression::None => take(uncompressed_size)(remain)?,
        Compression::Deflate => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
        Compression::HeatShrink11 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
        Compression::HeatShrink12 => {
            let (_remain, _data_compressed) = take(uncompressed_size)(remain)?;
            // Must decompress here
            todo!()
        }
    };

    let data: DataBlock = match data_parse(data_raw) {
        Ok((r, db)) => db,
        _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    };

    Ok((
        remain,
        FileMetadataBlock {
            header,
            data,
            checksum: todo!(),
        },
    ))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DataBlock(Parameter);

fn data_parse(input: &[u8]) -> IResult<&[u8], DataBlock> {
    match parameters_parse(input) {
        Ok((r, parameter)) => Ok((r, DataBlock(parameter))),
        _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

impl Display for DataBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Block")?;
        writeln!(f,)?;
        writeln!(f, "{}", self.0)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Parameter {
    Encoding(EncodingVal),
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Parameter::Encoding(encoding_val) => {
                writeln!(f, "Paramter: Encoding {}", encoding_val)?;
            }
        }

        Ok(())
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Self::Encoding(EncodingVal::Ini)
    }
}
fn parameters_parse(input: &[u8]) -> IResult<&[u8], Parameter> {
    match encoding_parse(input) {
        Ok((r, encoding_value)) => Ok((r, Parameter::Encoding(encoding_value))),
        Err(_) => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum EncodingVal {
    #[default]
    Ini = 0,
}

impl Display for EncodingVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &EncodingVal::Ini => {
                writeln!(f, "Ini")?;
            }
        }

        Ok(())
    }
}
use nom::error::ErrorKind;
use nom::Err;

fn encoding_parse(input: &[u8]) -> IResult<&[u8], EncodingVal> {
    map_res(le_u16, |encoding_val| {
        // header
        match encoding_val {
            1 => Ok(EncodingVal::Ini),
            _bad_encoding => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        }
    })(input)
}
