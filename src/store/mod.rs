mod crypto;
use crate::cli;
use crate::command;
use crate::files;
use command::ArgParseError;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::string::String;

const SALT_LEN: usize = 256;
const IV_LEN: usize = 16;
const VAULT_EXT: &'static str = ".vlt";

#[derive(Serialize, Deserialize)]
pub struct LockedVault {
    pub name: String,
    pub iv: Vec<u8>,
    pub salt: Vec<u8>,
    pub enc: Vec<u8>,
}

pub struct UnlockedVault {
    pub name: String,
    pub salt: Vec<u8>,
    pub pws: HashMap<String, String>,
}

impl LockedVault {
    pub fn unlock(&self, pass: &str) -> Result<UnlockedVault, String> {
        let salt = &self.salt;
        let key = crypto::key(&pass.as_bytes(), &salt).unwrap();
        let cipher = Cipher::aes_256_cbc();
        let data = &self.enc;
        let iv = &self.iv;
        let plain = decrypt(cipher, &key, Some(&iv), data);
        if plain.is_err() {
            return Err("Vault could not be decryted".to_string());
        }
        let json = String::from_utf8(plain.unwrap()).unwrap();
        let passwords: HashMap<String, String> = serde_json::from_str(&json).unwrap();

        Ok(UnlockedVault {
            name: self.name.clone(),
            salt: self.salt.to_vec(),
            pws: passwords,
        })
    }

    pub fn save(&self) {
        let path = files::rpwd_path(&self.name);
        let json = serde_json::to_string(&self).expect("Failed to serialize passwords");

        File::create(&path)
            .and_then(|mut f| {
                f.write_all(&json.as_bytes()).expect("Failed to write file");
                Ok(())
            })
            .or_else(|_| Err(format!("Failed to create database {}", path.display())))
            .expect("Failed to create vault file");
    }

    pub fn delete(&self) -> Result<(), String> {
        if cli::yesorno(
            format!("Would you really like to delete the vault {}?", &self.name).as_str(),
        ) && cli::yesorno("Are you reaaaaally sure? It's permanent.")
        {
            files::delete(format!("{}{}", &self.name, VAULT_EXT).as_str())?;
            return Ok(());
        }
        return Err("Did not delete vault".to_string());
    }
}

impl FromStr for LockedVault {
    type Err = ArgParseError;
    fn from_str(s: &str) -> Result<Self, ArgParseError> {
        let fname = files::rpwd_path(s);
        match File::open(&fname) {
            Ok(f) => Ok(serde_json::from_reader::<File, LockedVault>(f)
                .expect("Failed deserializing database")),
            Err(_) => Err(ArgParseError {
                arg: s.to_string(),
                value: fname.display().to_string(),
            }),
        }
    }
}

impl UnlockedVault {
    pub fn new(vault: &str) -> UnlockedVault {
        let mut salt = [0; SALT_LEN];
        crypto::salt(&mut salt);

        UnlockedVault {
            name: vault.to_string(),
            salt: salt.to_vec(),
            pws: HashMap::new(),
        }
    }

    pub fn lock(&self, pass: &str) -> LockedVault {
        let cipher = Cipher::aes_256_cbc();
        let salt = &self.salt;
        let key = crypto::key(&pass.as_bytes(), &salt).unwrap();
        let data = serde_json::to_string_pretty(&self.pws).expect("Failed to serialize passwords");
        let key = key;

        let mut iv = [0; IV_LEN];
        crypto::rand_bytes(&mut iv);
        let ciphertext = encrypt(cipher, &key, Some(&iv), data.as_bytes()).unwrap();
        LockedVault {
            name: self.name.clone(),
            iv: iv.to_vec(),
            salt: self.salt.to_vec(),
            enc: ciphertext,
        }
    }

    pub fn insert(&mut self, id: String, password: String) {
        self.pws.insert(id, password);
    }

    pub fn get(&self, id: String) -> Result<&str, String> {
        match &self.pws.get(&id) {
            Some(pw) => Ok(pw),
            None => Err(format!("Failed to find password {}", id)),
        }
    }
}
