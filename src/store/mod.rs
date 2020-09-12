mod crypto;
use crate::cli;
use crate::files;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
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
    pub fn unlock(&self, pass: &str) -> UnlockedVault {
        let salt = &self.salt;
        let key = crypto::key(&pass.as_bytes(), &salt).unwrap();
        let cipher = Cipher::aes_256_cbc();
        let data = &self.enc;
        let iv = &self.iv;
        let json = String::from_utf8(decrypt(cipher, &key, Some(&iv), data).unwrap()).unwrap();
        let passwords: HashMap<String, String> = serde_json::from_str(&json).unwrap();

        UnlockedVault {
            name: self.name.clone(),
            salt: self.salt.to_vec(),
            pws: passwords,
        }
    }
}

impl UnlockedVault {
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

pub fn open(vault: &str, pass: &str) -> Result<UnlockedVault, String> {
    let path = files::rpwd_path(vault);
    let lv: LockedVault = read(&path).expect("Could not find vault");
    let uv: UnlockedVault = lv.unlock(pass);
    Ok(uv)
}

pub fn new(vault: &str, pass: &str, vfied: &str) -> Result<(), String> {
    if pass != vfied {
        return Err("Passwords are not equal".to_string());
    }
    let mut salt = [0; SALT_LEN];
    crypto::salt(&mut salt);
    if files::exists(&vault) {
        return Err(format!("Vault {} already exists.", vault));
    }

    let path = files::rpwd_path(vault);
    let lv = UnlockedVault {
        name: vault.to_string(),
        salt: salt.to_vec(),
        pws: HashMap::new(),
    }
    .lock(&pass);
    write(&path, &lv)
}
pub fn delete(name: &str) -> Result<(), String> {
    if cli::yesorno(format!("Would you really like to delete the vault {}?", name).as_str())
        && cli::yesorno("Are you reaaaaally sure? It's permanent.")
    {
        files::delete(format!("{}{}", name, VAULT_EXT).as_str())?;
        return Ok(());
    }
    return Err("Did not delete vault".to_string());
}

pub fn add(vault: &str, alias: &str, pass: &str, new_pass: &str) -> Result<String, String> {
    let path = files::rpwd_path(vault);
    let vault: LockedVault = read(&path).unwrap();
    let mut unlocked = vault.unlock(pass);
    unlocked.insert(alias.to_string(), new_pass.to_string());
    write(&path, &unlocked.lock(&pass)).unwrap();

    Ok(format!("Entered {}", alias))
}

fn read(fname: &Path) -> Result<LockedVault, String> {
    match File::open(&fname) {
        Ok(f) => {
            Ok(serde_json::from_reader::<File, LockedVault>(f)
                .expect("Failed deserializing database"))
        }
        Err(_) => Err(format!("Failed reading database {}", fname.display())),
    }
}

fn write(fname: &Path, vault: &LockedVault) -> Result<(), String> {
    let json = serde_json::to_string(&vault).expect("Failed to serialize passwords");

    File::create(fname)
        .and_then(|mut f| {
            f.write_all(&json.as_bytes()).expect("Failed to write file");
            Ok(())
        })
        .or_else(|_| Err(format!("Failed to create database {}", fname.display())))
}
