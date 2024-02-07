use cidr::IpCidr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeCidr {
    Variable { name: String },
    Constant(IpCidr),
}
