extern crate dirs;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::string::String;

const DB_FNAME: &'static str = "remote.db";
const RPW_DIR: &'static str = ".rpw.d";

const BW_ALIAS: &'static str = "bw-token";
const BW_TOKEN: &'static str = "all-is-well";

pub struct BwStore {
    pub pws: Vec<PwAlias>,
}

#[derive(Serialize, Deserialize)]
pub struct PwAlias {
    pub id: String,
    pub alias: String,
}

#[derive(Serialize, Deserialize)]
pub struct BwID {
    pub id: String,
}

impl BwStore {
    pub fn new() -> BwStore {
        let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
        std::fs::create_dir_all(&rpw_d).expect("Failed to create rpw dir");
        let path = rpw_d.join(&DB_FNAME);

        match read(&path) {
            Ok(db) => BwStore { pws: db },
            Err(_) => BwStore {
                pws: vec![PwAlias {
                    alias: BW_ALIAS.to_string(),
                    id: BW_TOKEN.to_string(),
                }],
            },
        }
    }
}

impl Drop for BwStore {
    fn drop(&mut self) {
        let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
        write(&rpw_d.join(&DB_FNAME), &self.pws).unwrap();
    }
}

pub fn read(fname: &Path) -> Result<Vec<PwAlias>, String> {
    match File::open(&fname) {
        Ok(f) => Ok(serde_json::from_reader::<File, Vec<PwAlias>>(f)
            .expect("Failed deserializing database")),
        Err(_) => Err("Failed reading databse".to_string()),
    }
}

pub fn write(fname: &Path, entries: &[PwAlias]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&entries).expect("Failed to serialize passwords");

    File::create(fname)
        .and_then(|mut f| {
            f.write_all(&json.as_bytes()).expect("Failed to write file");
            Ok(())
        })
        .or_else(|_| Err(format!("Failed to create database {}", fname.display())))
}
