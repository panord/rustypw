use std::process::Command;
use std::result::Result;
use std::string::String;

use crate::jconf::BwID;
use crate::jconf::PwEntry;

pub fn lock() -> Result<String, String> {
    let out = Command::new("bw")
        .arg("lock")
        .output()
        .expect("Failed to set noisy terminal");

    match out.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
    }
}

pub fn unlock(pass: &str) -> Result<String, String> {
    let out = Command::new("bw")
        .arg("unlock")
        .arg("--raw")
        .arg(pass)
        .output()
        .expect("Failed to set noisy terminal");

    match out.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
    }
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
        .expect("Failed aliasing");

    match json.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&json.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&json.stderr).unwrap().to_string()),
    }
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
        Ok(id) => get_pw(&id, &session),
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

    match get_pw_id(name, &session) {
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
