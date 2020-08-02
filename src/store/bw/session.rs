use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// TODO: Proper error API

pub fn store(session: &str) -> Result<(), String> {
    let fname = Path::new("/tmp/rpw-session");
    let mut file = File::create(fname).expect("Failed to create session");
    match file.write_all(session.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to write session".to_string()),
    }
}

pub fn load() -> Result<String, String> {
    let fname: &Path = Path::new("/tmp/rpw-session");
    match std::fs::read_to_string(fname) {
        Ok(s) => Ok(s),
        Err(_) => Err("Failed to load session".to_string()),
    }
}

pub fn delete() -> Result<(), String> {
    let fname: &Path = Path::new("/tmp/rpw-session");
    match std::fs::remove_file(fname) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to delete session".to_string()),
    }
}
