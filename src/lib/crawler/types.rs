use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelayInfo {
    pub contact: String,
    pub description: String,
    pub name: String,
    pub software: String,
    pub supported_nips: Vec<i32>,
    pub version: String,
}
