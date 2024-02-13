use cidr::IpCidr;
use regex::Regex;
use std::collections::HashMap;
use std::net::IpAddr;

pub trait VariableType {
    type AccessedAs: ?Sized;

    fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> Variable<T>;
}

#[derive(Clone)]
pub enum Variable<T> {
    Boolean(fn(&T) -> &bool),
    Cidr(fn(&T) -> &IpCidr),
    Int64(fn(&T) -> &i64),
    Ip(fn(&T) -> &IpAddr),
    String(fn(&T) -> &str),
    Uint64(fn(&T) -> &u64),
    Regex(fn(&T) -> &Regex),
}

macro_rules! simple_field_type {
    ($type:ty, $variant:ident) => {
        impl VariableType for $type {
            type AccessedAs = $type;
            fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> Variable<T> {
                Variable::$variant(field)
            }
        }
    };
}

simple_field_type!(bool, Boolean);
simple_field_type!(i64, Int64);
simple_field_type!(u64, Uint64);
simple_field_type!(IpAddr, Ip);
simple_field_type!(IpCidr, Cidr);
simple_field_type!(str, String);
simple_field_type!(Regex, Regex);

impl VariableType for String {
    type AccessedAs = str;
    fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> Variable<T> {
        Variable::String(field)
    }
}

pub trait Variables: Sized + std::fmt::Debug {
    fn variables() -> HashMap<&'static str, Variable<Self>>;
}

impl<T> std::fmt::Debug for Variable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Boolean(_) => "Boolean(_)",
                Self::Cidr(_) => "Cidr(_)",
                Self::Int64(_) => "Int32(_)",
                Self::Ip(_) => "Ip(_)",
                Self::String(_) => "String(_)",
                Self::Uint64(_) => "Uint64(_)",
                Self::Regex(_) => "Regex(_)",
            }
        )
    }
}
