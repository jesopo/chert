pub mod nodes;
pub mod operators;

use self::nodes::boolean::NodeBoolean;
use self::nodes::cidr::NodeCidr;
use self::nodes::int64::NodeInt64;
use self::nodes::ip::NodeIp;
use self::nodes::regex::NodeRegex;
use self::nodes::string::NodeString;
use self::nodes::uint64::NodeUint64;
use self::nodes::Node;
use self::operators::{Associativity, BinaryOperator, Operator, ScopeOperator, UnaryOperator};
use crate::lex::Token;
use crate::variables::{Variable, Variables};
use std::ops::Range;

enum Keyword {
    Operand(Node),
    Operator(Operator),
}

fn get_keyword(name: &str) -> Option<Keyword> {
    Some(match name {
        "true" => Keyword::Operand(Node::Boolean(NodeBoolean::Constant(true))),
        "false" => Keyword::Operand(Node::Boolean(NodeBoolean::Constant(false))),
        "and" => Keyword::Operator(Operator::Binary(BinaryOperator::Both)),
        "or" => Keyword::Operator(Operator::Binary(BinaryOperator::Either)),
        "in" => Keyword::Operator(Operator::Binary(BinaryOperator::Within)),
        _ => {
            return None;
        }
    })
}

#[derive(Debug)]
pub enum Error {
    UnknownIdentifier(String),
    BadBinaryOperands {
        operator: BinaryOperator,
        left: Node,
        right: Node,
    },
    BadUnaryOperands {
        operator: UnaryOperator,
        node: Node,
    },
    UnknownBinaryOperator(String),
    UnknownUnaryOperator(String),
    MissingOperand,
    Unfinished,
    Empty,
    NonexistentScopeClose,
    NotBoolean,
}

// shunting yard time baby
fn pop_ops(
    new_operator: &Operator,
    operators: &mut Vec<(Operator, Range<usize>)>,
    operands: &mut Vec<(Node, Range<usize>)>,
) -> Result<(), Error> {
    while let Some((operator, span)) = operators.pop() {
        if match operator.associativity() {
            Associativity::Left => operator.specificity() >= new_operator.specificity(),
            Associativity::Right => operator.specificity() > new_operator.specificity(),
        } {
            match operator {
                Operator::Scope(scope) => match scope {
                    ScopeOperator::Open(_char) => {
                        break;
                    }
                    ScopeOperator::Close => {
                        return Err(Error::NonexistentScopeClose);
                    }
                },
                Operator::Binary(operator) => {
                    let (right, _right_span) = operands.pop().ok_or(Error::MissingOperand)?;
                    let (left, _left_span) = operands.pop().ok_or(Error::MissingOperand)?;
                    operands.push((
                        operator.to_node(left, right).map_err(|(left, right)| {
                            Error::BadBinaryOperands {
                                operator,
                                left,
                                right,
                            }
                        })?,
                        span,
                    ));
                }
                Operator::Unary(operator) => {
                    let (node, _node_span) = operands.pop().ok_or(Error::MissingOperand)?;
                    operands.push((
                        operator
                            .to_node(node)
                            .map_err(|node| Error::BadUnaryOperands { operator, node })?,
                        span,
                    ));
                }
            };
        } else {
            operators.push((operator, span));
            break;
        }
    }
    Ok(())
}

