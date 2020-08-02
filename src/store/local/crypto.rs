pub fn salt(buf: &mut [u8]) {
    openssl::rand::rand_bytes(buf).unwrap();
}

pub fn key(pass: &[u8], salt: &[u8]) -> Result<Vec<u8>, String> {
    let config = argon2::Config::default();

    match argon2::hash_raw(pass, salt, &config) {
        Ok(key) => Ok(key),
        Err(_) => Err("Failed hashing".to_string()),
    }
}
