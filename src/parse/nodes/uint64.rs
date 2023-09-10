#[derive(Debug)]
pub enum NodeUint64Add<T> {
    Uint64Uint64 {
        left: Box<NodeUint64<T>>,
        right: Box<NodeUint64<T>>,
    },
}

#[derive(Debug)]
pub enum NodeUint64Subtract<T> {
    Uint64Uint64 {
        left: Box<NodeUint64<T>>,
        right: Box<NodeUint64<T>>,
    },
}

#[derive(Debug)]
pub enum NodeUint64<T> {
    Variable { name: String },
    Constant(u64),
    Add(NodeUint64Add<T>),
    Subtract(NodeUint64Subtract<T>),
    _Phantom(T),
}
