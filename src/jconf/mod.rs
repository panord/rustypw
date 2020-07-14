use crate::cli;
use crate::store;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use store::PwEntry;

fn create(fname: &Path) {
    println!("Creating {}", fname.display());
    File::create(fname).expect(&format!("Failed to create database {}", fname.display()));
}

fn create_interactive(fname: &Path) -> bool {
    println!("Could not find database {}", fname.display());
    if !cli::yesorno(&format!("Would you like to create {} ?", fname.display())) {
        return false;
    }
    create(fname);
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
    let json = serde_json::to_string_pretty(&entries).expect("Failed to serialize passwords");

    File::create(fname)
        .and_then(|mut f| {
            f.write_all(&json.as_bytes()).expect("Failed to write file");
            Ok(())
        })
        .or_else(|_| Err(format!("Failed to create database {}", fname.display())))
}

pub fn init(fname: &Path) -> Result<(), String> {
    match File::open(&fname) {
        Ok(_) => Ok(()),
        Err(_) => {
            create_interactive(&fname);
            write(&fname, vec![])?;
            read(&fname)?;
            Ok(())
        }
    }
}
