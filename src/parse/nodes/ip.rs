use std::net::IpAddr;

#[derive(Debug)]
pub enum NodeIp<T> {
    Variable { name: String },
    Constant(IpAddr),
    _Phantom(T),
}
