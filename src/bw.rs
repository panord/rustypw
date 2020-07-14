use std::process::Command;
use std::result::Result;
use std::string::String;

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
