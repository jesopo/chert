use cidr::IpCidr;
use regex::Regex;
use std::collections::HashMap;
use std::net::IpAddr;

pub enum ChertField<T> {
    Boolean(fn(&T) -> &bool),
    Cidr(fn(&T) -> &IpCidr),
    Int64(fn(&T) -> &i64),
    Ip(fn(&T) -> &IpAddr),
    String(fn(&T) -> &String),
    Uint64(fn(&T) -> &u64),
    Regex(fn(&T) -> &Regex),
}

impl<T> From<fn(&T) -> &String> for ChertField<T> {
    fn from(field: fn(&T) -> &String) -> Self {
        Self::String(field)
    }
}

impl<T> From<fn(&T) -> &u64> for ChertField<T> {
    fn from(field: fn(&T) -> &u64) -> Self {
        Self::Uint64(field)
    }
}

impl<T> From<fn(&T) -> &i64> for ChertField<T> {
    fn from(field: fn(&T) -> &i64) -> Self {
        Self::Int64(field)
    }
}

impl<T> From<fn(&T) -> &bool> for ChertField<T> {
    fn from(field: fn(&T) -> &bool) -> Self {
        Self::Boolean(field)
    }
}

impl<T> From<fn(&T) -> &IpAddr> for ChertField<T> {
    fn from(field: fn(&T) -> &IpAddr) -> Self {
        Self::Ip(field)
    }
}

impl<T> From<fn(&T) -> &IpCidr> for ChertField<T> {
    fn from(field: fn(&T) -> &IpCidr) -> Self {
        Self::Cidr(field)
    }
}

impl<T> From<fn(&T) -> &Regex> for ChertField<T> {
    fn from(field: fn(&T) -> &Regex) -> Self {
        Self::Regex(field)
    }
}

pub trait ChertStructTrait: Sized + std::fmt::Debug {
    fn fields() -> HashMap<String, (usize, ChertField<Self>)>;
}

impl<T> ChertField<T> {
    pub fn type_key(&self) -> u8 {
        match self {
            Self::Boolean(_) => 0,
            Self::Cidr(_) => 1,
            Self::Int64(_) => 2,
            Self::Ip(_) => 3,
            Self::String(_) => 4,
            Self::Uint64(_) => 5,
            Self::Regex(_) => 6,
        }
    }
}

impl<T> std::fmt::Debug for ChertField<T> {
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
