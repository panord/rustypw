const RPW_DIR: &'static str = ".rpw.d";

pub fn exists(name: &str) -> bool {
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    let fname = rpw_d.join(&name);
    std::path::Path::new(&fname).exists()
}

pub fn delete(name: &str) -> Result<(), String> {
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    let fname = rpw_d.join(&name);
    match std::fs::remove_file(fname) {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed to delete vault".to_string()),
    }
}
