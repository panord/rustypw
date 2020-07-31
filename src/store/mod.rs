use std::string::String;

pub mod bw;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PwEntry {
    pub id: String,
    pub alias: String,
}

#[derive(Serialize, Deserialize)]
pub struct BwID {
    pub id: String,
}
