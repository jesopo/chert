use regex::Regex;

#[derive(Debug)]
pub enum NodeRegex<T> {
    Variable { name: String },
    Constant(Regex),
    _Phantom(T),
}
