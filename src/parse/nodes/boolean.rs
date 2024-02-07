use serde::{Deserialize, Serialize};

use super::cidr::NodeCidr;
use super::int64::NodeInt64;
use super::ip::NodeIp;
use super::regex::NodeRegex;
use super::string::NodeString;
use super::uint64::NodeUint64;

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanWithin {
    IpCidr { left: NodeIp, right: NodeCidr },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanNot {
    Boolean(Box<NodeBoolean>),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanBoth {
    BooleanBoolean {
        left: Box<NodeBoolean>,
        right: Box<NodeBoolean>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanEither {
    BooleanBoolean {
        left: Box<NodeBoolean>,
        right: Box<NodeBoolean>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanEquals {
    BooleanBoolean {
        left: Box<NodeBoolean>,
        right: Box<NodeBoolean>,
    },
    StringString {
        left: NodeString,
        right: NodeString,
    },
    Uint64Uint64 {
        left: NodeUint64,
        right: NodeUint64,
    },
    Int64Int64 {
        left: NodeInt64,
        right: NodeInt64,
    },
    IpIp {
        left: NodeIp,
        right: NodeIp,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBooleanMatches {
    StringRegex { left: NodeString, right: NodeRegex },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeBoolean {
    Variable { name: String },
    Constant(bool),
    Not(NodeBooleanNot),
    Both(NodeBooleanBoth),
    Either(NodeBooleanEither),
    Within(NodeBooleanWithin),
    Equals(NodeBooleanEquals),
    Matches(NodeBooleanMatches),
}
