mod session;

use super::*;
use std::process::Command;
use std::result::Result;
use std::string::String;

pub struct BwStore {
    pub pws: Vec<PwEntry>,
}

fn get_pw(id: &str, session: &str) -> Result<String, String> {
    let out = Command::new("bw")
        .arg("get")
        .arg("password")
        .arg(id)
        .arg("--session")
        .arg(session)
        .output()
        .expect("Failed getting pw");

    match out.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
    }
}

fn get_pw_id(name: &str, session: &str) -> Result<String, String> {
    let json = Command::new("bw")
        .arg("get")
        .arg("item")
        .arg(name)
        .arg("--session")
        .arg(session)
        .arg("--pretty")
        .output()
        .expect("Failed getting pw");

    match json.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&json.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&json.stderr).unwrap().to_string()),
    }
}

impl BwStore {
    fn get_id(&self, alias: &str) -> Result<String, String> {
        for pw in &self.pws {
            if pw.alias == alias {
                return Ok(pw.id.clone());
            }
        }
        Err(format!("Could not find id corresponding to '{}'\n", alias))
    }

    pub fn lock(&mut self) -> Result<String, String> {
        session::delete()?;
        let out = Command::new("bw")
            .arg("lock")
            .output()
            .expect("Failed to set noisy terminal");

        match out.status.code().unwrap() {
            0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
            _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
        }
    }

    pub fn unlock(&mut self, pass: &str) -> Result<String, String> {
        let out = Command::new("bw")
            .arg("unlock")
            .arg("--raw")
            .arg(pass)
            .output()
            .expect("Failed to set noisy terminal");

        match out.status.code().unwrap() {
            0 => {
                let session = std::str::from_utf8(&out.stdout).unwrap();
                let msg = session.to_string();
                session::store(session)?;
                Ok(msg)
            }
            _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
        }
    }

    /* The semantics of store is confusing as  it is supposed to be key->value
     * in the case of remote cache it is local_key->remote_key->value */
    pub fn store(&mut self, name: &str, alias: &str) -> Result<String, String> {
        if self.get_id(alias).is_ok() {
            return Ok(format!("Alias {} already known", alias));
        }

        match get_pw_id(name, &session::load()?) {
            Ok(id) => {
                let entry = PwEntry {
                    id: serde_json::from_str::<BwID>(&id).expect("fail").id,
                    alias: alias.to_string(),
                };
                let msg = format!("{}={}", &entry.alias, &entry.id);
                self.pws.push(entry);
                Ok(msg)
            }
            Err(msg) => Err(msg.to_string()),
        }
    }

    pub fn load(&self, id: &str) -> Result<String, String> {
        match self.get_id(id) {
            Ok(rid) => get_pw(&rid, &session::load()?),
            Err(msg) => Err(msg),
        }
    }
}
