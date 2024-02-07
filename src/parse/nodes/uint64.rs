use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeUint64Add {
    Uint64Uint64 {
        left: Box<NodeUint64>,
        right: Box<NodeUint64>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeUint64Subtract {
    Uint64Uint64 {
        left: Box<NodeUint64>,
        right: Box<NodeUint64>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeUint64 {
    Variable { name: String },
    Constant(u64),
    Add(NodeUint64Add),
    Subtract(NodeUint64Subtract),
}
