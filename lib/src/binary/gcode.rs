use super::BlockHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GCodeBlock {
    header: BlockHeader,
}
