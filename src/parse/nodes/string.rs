#[derive(Debug)]
pub enum NodeStringAdd<T> {
    StringString {
        left: Box<NodeString<T>>,
        right: Box<NodeString<T>>,
    },
}

#[derive(Debug)]
pub enum NodeString<T> {
    Variable { name: String },
    Constant(String),
    Add(NodeStringAdd<T>),
    _Phantom(T),
}
