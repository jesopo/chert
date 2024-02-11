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
use crate::parse::Ast;
use crate::variables::{Variable, Variables};

use cidr::{IpCidr, Ipv4Cidr};
use regex::Regex;
use std::borrow::Borrow;
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

#[derive(Clone, Debug)]
pub enum Pointer {
    Constant(usize),
    Dynamic(usize),
}

#[derive(Clone, Debug)]
pub enum Instruction<H: Hash> {
    Nothing,
    SkipIfTrue { check: Pointer, forward: usize },
    SkipIfFalse { check: Pointer, forward: usize },
    RaiseOutput { boolean: Pointer, id: H },
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
}

fn compile_ip<T>(
    node: &NodeIp,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeIp::Constant(value) => {
            constants.ip.push(*value);
            Pointer::Constant(constants.ip.len() - 1)
        }
        NodeIp::Variable { name } => {
            if let Some((index, Variable::Ip(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_cidr<T>(
    node: &NodeCidr,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeCidr::Constant(value) => {
            constants.cidr.push(*value);
            Pointer::Constant(constants.cidr.len() - 1)
        }
        NodeCidr::Variable { name } => {
            if let Some((index, Variable::Cidr(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
    }
}

fn compile_boolean<T, H: Hash>(
    node: &NodeBoolean,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Instruction<H>)>,
) -> Pointer {
    match node {
        NodeBoolean::Constant(value) => {
            constants.boolean.push(*value);
            Pointer::Constant(constants.boolean.len() - 1)
        }
        NodeBoolean::Variable { name } => {
            if let Some((index, Variable::Boolean(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeBoolean::Not(node) => match node {
            NodeBooleanNot::Boolean(node) => {
                let child = compile_boolean(node, variables, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::NotBool(child)));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Both(node) => match node {
            NodeBooleanBoth::BooleanBoolean { left, right } => {
                let left = compile_boolean(left, variables, constants, dynamics, operations);
                let jump_insert = operations.len();
                operations.push((0, Instruction::Nothing));
                let right = compile_boolean(right, variables, constants, dynamics, operations);
                let output = dynamics.boolean.len();
                dynamics.boolean.push(false);
                operations[jump_insert] = (
                    output,
                    Instruction::SkipIfFalse {
                        check: left.clone(),
                        forward: operations.len() - jump_insert,
                    },
                );
                operations.push((output, Instruction::BothBoolBool { left, right }));
                Pointer::Dynamic(output)
            }
        },
        NodeBoolean::Either(node) => match node {
            NodeBooleanEither::BooleanBoolean { left, right } => {
                let left = compile_boolean(left, variables, constants, dynamics, operations);
                let jump_insert = operations.len();
                operations.push((0, Instruction::Nothing));
                let right = compile_boolean(right, variables, constants, dynamics, operations);
                let output = dynamics.boolean.len();
                dynamics.boolean.push(false);
                operations[jump_insert] = (
                    output,
                    Instruction::SkipIfTrue {
                        check: left.clone(),
                        forward: operations.len() - jump_insert,
                    },
                );
                operations.push((output, Instruction::EitherBoolBool { left, right }));
                Pointer::Dynamic(output)
            }
        },
        NodeBoolean::Within(node) => match node {
            NodeBooleanWithin::IpCidr { left, right } => {
                let left = compile_ip(left, variables, constants);
                let right = compile_cidr(right, variables, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::WithinIpCidr { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Equals(node) => match node {
            NodeBooleanEquals::BooleanBoolean { left, right } => {
                let left = compile_boolean(left, variables, constants, dynamics, operations);
                let right = compile_boolean(right, variables, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::EqualsBoolBool { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::StringString { left, right } => {
                let left = compile_string(left, variables, constants, dynamics, operations);
                let right = compile_string(right, variables, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::EqualsStringString { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::Uint64Uint64 { left, right } => {
                let left = compile_uint64(left, variables, constants, dynamics, operations);
                let right = compile_uint64(right, variables, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::EqualsUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::Int64Int64 { left, right } => {
                let left = compile_int64(left, variables, constants, dynamics, operations);
                let right = compile_int64(right, variables, constants, dynamics, operations);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::EqualsInt64Int64 { left, right }));
                Pointer::Dynamic(index)
            }
            NodeBooleanEquals::IpIp { left, right } => {
                let left = compile_ip(left, variables, constants);
                let right = compile_ip(right, variables, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::EqualsIpIP { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeBoolean::Matches(node) => match node {
            NodeBooleanMatches::StringRegex { left, right } => {
                let left = compile_string(left, variables, constants, dynamics, operations);
                let right = compile_regex(right, variables, constants);
                dynamics.boolean.push(false);
                let index = dynamics.boolean.len() - 1;
                operations.push((index, Instruction::MatchesStringRegex { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_string<T, H: Hash>(
    node: &NodeString,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Instruction<H>)>,
) -> Pointer {
    match node {
        NodeString::Constant(value) => {
            constants.string.push(value.clone());
            Pointer::Constant(constants.string.len() - 1)
        }
        NodeString::Variable { name } => {
            if let Some((index, Variable::String(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeString::Add(node) => match node {
            NodeStringAdd::StringString { left, right } => {
                let left = compile_string(left, variables, constants, dynamics, operations);
                let right = compile_string(right, variables, constants, dynamics, operations);
                dynamics.string.push("".to_string());
                let index = dynamics.string.len() - 1;
                operations.push((index, Instruction::AddStringString { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_int64<T, H: Hash>(
    node: &NodeInt64,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Instruction<H>)>,
) -> Pointer {
    match node {
        NodeInt64::Variable { name } => {
            if let Some((index, Variable::Int64(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeInt64::Negative(node) => match node {
            NodeInt64Negative::Uint64(node) => {
                let child = compile_uint64(node, variables, constants, dynamics, operations);
                dynamics.int64.push(0);
                let index = dynamics.int64.len() - 1;
                operations.push((index, Instruction::NegativeUint64(child)));
                Pointer::Dynamic(index)
            }
        },
    }
}

fn compile_regex<T>(
    node: &NodeRegex,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
) -> Pointer {
    match node {
        NodeRegex::Variable { name } => {
            if let Some((index, Variable::Regex(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeRegex::Constant(value) => {
            constants.regex.push(value.clone());
            Pointer::Constant(constants.regex.len() - 1)
        }
    }
}

fn compile_uint64<T, H: Hash>(
    node: &NodeUint64,
    variables: &HashMap<String, (usize, Variable<T>)>,
    constants: &mut Scratch,
    dynamics: &mut Scratch,
    operations: &mut Vec<(usize, Instruction<H>)>,
) -> Pointer {
    match node {
        NodeUint64::Constant(value) => {
            constants.uint64.push(*value);
            Pointer::Constant(constants.uint64.len() - 1)
        }
        NodeUint64::Variable { name } => {
            if let Some((index, Variable::Uint64(_))) = variables.get(name) {
                Pointer::Dynamic(*index)
            } else {
                unreachable!();
            }
        }
        NodeUint64::Add(node) => match node {
            NodeUint64Add::Uint64Uint64 { left, right } => {
                let left = compile_uint64(left, variables, constants, dynamics, operations);
                let right = compile_uint64(right, variables, constants, dynamics, operations);
                dynamics.uint64.push(0);
                let index = dynamics.uint64.len() - 1;
                operations.push((index, Instruction::AddUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
        },
        NodeUint64::Subtract(node) => match node {
            NodeUint64Subtract::Uint64Uint64 { left, right } => {
                let left = compile_uint64(left, variables, constants, dynamics, operations);
                let right = compile_uint64(right, variables, constants, dynamics, operations);
                dynamics.uint64.push(0);
                let index = dynamics.uint64.len() - 1;
                operations.push((index, Instruction::SubtractUint64Uint64 { left, right }));
                Pointer::Dynamic(index)
            }
        },
    }
}

#[derive(Clone, Debug)]
pub struct Engine<T, H: Hash> {
    operations: Vec<(usize, Instruction<H>)>,
    constants: Scratch,
    reference_dynamics: Scratch,
    variables: HashMap<String, (usize, Variable<T>)>,
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

        for (_, (index, field)) in self.variables.iter() {
            match field {
                Variable::Boolean(field) => dynamics.boolean[*index] = *(*field)(variables),
                Variable::Cidr(field) => dynamics.cidr[*index] = *(*field)(variables),
                Variable::Int64(field) => dynamics.int64[*index] = *(*field)(variables),
                Variable::Ip(field) => dynamics.ip[*index] = *(*field)(variables),
                Variable::String(field) => dynamics.string[*index] = (*field)(variables).to_owned(),
                Variable::Uint64(field) => dynamics.uint64[*index] = *(*field)(variables),
                Variable::Regex(field) => dynamics.regex[*index] = (*field)(variables).clone(),
            };
        }

        let mut matched = Vec::new();
        let mut instructions = self.operations.iter();
        while let Some((output, instruction)) = instructions.next() {
            match instruction {
                Instruction::Nothing => {}
                Instruction::SkipIfTrue { check, forward } => {
                    let check = *self.resolve_boolean(&dynamics, check);
                    dynamics.boolean[*output] = check;
                    if check {
                        for _ in 0..*forward {
                            instructions.next();
                        }
                    }
                }
                Instruction::SkipIfFalse { check, forward } => {
                    let check = *self.resolve_boolean(&dynamics, check);
                    dynamics.boolean[*output] = check;
                    if !check {
                        for _ in 0..*forward {
                            instructions.next();
                        }
                    }
                }
                Instruction::RaiseOutput { boolean, id } => {
                    if *self.resolve_boolean(&dynamics, boolean) {
                        matched.push(id);
                    }
                }
                Instruction::AddUint64Uint64 { left, right } => {
                    dynamics.uint64[*output] =
                        self.resolve_uint64(&dynamics, left) + self.resolve_uint64(&dynamics, right)
                }
                Instruction::SubtractUint64Uint64 { left, right } => {
                    dynamics.uint64[*output] = self.resolve_uint64(&dynamics, left)
                        - self.resolve_uint64(&dynamics, right);
                }
                Instruction::EqualsUint64Uint64 { left, right } => {
                    dynamics.boolean[*output] = self.resolve_uint64(&dynamics, left)
                        == self.resolve_uint64(&dynamics, right)
                }
                Instruction::EqualsInt64Int64 { left, right } => {
                    dynamics.boolean[*output] =
                        self.resolve_int64(&dynamics, left) == self.resolve_int64(&dynamics, right);
                }
                Instruction::WithinIpCidr { left, right } => {
                    dynamics.boolean[*output] = self
                        .resolve_cidr(&dynamics, right)
                        .contains(self.resolve_ip(&dynamics, left));
                }
                Instruction::MatchesStringRegex { left, right } => {
                    dynamics.boolean[*output] = self
                        .resolve_regex(&dynamics, right)
                        .is_match(self.resolve_string(&dynamics, left));
                }
                Instruction::AddStringString { left, right } => {
                    dynamics.string[*output] = self.resolve_string(&dynamics, left).clone()
                        + self.resolve_string(&dynamics, right)
                }
                Instruction::BothBoolBool { left, right } => {
                    dynamics.boolean[*output] = *self.resolve_boolean(&dynamics, left)
                        && *self.resolve_boolean(&dynamics, right);
                }
                Instruction::EitherBoolBool { left, right } => {
                    dynamics.boolean[*output] = *self.resolve_boolean(&dynamics, left)
                        || *self.resolve_boolean(&dynamics, right);
                }
                Instruction::EqualsBoolBool { left, right } => {
                    dynamics.boolean[*output] = self.resolve_boolean(&dynamics, left)
                        == self.resolve_boolean(&dynamics, right);
                }
                Instruction::EqualsStringString { left, right } => {
                    dynamics.boolean[*output] = self.resolve_string(&dynamics, left)
                        == self.resolve_string(&dynamics, right);
                }
                Instruction::NegativeUint64(child) => {
                    // will happily overflow. perhaps we should emit warnings about this stuff at compiletime
                    dynamics.int64[*output] = -(*self.resolve_uint64(&dynamics, child) as i64);
                }
                Instruction::NotBool(child) => {
                    dynamics.boolean[*output] = !self.resolve_boolean(&dynamics, child);
                }
                Instruction::EqualsIpIP { left, right } => {
                    dynamics.boolean[*output] =
                        self.resolve_ip(&dynamics, left) == self.resolve_ip(&dynamics, right);
                }
            };
        }

        matched
    }
}

pub fn compile<T, H, I>(expressions: I) -> Engine<T, H>
where
    T: Variables,
    H: Hash,
    I: IntoIterator<Item = (H, Ast<T, NodeBoolean>)>,
{
    let expressions = expressions.into_iter().map(|(id, ast)| (id, ast.root));
    compile_unsafe(expressions)
}

pub fn compile_unsafe<T, H, N, I>(expressions: I) -> Engine<T, H>
where
    T: Variables,
    H: Hash,
    N: Borrow<NodeBoolean>,
    I: IntoIterator<Item = (H, N)>,
{
    let variables = T::variables();
    let mut constants = Scratch::new();
    let mut initial_dynamics = Scratch::new();

    for (_, (_, field)) in variables.iter() {
        match field {
            Variable::Boolean(_) => initial_dynamics.boolean.push(false),
            Variable::Cidr(_) => initial_dynamics
                .cidr
                .push(IpCidr::V4(Ipv4Cidr::new_host(Ipv4Addr::from(0)))),
            Variable::Int64(_) => initial_dynamics.int64.push(0),
            Variable::Ip(_) => initial_dynamics.ip.push(IpAddr::V4(Ipv4Addr::from(0))),
            Variable::String(_) => initial_dynamics.string.push("".to_string()),
            Variable::Uint64(_) => initial_dynamics.uint64.push(0),
            Variable::Regex(_) => initial_dynamics.regex.push(Regex::new("").unwrap()),
        };
    }

    let mut max_size_dynamics = initial_dynamics.clone();
    let mut operations = Vec::new();
    for (id, expression) in expressions {
        let expression = expression.borrow();
        let mut dynamics = initial_dynamics.clone();

        compile_boolean(
            expression,
            &variables,
            &mut constants,
            &mut dynamics,
            &mut operations,
        );
        operations.push((
            0,
            Instruction::RaiseOutput {
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
        variables,
    }
}
