use super::BlockHeader;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlicerMetadataBlock {
    header: BlockHeader,
}
