use crate::parse::nodes::boolean::{
    NodeBoolean, NodeBooleanBoth, NodeBooleanEither, NodeBooleanEquals, NodeBooleanMatches,
    NodeBooleanNot, NodeBooleanWithin,
};
use crate::parse::nodes::cidr::NodeCidr;
use crate::parse::nodes::int64::{NodeInt64, NodeInt64Negative};
use crate::parse::nodes::ip::NodeIp;
use crate::parse::nodes::regex::NodeRegex;
use crate::parse::nodes::string::{NodeString, NodeStringAdd};
use crate::parse::nodes::uint64::{NodeUint64, NodeUint64Add, NodeUint64Subtract};

use chert_accessor::{ChertField, ChertStruct};
use cidr::{IpCidr, Ipv4Cidr};
use regex::Regex;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Default)]
pub struct Heaps {
    pub boolean: Vec<bool>,
    pub cidr: Vec<IpCidr>,
    pub int64: Vec<i64>,
    pub ip: Vec<IpAddr>,
    pub string: Vec<String>,
    pub uint64: Vec<u64>,
    pub regex: Vec<Regex>,
}

#[derive(Debug)]
pub enum Operation {
    AddStringString { left: usize, right: usize },
    AddUint64Uint64 { left: usize, right: usize },
    BothBoolBool { left: usize, right: usize },
    EitherBoolBool { left: usize, right: usize },
    EqualsBoolBool { left: usize, right: usize },
    EqualsStringString { left: usize, right: usize },
    EqualsUint64Uint64 { left: usize, right: usize },
    EqualsInt64Int64 { left: usize, right: usize },
    NegativeUint64(usize),
    NotBool(usize),
    SubtractUint64Uint64 { left: usize, right: usize },
    WithinIpCidr { left: usize, right: usize },
    MatchesStringRegex { left: usize, right: usize },
}

fn compile_ip<T>(
    node: NodeIp<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
) -> usize {
    match node {
        NodeIp::_Phantom(_) => unreachable!(),
        NodeIp::Constant(value) => {
            heaps.ip.push(value);
            heaps.ip.len() - 1
        }
        NodeIp::Variable { name } => {
            if let Some((index, ChertField::Ip(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_cidr<T>(
    node: NodeCidr<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
) -> usize {
    match node {
        NodeCidr::_Phantom(_) => unreachable!(),
        NodeCidr::Constant(value) => {
            heaps.cidr.push(value);
            heaps.cidr.len() - 1
        }
        NodeCidr::Variable { name } => {
            if let Some((index, ChertField::Cidr(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_boolean<T>(
    node: NodeBoolean<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
    operations: &mut Vec<(usize, Operation)>,
) -> usize {
    match node {
        NodeBoolean::Constant(value) => {
            heaps.boolean.push(value);
            heaps.boolean.len() - 1
        }
        NodeBoolean::Variable { name } => {
            if let Some((index, ChertField::Boolean(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
        NodeBoolean::Not(node) => match node {
            NodeBooleanNot::Boolean(node) => {
                let child = compile_boolean(*node, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::NotBool(child)));
                index
            }
        },
        NodeBoolean::Both(node) => match node {
            NodeBooleanBoth::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, heaps, operations);
                let right = compile_boolean(*right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::BothBoolBool { left, right }));
                index
            }
        },
        NodeBoolean::Either(node) => match node {
            NodeBooleanEither::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, heaps, operations);
                let right = compile_boolean(*right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::EitherBoolBool { left, right }));
                index
            }
        },
        NodeBoolean::Within(node) => match node {
            NodeBooleanWithin::IpCidr { left, right } => {
                let left = compile_ip(left, fields, heaps);
                let right = compile_cidr(right, fields, heaps);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::WithinIpCidr { left, right }));
                index
            }
        },
        NodeBoolean::Equals(node) => match node {
            NodeBooleanEquals::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, heaps, operations);
                let right = compile_boolean(*right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::EqualsBoolBool { left, right }));
                index
            }
            NodeBooleanEquals::StringString { left, right } => {
                let left = compile_string(left, fields, heaps, operations);
                let right = compile_string(right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::EqualsStringString { left, right }));
                index
            }
            NodeBooleanEquals::Uint64Uint64 { left, right } => {
                let left = compile_uint64(left, fields, heaps, operations);
                let right = compile_uint64(right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::EqualsUint64Uint64 { left, right }));
                index
            }
            NodeBooleanEquals::Int64Int64 { left, right } => {
                let left = compile_int64(left, fields, heaps, operations);
                let right = compile_int64(right, fields, heaps, operations);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::EqualsUint64Uint64 { left, right }));
                index
            }
        },
        NodeBoolean::Matches(node) => match node {
            NodeBooleanMatches::StringRegex { left, right } => {
                let left = compile_string(left, fields, heaps, operations);
                let right = compile_regex(right, heaps);
                heaps.boolean.push(false);
                let index = heaps.boolean.len() - 1;
                operations.push((index, Operation::MatchesStringRegex { left, right }));
                index
            }
        },
    }
}

