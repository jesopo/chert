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
use crate::{ChertField, ChertStructTrait};

use cidr::{IpCidr, Ipv4Cidr};
use regex::Regex;
use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Clone, Debug)]
pub struct Scratch {
    boolean: Vec<bool>,
    cidr: Vec<IpCidr>,
    int64: Vec<i64>,
    ip: Vec<IpAddr>,
    string: Vec<String>,
    uint64: Vec<u64>,
    regex: Vec<Regex>,
}

impl Scratch {
    fn new() -> Self {
        Self {
            boolean: Vec::default(),
            cidr: Vec::default(),
            int64: Vec::default(),
            ip: Vec::default(),
            string: Vec::default(),
            uint64: Vec::default(),
            regex: Vec::default(),
        }
    }
}

#[derive(Debug)]
pub enum Pointer {
    Constant(usize),
    Dynamic(usize),
}

#[derive(Debug)]
pub enum Operation<H: Hash> {
    AddStringString { left: Pointer, right: Pointer },
    AddUint64Uint64 { left: Pointer, right: Pointer },
    BothBoolBool { left: Pointer, right: Pointer },
    EitherBoolBool { left: Pointer, right: Pointer },
    EqualsBoolBool { left: Pointer, right: Pointer },
    EqualsStringString { left: Pointer, right: Pointer },
    EqualsUint64Uint64 { left: Pointer, right: Pointer },
    EqualsInt64Int64 { left: Pointer, right: Pointer },
    EqualsIpIP { left: Pointer, right: Pointer },
    NegativeUint64(Pointer),
    NotBool(Pointer),
    SubtractUint64Uint64 { left: Pointer, right: Pointer },
    WithinIpCidr { left: Pointer, right: Pointer },
    MatchesStringRegex { left: Pointer, right: Pointer },
    RaiseOutput { boolean: Pointer, id: H },
}

