const RPW_DIR: &'static str = ".rpw.d";
use anyhow::Result;
use std::path::PathBuf;

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
