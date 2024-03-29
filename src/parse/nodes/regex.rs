use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeRegex {
    Variable {
        name: String,
    },
    #[serde(with = "serde_regex")]
    Constant(Regex),
}
