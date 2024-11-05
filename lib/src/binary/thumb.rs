use super::BlockHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ThumbnailBlock {
    header: BlockHeader,
}
