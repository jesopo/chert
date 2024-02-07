use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum NodeIp {
    Variable { name: String },
    Constant(IpAddr),
}
