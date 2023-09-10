use cidr::IpCidr;

#[derive(Debug)]
pub enum NodeCidr<T> {
    Variable { name: String },
    Constant(IpCidr),
    _Phantom(T),
}
