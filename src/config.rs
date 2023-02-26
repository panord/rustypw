use crate::files;
use anyhow::Context;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub clear_copy_timeout: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        let fname = files::rpwd_path("config.json");
        let f = File::open(&fname).map_err(|_| anyhow!("Failed to open configuration"))?;

        serde_json::from_reader::<File, Config>(f)
            .map_err(|_| anyhow!("Failed deserializing configuration"))
    }

    pub fn save(&self) -> Result<&Self> {
        let fname = files::rpwd_path("config.json");
        let json = serde_json::to_string_pretty(&self).context("Failed to serialize passwords")?;

        std::fs::create_dir_all(&files::rpwd()).context("Failed to create rpw dir")?;
        File::create(&fname)
            .map(|mut f| {
                f.write_all(json.as_bytes()).expect("Failed to write file");
            })
            .map_err(|_| format!("Failed to create database {}", fname.display()))
            .expect("Failed to create vault file");

        Ok(self)
    }

    pub fn new() -> Self {
        Config {
            clear_copy_timeout: 5,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