fn parse_inner<T: Variables>(tokens: Vec<(Token, Range<usize>)>) -> Result<Node, Error> {
    let fields = T::variables();

    let mut operands = Vec::new();
    let mut operators = Vec::new();
    let mut last_was_operand = false;

    for (token, span) in tokens {
        let operand = match token {
            Token::String(value) => Some(Node::String(NodeString::Constant(value))),
            Token::Number(value) => {
                Some(Node::Uint64(NodeUint64::Constant(value.parse().unwrap())))
            }
            Token::Ip(value) => Some(Node::Ip(NodeIp::Constant(value))),
            Token::Cidr(value) => Some(Node::Cidr(NodeCidr::Constant(value))),
            Token::Regex(value) => Some(Node::Regex(NodeRegex::Constant(value))),
            Token::Identifier(ref name) => {
                let name = name.clone();
                let ret = if let Some(keyword) = get_keyword(&name) {
                    match keyword {
                        Keyword::Operand(operand) => Some(operand),
                        Keyword::Operator(operator) => {
                            pop_ops(&operator, &mut operators, &mut operands)?;
                            operators.push((operator, span.clone()));
                            last_was_operand = false;
                            None
                        }
                    }
                } else if let Some((_index, field)) = fields.get(&name) {
                    Some(match field {
                        Variable::Boolean(_) => Node::Boolean(NodeBoolean::Variable { name }),
                        Variable::Cidr(_) => Node::Cidr(NodeCidr::Variable { name }),
                        Variable::Int64(_) => Node::Int64(NodeInt64::Variable { name }),
                        Variable::Ip(_) => Node::Ip(NodeIp::Variable { name }),
                        Variable::String(_) => Node::String(NodeString::Variable { name }),
                        Variable::Uint64(_) => Node::Uint64(NodeUint64::Variable { name }),
                        Variable::Regex(_) => Node::Regex(NodeRegex::Variable { name }),
                    })
                } else {
                    return Err(Error::UnknownIdentifier(name));
                };
                ret
            }
            Token::ParenthesisOpen => {
                last_was_operand = false;
                operators.push((Operator::Scope(ScopeOperator::Open('(')), span.clone()));
                None
            }
            Token::ParenthesisClose => {
                last_was_operand = true;
                pop_ops(
                    &Operator::Scope(ScopeOperator::Close),
                    &mut operators,
                    &mut operands,
                )?;
                None
            }
            Token::Operator(operator) => {
                let operator = if last_was_operand {
                    Operator::Binary(
                        BinaryOperator::parse(operator.as_str())
                            .ok_or_else(|| Error::UnknownBinaryOperator(operator))?,
                    )
                } else {
                    Operator::Unary(
                        UnaryOperator::parse(operator.as_str())
                            .ok_or_else(|| Error::UnknownUnaryOperator(operator))?,
                    )
                };
                pop_ops(&operator, &mut operators, &mut operands)?;
                operators.push((operator, span.clone()));
                last_was_operand = false;
                None
            }
            Token::Space(_) => None,
            token => todo!("not implemented {token:?}"),
        };
        if let Some(operand) = operand {
            last_was_operand = true;
            operands.push((operand, span));
        }
    }

    pop_ops(
        &Operator::Scope(ScopeOperator::Close),
        &mut operators,
        &mut operands,
    )?;

    if let Some((root, _span)) = operands.pop() {
        if !operands.is_empty() {
            Err(Error::Unfinished)
        } else {
            Ok(root)
        }
    } else {
        Err(Error::Empty)
    }
}

#[derive(Debug)]
pub struct Ast<T, R> {
    pub(crate) root: R,
    _type: Option<T>,
}

impl<T, R> Ast<T, R> {
    pub fn get_root(&self) -> &R {
        &self.root
    }
    pub fn into_root(self) -> R {
        self.root
    }
}

pub fn parse<T: Variables>(tokens: Vec<(Token, Range<usize>)>) -> Result<Ast<T, Node>, Error> {
    Ok(Ast {
        root: parse_inner::<T>(tokens)?,
        _type: None,
    })
}

pub fn parse_boolean<T: Variables>(
    tokens: Vec<(Token, Range<usize>)>,
) -> Result<Ast<T, NodeBoolean>, Error> {
    let root = parse_inner::<T>(tokens)?;
    if let Node::Boolean(root) = root {
        Ok(Ast { root, _type: None })
    } else {
        Err(Error::NotBoolean)
    }
}
