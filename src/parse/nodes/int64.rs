use super::uint64::NodeUint64;

#[derive(Debug)]
pub enum NodeInt64Negative<T> {
    Uint64(Box<NodeUint64<T>>),
}

#[derive(Debug)]
pub enum NodeInt64<T> {
    Variable { name: String },
    Negative(NodeInt64Negative<T>),
}
