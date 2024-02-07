pub mod compile;
pub mod lex;
pub mod parse;

pub use crate::compile::{compile, Engine};
pub use chert_accessor::{ChertField, ChertStructTrait};
pub use chert_derive::ChertStruct;

#[derive(Debug)]
pub enum ParseError<T: std::fmt::Debug> {
    Lex(crate::lex::Error),
    Parse(crate::parse::Error<T>),
}

impl<T: std::fmt::Debug> From<crate::lex::Error> for ParseError<T> {
    fn from(value: crate::lex::Error) -> Self {
        Self::Lex(value)
    }
}

impl<T: std::fmt::Debug> From<crate::parse::Error<T>> for ParseError<T> {
    fn from(value: crate::parse::Error<T>) -> Self {
        Self::Parse(value)
    }
}

pub fn parse<T: ChertStructTrait>(
    expression: &str,
) -> Result<crate::parse::nodes::Node<T>, ParseError<T>> {
    let tokens = crate::lex::lex(expression)?;
    Ok(crate::parse::parse(tokens)?)
}
