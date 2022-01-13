use anyhow::Result;
use std::path::PathBuf;

const RPW_DIR: &str = ".rpw.d";

pub fn rpwd() -> PathBuf {
    dirs::home_dir().unwrap().join(RPW_DIR)
}

pub fn rpwd_path(name: &str) -> PathBuf {
    rpwd().join(name.to_string())
}

pub fn delete(name: &str) -> Result<()> {
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    let fname = rpw_d.join(&name);
    std::fs::remove_file(&fname)?;
    Ok(())
}
