use super::uint64::NodeUint64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeInt64Negative {
    Uint64(Box<NodeUint64>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeInt64 {
    Variable { name: String },
    Negative(NodeInt64Negative),
}
