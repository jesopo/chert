use regex::Regex;

#[derive(Debug)]
pub enum NodeRegex<T> {
    Constant(Regex),
    _Phantom(T),
}
