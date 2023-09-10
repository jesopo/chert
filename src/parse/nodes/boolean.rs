use super::cidr::NodeCidr;
use super::int64::NodeInt64;
use super::ip::NodeIp;
use super::regex::NodeRegex;
use super::string::NodeString;
use super::uint64::NodeUint64;

#[derive(Debug)]
pub enum NodeBooleanWithin<T> {
    IpCidr { left: NodeIp<T>, right: NodeCidr<T> },
}

#[derive(Debug)]
pub enum NodeBooleanNot<T> {
    Boolean(Box<NodeBoolean<T>>),
}

#[derive(Debug)]
pub enum NodeBooleanBoth<T> {
    BooleanBoolean {
        left: Box<NodeBoolean<T>>,
        right: Box<NodeBoolean<T>>,
    },
}

#[derive(Debug)]
pub enum NodeBooleanEither<T> {
    BooleanBoolean {
        left: Box<NodeBoolean<T>>,
        right: Box<NodeBoolean<T>>,
    },
}

#[derive(Debug)]
pub enum NodeBooleanEquals<T> {
    BooleanBoolean {
        left: Box<NodeBoolean<T>>,
        right: Box<NodeBoolean<T>>,
    },
    StringString {
        left: NodeString<T>,
        right: NodeString<T>,
    },
    Uint64Uint64 {
        left: NodeUint64<T>,
        right: NodeUint64<T>,
    },
    Int64Int64 {
        left: NodeInt64<T>,
        right: NodeInt64<T>,
    },
    IpIp {
        left: NodeIp<T>,
        right: NodeIp<T>,
    },
}

#[derive(Debug)]
pub enum NodeBooleanMatches<T> {
    StringRegex {
        left: NodeString<T>,
        right: NodeRegex<T>,
    },
}

#[derive(Debug)]
pub enum NodeBoolean<T> {
    Variable { name: String },
    Constant(bool),
    Not(NodeBooleanNot<T>),
    Both(NodeBooleanBoth<T>),
    Either(NodeBooleanEither<T>),
    Within(NodeBooleanWithin<T>),
    Equals(NodeBooleanEquals<T>),
    Matches(NodeBooleanMatches<T>),
}
