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
use crate::{ChertField, ChertStructTrait};
use std::ops::Range;

pub enum Keyword<T> {
    Operand(Node<T>),
    Operator(Operator),
}

fn get_keyword<T>(name: &str) -> Option<Keyword<T>> {
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
pub enum Error<T: std::fmt::Debug> {
    UnknownIdentifier(String),
    BadBinaryOperands {
        operator: BinaryOperator,
        left: Node<T>,
        right: Node<T>,
    },
    BadUnaryOperands {
        operator: UnaryOperator,
        node: Node<T>,
    },
    UnknownBinaryOperator(String),
    UnknownUnaryOperator(String),
    MissingOperand,
    Unfinished,
    Empty,
    NonexistentScopeClose,
}

// shunting yard time baby
fn pop_ops<T: std::fmt::Debug>(
    new_operator: &Operator,
    operators: &mut Vec<Operator>,
    operands: &mut Vec<Node<T>>,
) -> Result<(), Error<T>> {
    while let Some(operator) = operators.pop() {
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
                    let right = operands.pop().ok_or(Error::MissingOperand)?;
                    let left = operands.pop().ok_or(Error::MissingOperand)?;
                    operands.push(operator.to_node(left, right).map_err(|(left, right)| {
                        Error::BadBinaryOperands {
                            operator,
                            left,
                            right,
                        }
                    })?);
                }
                Operator::Unary(operator) => {
                    let node = operands.pop().ok_or(Error::MissingOperand)?;
                    operands.push(
                        operator
                            .to_node(node)
                            .map_err(|node| Error::BadUnaryOperands { operator, node })?,
                    );
                }
            };
        } else {
            operators.push(operator);
            break;
        }
    }
    Ok(())
}

pub fn parse<T: ChertStructTrait>(tokens: Vec<(Token, Range<usize>)>) -> Result<Node<T>, Error<T>> {
    let fields = T::fields();

    let mut operands = Vec::new();
    let mut operators = Vec::new();
    let mut last_was_operand = false;

    for (token, _span) in tokens {
        match token {
            Token::String(value) => {
                last_was_operand = true;
                operands.push(Node::String(NodeString::Constant(value)))
            }
            Token::Number(value) => {
                last_was_operand = true;
                operands.push(Node::Uint64(NodeUint64::Constant(value.parse().unwrap())));
            }
            Token::Ip(value) => {
                last_was_operand = true;
                operands.push(Node::Ip(NodeIp::Constant(value)));
            }
            Token::Cidr(value) => {
                last_was_operand = true;
                operands.push(Node::Cidr(NodeCidr::Constant(value)));
            }
            Token::Regex(value) => {
                last_was_operand = true;
                operands.push(Node::Regex(NodeRegex::Constant(value)));
            }
            Token::Identifier(ref name) => {
                let name = name.clone();
                if let Some(keyword) = get_keyword(&name) {
                    match keyword {
                        Keyword::Operand(operand) => {
                            last_was_operand = true;
                            operands.push(operand);
                        }
                        Keyword::Operator(operator) => {
                            last_was_operand = false;
                            operators.push(operator);
                        }
                    };
                } else if let Some((_index, field)) = fields.get(&name) {
                    last_was_operand = true;
                    operands.push(match field {
                        ChertField::Boolean(_) => Node::Boolean(NodeBoolean::Variable { name }),
                        ChertField::Cidr(_) => Node::Cidr(NodeCidr::Variable { name }),
                        ChertField::Int64(_) => Node::Int64(NodeInt64::Variable { name }),
                        ChertField::Ip(_) => Node::Ip(NodeIp::Variable { name }),
                        ChertField::String(_) => Node::String(NodeString::Variable { name }),
                        ChertField::Uint64(_) => Node::Uint64(NodeUint64::Variable { name }),
                    });
                } else {
                    return Err(Error::UnknownIdentifier(name));
                }
            }
            Token::ParenthesisOpen => {
                last_was_operand = false;
                operators.push(Operator::Scope(ScopeOperator::Open('(')));
            }
            Token::ParenthesisClose => {
                last_was_operand = true;
                pop_ops(
                    &Operator::Scope(ScopeOperator::Close),
                    &mut operators,
                    &mut operands,
                )?;
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
                operators.push(operator);
                last_was_operand = false;
            }
            Token::Space(_) => {
                // skip space
            }
            token => todo!("not implemented {token:?}"),
        };
    }

    pop_ops(
        &Operator::Scope(ScopeOperator::Close),
        &mut operators,
        &mut operands,
    )?;

    if let Some(root) = operands.pop() {
        if !operands.is_empty() {
            Err(Error::Unfinished)
        } else {
            Ok(root)
        }
    } else {
        Err(Error::Empty)
    }
}
