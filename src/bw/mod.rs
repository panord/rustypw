use std::process::Command;
use std::result::Result;
use std::string::String;

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

pub fn get_pw(id: &str, session: &str) -> Result<String, String> {
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

pub fn get_pw_id(name: &str, session: &str) -> Result<String, String> {
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

pub fn phrase(len: u8) -> Result<String, String> {
    let out = Command::new("bw")
        .arg("generate")
        .arg("-p")
        .arg("--words")
        .arg(len.to_string())
        .arg("--separator")
        .arg("space")
        .output()
        .expect("Failed getting pw");

    match out.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
    }
}
