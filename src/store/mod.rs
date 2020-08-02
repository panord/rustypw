use std::string::String;

pub mod bw;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PwAlias {
    pub id: String,
    pub alias: String,
}

#[derive(Serialize, Deserialize)]
pub struct BwID {
    pub id: String,
}