fn compile_string<T>(
    node: NodeString<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
    operations: &mut Vec<(usize, Operation)>,
) -> usize {
    match node {
        NodeString::_Phantom(_) => unreachable!(),
        NodeString::Constant(value) => {
            heaps.string.push(value);
            heaps.string.len() - 1
        }
        NodeString::Variable { name } => {
            if let Some((index, ChertField::String(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
        NodeString::Add(node) => match node {
            NodeStringAdd::StringString { left, right } => {
                let left = compile_string(*left, fields, heaps, operations);
                let right = compile_string(*right, fields, heaps, operations);
                heaps.string.push("".to_string());
                let index = heaps.string.len() - 1;
                operations.push((index, Operation::AddStringString { left, right }));
                index
            }
        },
    }
}

fn compile_int64<T>(
    node: NodeInt64<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
    operations: &mut Vec<(usize, Operation)>,
) -> usize {
    match node {
        NodeInt64::Variable { name } => {
            if let Some((index, ChertField::Int64(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
        NodeInt64::Negative(node) => match node {
            NodeInt64Negative::Uint64(node) => {
                let child = compile_uint64(*node, fields, heaps, operations);
                heaps.int64.push(0);
                let index = heaps.int64.len() - 1;
                operations.push((index, Operation::NegativeUint64(child)));
                index
            }
        },
    }
}

fn compile_regex<T>(node: NodeRegex<T>, heaps: &mut Heaps) -> usize {
    match node {
        NodeRegex::_Phantom(_) => unreachable!(),
        NodeRegex::Constant(value) => {
            heaps.regex.push(value);
            heaps.regex.len() - 1
        }
    }
}

fn compile_uint64<T>(
    node: NodeUint64<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    heaps: &mut Heaps,
    operations: &mut Vec<(usize, Operation)>,
) -> usize {
    match node {
        NodeUint64::_Phantom(_) => unreachable!(),
        NodeUint64::Constant(value) => {
            heaps.uint64.push(value);
            heaps.uint64.len() - 1
        }
        NodeUint64::Variable { name } => {
            if let Some((index, ChertField::Uint64(_))) = fields.get(&name) {
                *index
            } else {
                unreachable!();
            }
        }
        NodeUint64::Add(node) => match node {
            NodeUint64Add::Uint64Uint64 { left, right } => {
                let left = compile_uint64(*left, fields, heaps, operations);
                let right = compile_uint64(*right, fields, heaps, operations);
                heaps.uint64.push(0);
                let index = heaps.uint64.len() - 1;
                operations.push((index, Operation::AddUint64Uint64 { left, right }));
                index
            }
        },
        NodeUint64::Subtract(node) => match node {
            NodeUint64Subtract::Uint64Uint64 { left, right } => {
                let left = compile_uint64(*left, fields, heaps, operations);
                let right = compile_uint64(*right, fields, heaps, operations);
                heaps.uint64.push(0);
                let index = heaps.uint64.len() - 1;
                operations.push((index, Operation::SubtractUint64Uint64 { left, right }));
                index
            }
        },
    }
}

#[derive(Debug)]
pub struct Engine<T> {
    operations: Vec<(usize, Operation)>,
    pub heaps: Heaps,
    fields: HashMap<String, (usize, ChertField<T>)>,
    pub results: Vec<usize>,
    _phantom: Option<T>,
}

impl<T> Engine<T> {
    pub fn load_variables(&mut self, variables: &T) {
        for (_, (index, field)) in self.fields.iter() {
            match field {
                ChertField::Boolean(field) => self.heaps.boolean[*index] = *(*field)(variables),
                ChertField::Cidr(field) => self.heaps.cidr[*index] = *(*field)(variables),
                ChertField::Int64(field) => self.heaps.int64[*index] = *(*field)(variables),
                ChertField::Ip(field) => self.heaps.ip[*index] = *(*field)(variables),
                ChertField::String(field) => {
                    self.heaps.string[*index] = (*field)(variables).clone()
                }
                ChertField::Uint64(field) => self.heaps.uint64[*index] = *(*field)(variables),
            };
        }
    }

    pub fn eval(&mut self) {
        for (index, operation) in &self.operations {
            match operation {
                Operation::AddUint64Uint64 { left, right } => {
                    self.heaps.uint64[*index] = self.heaps.uint64[*left] + self.heaps.uint64[*right]
                }
                Operation::SubtractUint64Uint64 { left, right } => {
                    self.heaps.uint64[*index] = self.heaps.uint64[*left] - self.heaps.uint64[*right]
                }
                Operation::EqualsUint64Uint64 { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.uint64[*left] == self.heaps.uint64[*right]
                }
                Operation::EqualsInt64Int64 { left, right } => {
                    self.heaps.boolean[*index] = self.heaps.int64[*left] == self.heaps.int64[*right]
                }
                Operation::WithinIpCidr { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.cidr[*left].contains(&self.heaps.ip[*right])
                }
                Operation::MatchesStringRegex { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.regex[*right].is_match(&self.heaps.string[*left])
                }
                Operation::AddStringString { left, right } => {
                    self.heaps.string[*index] =
                        self.heaps.string[*left].to_string() + &self.heaps.string[*right]
                }
                Operation::BothBoolBool { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.boolean[*left] && self.heaps.boolean[*right]
                }
                Operation::EitherBoolBool { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.boolean[*left] || self.heaps.boolean[*right]
                }
                Operation::EqualsBoolBool { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.boolean[*left] == self.heaps.boolean[*right]
                }
                Operation::EqualsStringString { left, right } => {
                    self.heaps.boolean[*index] =
                        self.heaps.string[*left] == self.heaps.string[*right]
                }
                Operation::NegativeUint64(_child) => self.heaps.int64[*index] = -1,
                Operation::NotBool(child) => {
                    self.heaps.boolean[*index] = !self.heaps.boolean[*child]
                }
            };
        }
    }
}

pub fn compile<T: ChertStruct>(expressions: Vec<NodeBoolean<T>>) -> Engine<T> {
    let fields = T::fields();
    let mut heaps = Heaps::default();

    for (_, (_, field)) in fields.iter() {
        match field {
            ChertField::Boolean(_) => heaps.boolean.push(false),
            ChertField::Cidr(_) => heaps
                .cidr
                .push(IpCidr::V4(Ipv4Cidr::new_host(Ipv4Addr::from(0)))),
            ChertField::Int64(_) => heaps.int64.push(0),
            ChertField::Ip(_) => heaps.ip.push(IpAddr::V4(Ipv4Addr::from(0))),
            ChertField::String(_) => heaps.string.push("".to_string()),
            ChertField::Uint64(_) => heaps.uint64.push(0),
        };
    }

    let mut operations = Vec::new();
    let mut results = Vec::new();
    for node in expressions {
        compile_boolean(node, &fields, &mut heaps, &mut operations);
        results.push(heaps.boolean.len() - 1);
    }

    Engine {
        operations,
        heaps,
        results,
        fields,
        _phantom: Option::<T>::None,
    }
}
