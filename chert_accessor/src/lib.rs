use cidr::IpCidr;
use regex::Regex;
use std::collections::HashMap;
use std::net::IpAddr;

pub trait ChertFieldType {
    type AccessedAs: ?Sized;

    fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> ChertField<T>;
}

pub enum ChertField<T> {
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
        impl ChertFieldType for $type {
            type AccessedAs = $type;
            fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> ChertField<T> {
                ChertField::$variant(field)
            }
        }
    }
}

simple_field_type!(bool, Boolean);
simple_field_type!(i64, Int64);
simple_field_type!(u64, Uint64);
simple_field_type!(IpAddr, Ip);
simple_field_type!(IpCidr, Cidr);
simple_field_type!(str, String);
simple_field_type!(Regex, Regex);

impl ChertFieldType for String {
    type AccessedAs = str;
    fn from_field<T>(field: fn(&T) -> &Self::AccessedAs) -> ChertField<T> {
        ChertField::String(field)
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
