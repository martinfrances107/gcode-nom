use core::fmt::Display;

use super::{
    block_header::{block_header_parser, BlockHeader},
    default_params::{param_parser, Param},
    inflate::decompress_data_block,
    BlockError,
};

use nom::{
    bytes::streaming::take,
    combinator::verify,
    number::streaming::{le_u16, le_u32},
    sequence::preceded,
    IResult, Parser,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterMetadataBlock<'a> {
    header: BlockHeader,
    param: Param,
    data: &'a [u8],
    checksum: Option<u32>,
}
impl Display for PrinterMetadataBlock<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let datablock: String =
            match decompress_data_block(self.data, &self.param.encoding, &self.header) {
                Ok((_remain, data)) => String::from_utf8_lossy(&data).to_string(),
                Err(_e) => String::from("failed to decompress"),
            };
        writeln!(
            f,
            "-------------------------- PrinterMetadataBlock --------------------------"
        )?;
        writeln!(f, "Params")?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f, "DataBlock {datablock}")?;
        writeln!(f)?;
        write!(f, "-------------------------- PrinterMetadataBlock ")?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X} ---------")?,
            None => writeln!(f, "No checksum")?,
        }
        Ok(())
    }
}

impl PrinterMetadataBlock<'_> {
    /// Write to formatter a markdown block.
    pub fn markdown<W>(&self, mut f: W) -> core::fmt::Result
    where
        W: std::fmt::Write,
    {
        let datablock: String =
            match decompress_data_block(self.data, &self.param.encoding, &self.header) {
                Ok((_remain, data)) => String::from_utf8_lossy(&data).to_string(),
                Err(_e) => String::from("failed to decompress"),
            };
        writeln!(f)?;
        writeln!(f, "## PrinterMetadataBlock")?;
        writeln!(f)?;
        writeln!(f, "### Params")?;
        writeln!(f)?;
        writeln!(f, "params {:#?}", self.param)?;
        writeln!(f, "<details>")?;
        writeln!(f, "<summary>DataBlock</summary>")?;
        writeln!(f, "<br>")?;
        writeln!(f, "{datablock}")?;
        writeln!(f, "</details>")?;
        writeln!(f)?;
        match self.checksum {
            Some(checksum) => writeln!(f, "Checksum Ox{checksum:X}")?,
            None => writeln!(f, "No checksum")?,
        }
        Ok(())
    }
}

static PRINTER_METADATA_BLOCK_ID: u16 = 3u16;
pub fn printer_metadata_parser(
    input: &[u8],
) -> IResult<&[u8], PrinterMetadataBlock<'_>, BlockError> {
    let (after_block_header, header) = preceded(
        verify(le_u16, |block_type| {
            log::debug!(
                "Looking for PRINTER_METADATA_BLOCK_ID {PRINTER_METADATA_BLOCK_ID} {block_type} cond {}",
                *block_type == PRINTER_METADATA_BLOCK_ID
            );
            *block_type == PRINTER_METADATA_BLOCK_ID
        }),
        block_header_parser,
    ).parse(input)
    .map_err(|e| {
        e.map(|_e| BlockError::PrinterMetaData)
    })?;

    log::info!("Found printer metadata block id.");

    let (after_param, param) =
        param_parser(after_block_header).map_err(|e| e.map(|_e| BlockError::PrinterMetaData))?;

    // Decompress data block
    let (after_data, data) = match header.compressed_size {
        Some(size) => take(size)(after_param)?,
        None => take(header.uncompressed_size)(after_param)?,
    };

    let (after_checksum, checksum) = le_u32(after_data)
        .map_err(|e| e.map(|_e: nom::error::Error<_>| BlockError::PrinterMetaData))?;

    Ok((
        after_checksum,
        PrinterMetadataBlock {
            header,
            param,
            data,
            checksum: Some(checksum),
        },
    ))
}

pub fn printer_metadata_parser_with_checksum(
    input: &[u8],
) -> IResult<&[u8], PrinterMetadataBlock<'_>, BlockError> {
    let (remain, pm) = printer_metadata_parser(input)?;
    if let Some(checksum) = pm.checksum {
        let param_size = 2;
        let block_size = pm.header.size_in_bytes() + param_size + pm.header.payload_size_in_bytes();
        let crc_input = &input[..block_size];
        let computed_checksum = crc32fast::hash(crc_input);

        log::debug!(
            "slicer checksum 0x{checksum:04x} computed checksum 0x{computed_checksum:04x} "
        );
        if checksum == computed_checksum {
            log::debug!("checksum match");
        } else {
            log::error!("fail checksum");
            return Err(nom::Err::Error(BlockError::PrinterMetaData));
        }
    }

    Ok((remain, pm))
}
