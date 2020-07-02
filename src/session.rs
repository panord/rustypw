use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn store_session(session: &str) {
    let fname = Path::new("/tmp/rpw-session");
    let mut file = File::create(fname).expect("Failed to create session");
    file.write_all(session.as_bytes())
        .expect("Failed to write session");
}

pub fn load_session() -> String {
    let fname: &Path = Path::new("/tmp/rpw-session");
    std::fs::read_to_string(fname).expect("failed to load session")
}
