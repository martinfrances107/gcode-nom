use super::BlockHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SlicerMetadataBlock {
    header: BlockHeader,
}
