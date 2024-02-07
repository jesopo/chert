use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeUint64Add {
    Uint64Uint64 {
        left: Box<NodeUint64>,
        right: Box<NodeUint64>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeUint64Subtract {
    Uint64Uint64 {
        left: Box<NodeUint64>,
        right: Box<NodeUint64>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeUint64 {
    Variable { name: String },
    Constant(u64),
    Add(NodeUint64Add),
    Subtract(NodeUint64Subtract),
}
