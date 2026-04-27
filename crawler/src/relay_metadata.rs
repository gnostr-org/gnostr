use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Relay {
    pub contact: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub software: Option<String>,
    pub supported_nips: Option<Vec<i32>>,
    pub supported_nip_extensions: Option<Vec<String>>,
    pub version: Option<String>,
}
