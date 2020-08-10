mod crypto;
mod vault;

use crate::cli;
use openssl::symm::{decrypt, encrypt, Cipher};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::string::String;

const SALT_LEN: usize = 256;
const IV_LEN: usize = 16;
const VAULT_EXT: &'static str = ".vlt";
const VAULT_ID: &'static str = "vault-token";
const VAULT_TOKEN: &'static str = "all-is-well";
const RPW_DIR: &'static str = ".rpw.d";

#[derive(Serialize, Deserialize)]
pub struct PwEntry {
    pub id: String,
    pub password: String,
}

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
    pub pws: Vec<PwEntry>,
}

impl LockedVault {
    pub fn unlock(&self, key: &[u8]) -> UnlockedVault {
        let cipher = Cipher::aes_256_cbc();
        let data = &self.enc;
        let iv = &self.iv;
        let json = String::from_utf8(decrypt(cipher, key, Some(&iv), data).unwrap()).unwrap();
        let passwords: Vec<PwEntry> = serde_json::from_str(&json).unwrap();

        UnlockedVault {
            name: self.name.clone(),
            salt: self.salt.to_vec(),
            pws: passwords,
        }
    }
}

impl UnlockedVault {
    pub fn lock(&self, key: &[u8]) -> LockedVault {
        let cipher = Cipher::aes_256_cbc();
        let data = serde_json::to_string_pretty(&self.pws).expect("Failed to serialize passwords");
        let key = key;

        let mut iv = [0; IV_LEN];
        crypto::rand_bytes(&mut iv);
        let ciphertext = encrypt(cipher, key, Some(&iv), data.as_bytes()).unwrap();
        LockedVault {
            name: self.name.clone(),
            iv: iv.to_vec(),
            salt: self.salt.to_vec(),
            enc: ciphertext,
        }
    }

    pub fn add(&mut self, id: String, password: String) {
        self.pws.push(PwEntry {
            id: id,
            password: password,
        });
    }
}

pub fn get_pw(name: &str, id: &str, pass: &str) -> Result<String, String> {
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    std::fs::create_dir_all(&rpw_d).expect("Failed to create rpw dir");
    let path = rpw_d.join(format!("{}{}", name, VAULT_EXT));
    let lv: LockedVault = read(&path).expect("Could not find vault");

    match crypto::key(pass.as_bytes(), &lv.salt) {
        Ok(key) => {
            let uv: UnlockedVault = lv.unlock(&key);
            for pw in uv.pws {
                if pw.id == id {
                    println!("{}", pw.password);
                    return Ok(pw.password);
                }
            }
        }
        Err(_) => return Err("Failed to derive key".to_string()),
    };
    return Err("Could not find password".to_string());
}

pub fn new(name: &str, pass: &str, vfied: &str) -> Result<(), String> {
    if pass != vfied {
        return Err("Passwords are not equal".to_string());
    }
    let vname = format!("{}{}", name, VAULT_EXT);
    // Should get this from user storage
    let mut salt = [0; SALT_LEN];
    crypto::salt(&mut salt);
    if name != "test" && vault::exists(&vname) {
        return Err(format!("Vault {} already exists.", vname));
    }

    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    std::fs::create_dir_all(&rpw_d).expect("Failed to create rpw dir");
    let path = rpw_d.join(format!("{}{}", name, VAULT_EXT));

    match crypto::key(pass.as_bytes(), &salt) {
        Ok(key) => write(
            &path,
            &UnlockedVault {
                name: name.to_string(),
                salt: salt.to_vec(),
                pws: vec![PwEntry {
                    id: VAULT_ID.to_string(),
                    password: VAULT_TOKEN.to_string(),
                }],
            }
            .lock(&key),
        ),
        Err(_) => Err(format!("Failed creating new vault {}", name)),
    }
}

pub fn delete(name: &str) -> Result<(), String> {
    if cli::yesorno(format!("Would you really like to delete the vault {}?", name).as_str())
        && cli::yesorno("Are you reaaaaally sure? It's permanent.")
    {
        vault::delete(format!("{}{}", name, VAULT_EXT).as_str())?;
        return Ok(());
    }
    return Err("Did not delete vault".to_string());
}

pub fn add(vault: &str, alias: &str, pass: &str, new_pass: &str) -> Result<String, String> {
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    std::fs::create_dir_all(&rpw_d).expect("Failed to create rpw dir");
    let path = rpw_d.join(format!("{}{}", vault, VAULT_EXT));

    let vault: LockedVault = read(&path).unwrap();
    let salt = &vault.salt;
    let key = crypto::key(pass.as_bytes(), &salt).unwrap();
    let mut unlocked = vault.unlock(&key);
    unlocked.add(alias.to_string(), new_pass.to_string());
    write(&path, &unlocked.lock(&key)).unwrap();

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
