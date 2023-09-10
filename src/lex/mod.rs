mod util;

use cidr::errors::NetworkParseError;
use cidr::IpCidr;
use logos::Logos;
use regex::{Error as RegexError, Regex};
use std::net::{AddrParseError, IpAddr};
use std::str::FromStr as _;

#[derive(Clone, Debug, Default)]
pub enum Error {
    #[default]
    BadSyntax,
    AddrParseError(AddrParseError),
    NetworkParseError(NetworkParseError),
    Custom(String),
    Unfinished,
    Regex(regex::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        // this is bad, don't do this
        format!("{self:?}") == format!("{other:?}")
    }
}

impl From<AddrParseError> for Error {
    fn from(error: AddrParseError) -> Self {
        Self::AddrParseError(error)
    }
}

impl From<RegexError> for Error {
    fn from(error: RegexError) -> Self {
        Self::Regex(error)
    }
}

impl From<NetworkParseError> for Error {
    fn from(error: NetworkParseError) -> Self {
        Self::NetworkParseError(error)
    }
}

#[derive(Debug, Logos)]
#[logos(error = Error)]
pub enum Token {
    #[token("(")]
    ParenthesisOpen,
    #[token(")")]
    ParenthesisClose,
    #[regex(r"(\d+w)?(\d+d)?(\d+h)?(\d+m)?(\d+s)?", util::parse_duration)]
    Duration(u64),
    #[regex("[a-z][a-zA-Z0-9]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex(r"(\d{1,3}\.){3}\d{1,3}", |lex| IpAddr::from_str(lex.slice()))]
    Ip(IpAddr),
    #[regex(r"(\d{1,3}\.){3}\d{1,3}/\d{1,2}", |lex| IpCidr::from_str(lex.slice()))]
    Cidr(IpCidr),
    #[regex(r"(\d+)?\.?\d+", |lex| lex.slice().to_owned())]
    Number(String),
    #[regex(r"&&|[||]{2}|==|[+]|-|~", |lex| lex.slice().to_owned())]
    Operator(String),
    #[regex(r"m[^\w\s]", |lex| util::compile_regex(lex))]
    Regex(Regex),
    #[regex("[ ]+", |lex| lex.slice().to_owned())]
    Space(String),
    #[regex("'|\"", util::find_closing)]
    String(String),
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        // this is bad, don't do this
        format!("{self:?}") == format!("{other:?}")
    }
}

pub fn lex(expression: &str) -> Result<Vec<(Token, std::ops::Range<usize>)>, Error> {
    let lexer = Token::lexer(expression);
    let mut tokens = Vec::new();

    for (token, span) in lexer.spanned() {
        tokens.push((token?, span));
    }

    Ok(tokens)
}
