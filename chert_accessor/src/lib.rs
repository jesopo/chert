use cidr::IpCidr;
use std::collections::HashMap;
use std::net::IpAddr;

pub enum ChertField<T> {
    Boolean(Box<dyn Fn(&T) -> &bool>),
    Cidr(Box<dyn Fn(&T) -> &IpCidr>),
    Int64(Box<dyn Fn(&T) -> &i64>),
    Ip(Box<dyn Fn(&T) -> &IpAddr>),
    String(Box<dyn Fn(&T) -> &String>),
    Uint64(Box<dyn Fn(&T) -> &u64>),
}

impl<T> From<Box<dyn Fn(&T) -> &String>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &String>) -> Self {
        Self::String(field)
    }
}

impl<T> From<Box<dyn Fn(&T) -> &u64>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &u64>) -> Self {
        Self::Uint64(field)
    }
}

impl<T> From<Box<dyn Fn(&T) -> &i64>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &i64>) -> Self {
        Self::Int64(field)
    }
}

impl<T> From<Box<dyn Fn(&T) -> &bool>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &bool>) -> Self {
        Self::Boolean(field)
    }
}

impl<T> From<Box<dyn Fn(&T) -> &IpAddr>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &IpAddr>) -> Self {
        Self::Ip(field)
    }
}

impl<T> From<Box<dyn Fn(&T) -> &IpCidr>> for ChertField<T> {
    fn from(field: Box<dyn Fn(&T) -> &IpCidr>) -> Self {
        Self::Cidr(field)
    }
}

pub trait ChertStruct: Sized + std::fmt::Debug {
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
            }
        )
    }
}
