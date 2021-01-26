use crate::files;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub clear_copy_timeout: u64,
}

impl Config {
    pub fn load() -> Result<Self, ()> {
        let fname = files::rpwd_path("config.json");
        match File::open(&fname) {
            Ok(f) => Ok(serde_json::from_reader::<File, Config>(f)
                .expect("Failed deserializing configuration")),
            Err(_) => Err(()),
        }
    }

    pub fn save(&self) -> Self {
        let fname = files::rpwd_path("config.json");
        let json = serde_json::to_string_pretty(&self).expect("Failed to serialize passwords");

        File::create(&fname)
            .and_then(|mut f| {
                f.write_all(&json.as_bytes()).expect("Failed to write file");
                Ok(())
            })
            .or_else(|_| Err(format!("Failed to create database {}", fname.display())))
            .expect("Failed to create vault file");
        return self.clone();
    }

    pub fn new() -> Self {
        Config {
            clear_copy_timeout: 5,
        }
    }
}
