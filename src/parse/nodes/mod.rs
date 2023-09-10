pub mod boolean;
pub mod cidr;
pub mod int64;
pub mod ip;
pub mod regex;
pub mod string;
pub mod uint64;

use self::boolean::NodeBoolean;
use self::cidr::NodeCidr;
use self::int64::NodeInt64;
use self::ip::NodeIp;
use self::regex::NodeRegex;
use self::string::NodeString;
use self::uint64::NodeUint64;

#[derive(Debug)]
pub enum Node<T> {
    Boolean(NodeBoolean<T>),
    Cidr(NodeCidr<T>),
    Int64(NodeInt64<T>),
    Ip(NodeIp<T>),
    Regex(NodeRegex<T>),
    String(NodeString<T>),
    Uint64(NodeUint64<T>),
}
