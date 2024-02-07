use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeStringAdd {
    StringString {
        left: Box<NodeString>,
        right: Box<NodeString>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeString {
    Variable { name: String },
    Constant(String),
    Add(NodeStringAdd),
}
