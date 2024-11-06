use super::block_header::BlockHeader;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrinterMetadataBlock {
    header: BlockHeader,
}
