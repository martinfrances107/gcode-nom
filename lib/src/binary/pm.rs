use super::BlockHeader;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PrinterMetadataBlock {
    header: BlockHeader,
}
