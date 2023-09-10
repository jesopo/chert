use super::nodes::boolean::{
    NodeBoolean, NodeBooleanBoth, NodeBooleanEither, NodeBooleanEquals, NodeBooleanMatches,
    NodeBooleanNot, NodeBooleanWithin,
};
use super::nodes::int64::{NodeInt64, NodeInt64Negative};
use super::nodes::string::{NodeString, NodeStringAdd};
use super::nodes::uint64::{NodeUint64, NodeUint64Add, NodeUint64Subtract};
use super::nodes::Node;

#[derive(Debug)]
pub enum UnaryOperator {
    Negative,
    Not,
    Positive,
}

#[derive(Debug)]
pub enum BinaryOperator {
    Exponent,
    Multiply,
    Divide,
    Modulo,
    Both,
    Either,
    Add,
    Subtract,
    Within,
    Equals,
    Matches,
}

#[derive(Debug)]
pub enum ScopeOperator {
    Open(char),
    Close,
}

#[derive(Debug)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
    Scope(ScopeOperator),
}

#[derive(Debug)]
pub enum Error {
    UnknownOperator,
    BadOperands,
}

impl UnaryOperator {
    pub fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "!" => Self::Not,
            "+" => Self::Positive,
            "-" => Self::Negative,
            _ => {
                return None;
            }
        })
    }

    pub fn to_node<T: std::fmt::Debug>(&self, node: Node<T>) -> Result<Node<T>, Node<T>> {
        Ok(match self {
            Self::Not => match node {
                Node::Boolean(node) => {
                    Node::Boolean(NodeBoolean::Not(NodeBooleanNot::Boolean(Box::new(node))))
                }
                _ => {
                    return Err(node);
                }
            },
            Self::Positive => match node {
                Node::Uint64(node) => Node::Uint64(node),
                _ => {
                    return Err(node);
                }
            },
            Self::Negative => match node {
                Node::Uint64(node) => Node::Int64(NodeInt64::Negative(NodeInt64Negative::Uint64(
                    Box::new(node),
                ))),
                _ => {
                    return Err(node);
                }
            },
        })
    }
}

impl BinaryOperator {
    pub fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "**" => Self::Exponent,
            "&&" => Self::Both,
            "||" => Self::Either,
            "==" => Self::Equals,
            "+" => Self::Add,
            "-" => Self::Subtract,
            "~" => Self::Matches,
            _ => {
                return None;
            }
        })
    }

    pub fn to_node<T: std::fmt::Debug>(
        &self,
        left: Node<T>,
        right: Node<T>,
    ) -> Result<Node<T>, (Node<T>, Node<T>)> {
        Ok(match self {
            Self::Both => match (left, right) {
                (Node::Boolean(left), Node::Boolean(right)) => {
                    Node::Boolean(NodeBoolean::Both(NodeBooleanBoth::BooleanBoolean {
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Either => match (left, right) {
                (Node::Boolean(left), Node::Boolean(right)) => {
                    Node::Boolean(NodeBoolean::Either(NodeBooleanEither::BooleanBoolean {
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Equals => match (left, right) {
                (Node::Uint64(left), Node::Uint64(right)) => {
                    Node::Boolean(NodeBoolean::Equals(NodeBooleanEquals::Uint64Uint64 {
                        left,
                        right,
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Add => match (left, right) {
                (Node::String(left), Node::String(right)) => {
                    Node::String(NodeString::Add(NodeStringAdd::StringString {
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                }
                (Node::Uint64(left), Node::Uint64(right)) => {
                    Node::Uint64(NodeUint64::Add(NodeUint64Add::Uint64Uint64 {
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Subtract => match (left, right) {
                (Node::Uint64(left), Node::Uint64(right)) => {
                    Node::Uint64(NodeUint64::Subtract(NodeUint64Subtract::Uint64Uint64 {
                        left: Box::new(left),
                        right: Box::new(right),
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Within => match (left, right) {
                (Node::Ip(left), Node::Cidr(right)) => {
                    Node::Boolean(NodeBoolean::Within(NodeBooleanWithin::IpCidr {
                        left,
                        right,
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            Self::Matches => match (left, right) {
                (Node::String(left), Node::Regex(right)) => {
                    Node::Boolean(NodeBoolean::Matches(NodeBooleanMatches::StringRegex {
                        left,
                        right,
                    }))
                }
                (left, right) => {
                    return Err((left, right));
                }
            },
            _ => unreachable!(),
        })
    }
}

#[derive(Eq, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

impl Operator {
    pub fn associativity(&self) -> Associativity {
        match self {
            Self::Unary(_) | Self::Scope(_) => Associativity::Right,
            Self::Binary(operator) => match operator {
                BinaryOperator::Exponent => Associativity::Right,
                _ => Associativity::Left,
            },
        }
    }

    pub fn specificity(&self) -> u8 {
        match self {
            Self::Scope(_) => 0,
            Self::Binary(operator) => match operator {
                BinaryOperator::Matches | BinaryOperator::Within => 2,
                BinaryOperator::Equals => 3,
                BinaryOperator::Either => 4,
                BinaryOperator::Both => 5,
                BinaryOperator::Add | BinaryOperator::Subtract => 6,
                BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 7,
                BinaryOperator::Exponent => 8,
            },
            Self::Unary(_) => 255,
        }
    }
}
