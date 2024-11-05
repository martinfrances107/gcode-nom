use super::BlockHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrinterMetadataBlock {
    header: BlockHeader,
}
