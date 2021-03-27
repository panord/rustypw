mod crypto;
use crate::cli;
use crate::files;
use anyhow::{Context, Result};
use openssl::base64::decode_block;
use openssl::base64::encode_block;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::String;

const SALT_LEN: usize = 256;
const IV_LEN: usize = 16;
const VAULT_EXT: &'static str = ".vlt";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub id: String,
    pub pw: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockedVault {
    pub name: String,
    pub iv: String,
    pub salt: String,
    pub enc: String,
}

pub struct UnlockedVault {
    pub name: String,
    pub salt: Vec<u8>,
    pub pws: HashMap<String, String>,
}

impl LockedVault {
    pub fn unlock(&self, pass: &str) -> Result<UnlockedVault> {
        let salt = decode_block(&self.salt).context("Failed to decode salt")?;
        let data = decode_block(&self.enc).context("Failed to decode data")?;
        let iv = decode_block(&self.iv).context("Failed to decode iv")?;
        let key = crypto::key(&pass.as_bytes(), &salt)?;
        let cipher = Cipher::aes_256_cbc();
        let plain =
            decrypt(cipher, &key, Some(&iv), &data).context("Cipher could not be decrypted")?;

        let json = String::from_utf8(plain).context("UTF8 conversion failed")?;
        let passwords: HashMap<String, String> =
            serde_json::from_str(&json).context("JSON conversion failed")?;

        Ok(UnlockedVault {
            name: self.name.clone(),
            salt: salt,
            pws: passwords,
        })
    }

    pub fn exists(&self) -> bool {
        let path = files::rpwd_path(&format!("{}{}", self.name, VAULT_EXT));
        path.exists()
    }

    pub fn save(&self) -> Result<()> {
        let path = files::rpwd_path(&format!("{}{}", self.name, VAULT_EXT));
        let json = serde_json::to_string(&self).context("Failed to serialize passwords")?;
        Ok(File::create(&path)
            .and_then(|mut f| f.write_all(&json.as_bytes()))
            .context("Failed to save vault")?)
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
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fname = files::rpwd_path(&format!("{}{}", s, VAULT_EXT));
        let f = File::open(&fname)?;
        Ok(serde_json::from_reader::<File, LockedVault>(f)?)
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

    pub fn import(&mut self, path: &PathBuf) -> Result<Vec<Password>> {
        let f = File::open(&path)?;
        let pws: Vec<Password> = serde_json::from_reader::<File, Vec<Password>>(f)?;

        let dup = pws
            .iter()
            .filter(|p| !self.try_insert(p.id.clone(), p.pw.clone()))
            .map(|p| p.clone())
            .collect();

        Ok(dup)
    }

    pub fn export(&self, path: &PathBuf) -> Result<()> {
        let pws: Vec<Password> = self
            .pws
            .iter()
            .map(|(k, v)| Password {
                id: k.to_string(),
                pw: v.to_string(),
            })
            .collect();
        let json = serde_json::to_string_pretty(&pws).context("Failed to serialize vault")?;
        File::create(&path).and_then(|mut f| f.write_all(&json.as_bytes()))?;
        Ok(())
    }

    pub fn lock(&self, pass: &str) -> Result<LockedVault> {
        let cipher = Cipher::aes_256_cbc();
        let salt = &self.salt;
        let key = crypto::key(&pass.as_bytes(), &salt).context("Failed to derive key")?;
        let data =
            serde_json::to_string_pretty(&self.pws).context("Failed to serialize passwords")?;
        let key = key;

        let mut iv = [0; IV_LEN];
        crypto::rand_bytes(&mut iv);
        let ciphertext = encrypt(cipher, &key, Some(&iv), data.as_bytes())
            .context("Failed to encrypt plaintext")?;
        Ok(LockedVault {
            name: self.name.clone(),
            iv: encode_block(&iv.to_vec()),
            salt: encode_block(&self.salt.to_vec()),
            enc: encode_block(&ciphertext),
        })
    }

    pub fn try_insert(&mut self, id: String, password: String) -> bool {
        if !self.pws.contains_key(&id) {
            self.pws.insert(id, password);
            return true;
        }
        return false;
    }

    pub fn insert(&mut self, id: String, password: String) {
        self.pws.insert(id, password);
    }

    pub fn get(&self, id: String) -> Result<&String, String> {
        match &self.pws.get(&id) {
            Some(pw) => Ok(pw),
            None => Err(format!("Failed to find password {}", id)),
        }
    }
}
