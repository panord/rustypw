use anyhow::{Error, Result};

pub fn salt(buf: &mut [u8]) {
    openssl::rand::rand_bytes(buf).unwrap();
}

pub fn rand_bytes(buf: &mut [u8]) {
    openssl::rand::rand_bytes(buf).unwrap();
}

pub fn key(pass: &[u8], salt: &[u8]) -> Result<Vec<u8>, Error> {
    let config = argon2::Config::default();

    Ok(argon2::hash_raw(pass, salt, &config)?)
}
