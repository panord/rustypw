use std::result::Result;
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

pub trait PwStore {
    fn lock(&mut self) -> Result<String, String>;
    fn unlock(&mut self, mpw: &str) -> Result<String, String>;

    fn store(&mut self, id: &str, pw: &str) -> Result<String, String>;
    fn load(&self, id: &str) -> Result<String, String>;
}
