pub mod compile;
pub mod lex;
pub mod parse;

pub use crate::compile::{compile, compile_unsafe, Engine};
pub use crate::parse::{nodes::boolean::NodeBoolean, Ast};
pub use chert_accessor::{ChertField, ChertFieldType, ChertStructTrait};
pub use chert_derive::ChertStruct;

#[derive(Debug)]
pub enum ParseError {
    Lex(crate::lex::Error),
    Parse(crate::parse::Error),
}

impl From<crate::lex::Error> for ParseError {
    fn from(value: crate::lex::Error) -> Self {
        Self::Lex(value)
    }
}

impl From<crate::parse::Error> for ParseError {
    fn from(value: crate::parse::Error) -> Self {
        Self::Parse(value)
    }
}

pub fn parse<T: ChertStructTrait>(expression: &str) -> Result<Ast<T, NodeBoolean>, ParseError> {
    let tokens = crate::lex::lex(expression)?;
    Ok(crate::parse::parse_boolean::<T>(tokens)?)
}
