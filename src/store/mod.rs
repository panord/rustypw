use std::result::Result;
use std::string::String;

pub mod bw;

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

fn get_id<'a>(alias: &str, pws: &'a Vec<PwEntry>) -> Result<&'a str, String> {
    for pw in pws {
        if pw.alias == alias {
            return Ok(&pw.id);
        }
    }
    Err(format!("Could not find id corresponding to '{}'\n", alias))
}

pub fn get_pw_by_alias(pws: &Vec<PwEntry>, alias: &str, session: &str) -> Result<String, String> {
    match get_id(alias, pws) {
        Ok(id) => bw::get_pw(&id, &session),
        Err(msg) => Err(msg),
    }
}

pub fn alias(
    pws: &mut Vec<PwEntry>,
    name: &str,
    alias: &str,
    session: &str,
) -> Result<String, String> {
    if get_id(alias, pws).is_ok() {
        return Ok(format!("Alias {} already known", alias));
    }

    match bw::get_pw_id(name, &session) {
        Ok(id) => {
            let entry = PwEntry {
                id: serde_json::from_str::<BwID>(&id).expect("fail").id,
                alias: alias.to_string(),
            };
            let msg = format!("{}={}", &entry.alias, &entry.id);
            pws.push(entry);
            Ok(msg)
        }
        Err(msg) => Err(msg.to_string()),
    }
}
