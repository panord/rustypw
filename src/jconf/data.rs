use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PwEntry {
    pub alias: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct BwID {
    pub id: String,
}
