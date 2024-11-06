use super::block_header::BlockHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GCodeBlock {
    header: BlockHeader,
}
