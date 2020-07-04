use std::fs::File;
use std::path::Path;

use crate::cli;
use crate::jconf::*;

fn db_create_if_yes(fname: &Path) -> bool {
    println!("Could not find database {}", fname.display());
    if !cli::yesorno(&format!("Would you like to create {} ?", fname.display())) {
        return false;
    }
    println!("Creating {}", fname.display());
    File::create(fname).expect(&format!("Failed to create database {}", fname.display()));
    return true;
}

pub fn read(fname: &Path) -> Result<Vec<PwEntry>, String> {
    match File::open(&fname) {
        Ok(f) => Ok(serde_json::from_reader::<File, Vec<PwEntry>>(f)
            .expect("Failed deserializing database")),
        Err(_) => Err("Failed reading databse".to_string()),
    }
}

pub fn write(fname: &Path, entries: Vec<PwEntry>) -> Result<(), String> {
    serde_json::to_string(&entries).expect("Failed to serialize passwords");

    File::create(fname)
        .and_then(|_| Ok(()))
        .or_else(|_| Err(format!("Failed to create database {}", fname.display())))
}

pub fn init(fname: &Path) -> Result<(), String> {
    match File::open(&fname) {
        Ok(_) => Ok(()),
        Err(_) => {
            db_create_if_yes(&fname);
            write(&fname, vec![])?;
            read(&fname)?;
            Ok(())
        }
    }
}