fn compile_ip<T>(
    node: NodeIp<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeIp::_Phantom(_) => unreachable!(),
        NodeIp::Constant(value) => {
            constants.ip.push(value);
            Pointer::Constant(constants.ip.len() - 1)
        }
        NodeIp::Variable { name } => {
            if let Some((index, ChertField::Ip(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_cidr<T>(
    node: NodeCidr<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeCidr::_Phantom(_) => unreachable!(),
        NodeCidr::Constant(value) => {
            constants.cidr.push(value);
            Pointer::Constant(constants.cidr.len() - 1)
        }
        NodeCidr::Variable { name } => {
            if let Some((index, ChertField::Cidr(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_boolean<T, H: Hash>(
    node: NodeBoolean<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Operation<H>)>,
) -> Pointer {
    match node {
        NodeBoolean::Constant(value) => {
            constants.boolean.push(value);
            Pointer::Constant(constants.boolean.len() - 1)
        }
        NodeBoolean::Variable { name } => {
            if let Some((index, ChertField::Boolean(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeBoolean::Not(node) => match node {
            NodeBooleanNot::Boolean(node) => {
                let child = compile_boolean(*node, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::NotBool(child)));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Both(node) => match node {
            NodeBooleanBoth::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, constants, dynamics, operations);
                let right = compile_boolean(*right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::BothBoolBool { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Either(node) => match node {
            NodeBooleanEither::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, constants, dynamics, operations);
                let right = compile_boolean(*right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EitherBoolBool { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Within(node) => match node {
            NodeBooleanWithin::IpCidr { left, right } => {
                let left = compile_ip(left, fields, constants);
                let right = compile_cidr(right, fields, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::WithinIpCidr { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Equals(node) => match node {
            NodeBooleanEquals::BooleanBoolean { left, right } => {
                let left = compile_boolean(*left, fields, constants, dynamics, operations);
                let right = compile_boolean(*right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EqualsBoolBool { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::StringString { left, right } => {
                let left = compile_string(left, fields, constants, dynamics, operations);
                let right = compile_string(right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EqualsStringString { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::Uint64Uint64 { left, right } => {
                let left = compile_uint64(left, fields, constants, dynamics, operations);
                let right = compile_uint64(right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EqualsUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::Int64Int64 { left, right } => {
                let left = compile_int64(left, fields, constants, dynamics, operations);
                let right = compile_int64(right, fields, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EqualsInt64Int64 { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::IpIp { left, right } => {
                let left = compile_ip(left, fields, constants);
                let right = compile_ip(right, fields, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::EqualsIpIP { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Matches(node) => match node {
            NodeBooleanMatches::StringRegex { left, right } => {
                let left = compile_string(left, fields, constants, dynamics, operations);
                let right = compile_regex(right, fields, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Operation::MatchesStringRegex { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_string<T, H: Hash>(
    node: NodeString<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Operation<H>)>,
) -> Pointer {
    match node {
        NodeString::_Phantom(_) => unreachable!(),
        NodeString::Constant(value) => {
            constants.string.push(value);
            Pointer::Constant(constants.string.len() - 1)
        }
        NodeString::Variable { name } => {
            if let Some((index, ChertField::String(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeString::Add(node) => match node {
            NodeStringAdd::StringString { left, right } => {
                let left = compile_string(*left, fields, constants, dynamics, operations);
                let right = compile_string(*right, fields, constants, dynamics, operations);
                dynamics.string.push("".to_string());
                let index = dynamics.string.len() - 1;
                operations.push((index, Operation::AddStringString { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_int64<T, H: Hash>(
    node: NodeInt64<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Operation<H>)>,
) -> Pointer {
    match node {
        NodeInt64::Variable { name } => {
            if let Some((index, ChertField::Int64(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeInt64::Negative(node) => match node {
            NodeInt64Negative::Uint64(node) => {
                let child = compile_uint64(*node, fields, constants, dynamics, operations);
                dynamics.int64.push(0);
                let index = dynamics.int64.len() - 1;
                operations.push((index, Operation::NegativeUint64(child)));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_regex<T>(
    node: NodeRegex<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeRegex::_Phantom(_) => unreachable!(),
        NodeRegex::Variable { name } => {
            if let Some((index, ChertField::Regex(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeRegex::Constant(value) => {
            constants.regex.push(value);
            Pointer::Constant(constants.regex.len() - 1)
        }
    }
}

fn compile_uint64<T, H: Hash>(
    node: NodeUint64<T>,
    fields: &HashMap<String, (usize, ChertField<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Operation<H>)>,
) -> Pointer {
    match node {
        NodeUint64::_Phantom(_) => unreachable!(),
        NodeUint64::Constant(value) => {
            constants.uint64.push(value);
            Pointer::Constant(constants.uint64.len() - 1)
        }
        NodeUint64::Variable { name } => {
            if let Some((index, ChertField::Uint64(_))) = fields.get(&name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeUint64::Add(node) => match node {
            NodeUint64Add::Uint64Uint64 { left, right } => {
                let left = compile_uint64(*left, fields, constants, dynamics, operations);
                let right = compile_uint64(*right, fields, constants, dynamics, operations);
                dynamics.uint64.push(0);
                let index = dynamics.uint64.len() - 1;
                operations.push((index, Operation::AddUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeUint64::Subtract(node) => match node {
            NodeUint64Subtract::Uint64Uint64 { left, right } => {
                let left = compile_uint64(*left, fields, constants, dynamics, operations);
                let right = compile_uint64(*right, fields, constants, dynamics, operations);
                dynamics.uint64.push(0);
                let index = dynamics.uint64.len() - 1;
                operations.push((index, Operation::SubtractUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

#[derive(Debug)]
pub struct Engine<T, H: Hash> {
    operations: Vec<(usize, Operation<H>)>,
    constants: Scratch,
    reference_dynamics: Scratch,
    fields: HashMap<String, (usize, ChertField<T>)>,
}

impl<T, H: Hash> Engine<T, H> {
    fn make_scratch(&self) -> Scratch {
        self.reference_dynamics.clone()
    }

    fn resolve_uint64<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &u64 {
        match pointer {
            Pointer::Constant(i) => &self.constants.uint64[*i],
            Pointer::Dynamic(i) => &dynamics.uint64[*i],
        }
    }

    fn resolve_int64<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &i64 {
        match pointer {
            Pointer::Constant(i) => &self.constants.int64[*i],
            Pointer::Dynamic(i) => &dynamics.int64[*i],
        }
    }

    fn resolve_cidr<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &IpCidr {
        match pointer {
            Pointer::Constant(i) => &self.constants.cidr[*i],
            Pointer::Dynamic(i) => &dynamics.cidr[*i],
        }
    }

    fn resolve_ip<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &IpAddr {
        match pointer {
            Pointer::Constant(i) => &self.constants.ip[*i],
            Pointer::Dynamic(i) => &dynamics.ip[*i],
        }
    }

    fn resolve_regex<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &Regex {
        match pointer {
            Pointer::Constant(i) => &self.constants.regex[*i],
            Pointer::Dynamic(i) => &dynamics.regex[*i],
        }
    }

    fn resolve_string<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &String {
        match pointer {
            Pointer::Constant(i) => &self.constants.string[*i],
            Pointer::Dynamic(i) => &dynamics.string[*i],
        }
    }

    fn resolve_boolean<'a>(&'a self, dynamics: &'a Scratch, pointer: &Pointer) -> &bool {
        match pointer {
            Pointer::Constant(i) => &self.constants.boolean[*i],
            Pointer::Dynamic(i) => &dynamics.boolean[*i],
        }
    }

    pub fn eval(&self, variables: &T) -> Vec<&H> {
        let mut dynamics = self.make_scratch();

        for (_, (index, field)) in self.fields.iter() {
            match field {
                ChertField::Boolean(field) => dynamics.boolean[*index] = *(*field)(variables),
                ChertField::Cidr(field) => dynamics.cidr[*index] = *(*field)(variables),
                ChertField::Int64(field) => dynamics.int64[*index] = *(*field)(variables),
                ChertField::Ip(field) => dynamics.ip[*index] = *(*field)(variables),
                ChertField::String(field) => {
                    dynamics.string[*index] = (*field)(variables).to_owned()
                }
                ChertField::Uint64(field) => dynamics.uint64[*index] = *(*field)(variables),
                ChertField::Regex(field) => dynamics.regex[*index] = (*field)(variables).clone(),
            };
        }

        let mut matched = Vec::new();
        for (index, operation) in &self.operations {
            match operation {
                Operation::AddUint64Uint64 { left, right } => {
                    dynamics.uint64[*index] =
                        self.resolve_uint64(&dynamics, left) + self.resolve_uint64(&dynamics, right)
                }
                Operation::SubtractUint64Uint64 { left, right } => {
                    dynamics.uint64[*index] = self.resolve_uint64(&dynamics, left)
                        - self.resolve_uint64(&dynamics, right);
                }
                Operation::EqualsUint64Uint64 { left, right } => {
                    dynamics.boolean[*index] = self.resolve_uint64(&dynamics, left)
                        == self.resolve_uint64(&dynamics, right)
                }
                Operation::EqualsInt64Int64 { left, right } => {
                    dynamics.boolean[*index] =
                        self.resolve_int64(&dynamics, left) == self.resolve_int64(&dynamics, right);
                }
                Operation::WithinIpCidr { left, right } => {
                    dynamics.boolean[*index] = self
                        .resolve_cidr(&dynamics, right)
                        .contains(self.resolve_ip(&dynamics, left));
                }
                Operation::MatchesStringRegex { left, right } => {
                    dynamics.boolean[*index] = self
                        .resolve_regex(&dynamics, right)
                        .is_match(self.resolve_string(&dynamics, left));
                }
                Operation::AddStringString { left, right } => {
                    dynamics.string[*index] = self.resolve_string(&dynamics, left).clone()
                        + self.resolve_string(&dynamics, right)
                }
                Operation::BothBoolBool { left, right } => {
                    dynamics.boolean[*index] = *self.resolve_boolean(&dynamics, left)
                        && *self.resolve_boolean(&dynamics, right);
                }
                Operation::EitherBoolBool { left, right } => {
                    dynamics.boolean[*index] = *self.resolve_boolean(&dynamics, left)
                        || *self.resolve_boolean(&dynamics, right);
                }
                Operation::EqualsBoolBool { left, right } => {
                    dynamics.boolean[*index] = self.resolve_boolean(&dynamics, left)
                        == self.resolve_boolean(&dynamics, right);
                }
                Operation::EqualsStringString { left, right } => {
                    dynamics.boolean[*index] = self.resolve_string(&dynamics, left)
                        == self.resolve_string(&dynamics, right);
                }
                Operation::NegativeUint64(child) => {
                    // will happily overflow. perhaps we should emit warnings about this stuff at compiletime
                    dynamics.int64[*index] = -(*self.resolve_uint64(&dynamics, child) as i64);
                }
                Operation::NotBool(child) => {
                    dynamics.boolean[*index] = !self.resolve_boolean(&dynamics, child);
                }
                Operation::EqualsIpIP { left, right } => {
                    dynamics.boolean[*index] =
                        self.resolve_ip(&dynamics, left) == self.resolve_ip(&dynamics, right);
                }
                Operation::RaiseOutput { boolean, id } => {
                    if *self.resolve_boolean(&dynamics, boolean) {
                        matched.push(id);
                    }
                }
            };
        }

        matched
    }
}

pub fn compile<T: ChertStructTrait, H: Hash>(
    expressions: impl IntoIterator<Item = (H, NodeBoolean<T>)>,
) -> Engine<T, H> {
    let fields = T::fields();
    let mut constants = Scratch::new();
    let mut initial_dynamics = Scratch::new();

    for (_, (_, field)) in fields.iter() {
        match field {
            ChertField::Boolean(_) => initial_dynamics.boolean.push(false),
            ChertField::Cidr(_) => initial_dynamics
                .cidr
                .push(IpCidr::V4(Ipv4Cidr::new_host(Ipv4Addr::from(0)))),
            ChertField::Int64(_) => initial_dynamics.int64.push(0),
            ChertField::Ip(_) => initial_dynamics.ip.push(IpAddr::V4(Ipv4Addr::from(0))),
            ChertField::String(_) => initial_dynamics.string.push("".to_string()),
            ChertField::Uint64(_) => initial_dynamics.uint64.push(0),
            ChertField::Regex(_) => initial_dynamics.regex.push(Regex::new("").unwrap()),
        };
    }

    let mut max_size_dynamics = initial_dynamics.clone();
    let mut operations = Vec::new();
    for (id, expression) in expressions {
        let mut dynamics = initial_dynamics.clone();
        compile_boolean(
            expression,
            &fields,
            &mut constants,
            &mut dynamics,
            &mut operations,
        );
        operations.push((
            0,
            Operation::RaiseOutput {
                boolean: Pointer::Dynamic(dynamics.boolean.len() - 1),
                id,
            },
        ));
        let Scratch {
            boolean,
            cidr,
            int64,
            ip,
            string,
            uint64,
            regex,
        } = dynamics;
        //TODO: this sucks, do better than this
        if boolean.len() > max_size_dynamics.boolean.len() {
            max_size_dynamics.boolean = boolean;
        }
        if cidr.len() > max_size_dynamics.boolean.len() {
            max_size_dynamics.cidr = cidr;
        }
        if int64.len() > max_size_dynamics.int64.len() {
            max_size_dynamics.int64 = int64;
        }
        if ip.len() > max_size_dynamics.ip.len() {
            max_size_dynamics.ip = ip;
        }
        if string.len() > max_size_dynamics.string.len() {
            max_size_dynamics.string = string;
        }
        if uint64.len() > max_size_dynamics.uint64.len() {
            max_size_dynamics.uint64 = uint64;
        }
        if regex.len() > max_size_dynamics.regex.len() {
            max_size_dynamics.regex = regex;
        }
    }

    Engine {
        operations,
        constants,
        reference_dynamics: max_size_dynamics,
        fields,
    }
}
