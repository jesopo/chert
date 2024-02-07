use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeIp {
    Variable { name: String },
    Constant(IpAddr),
}
