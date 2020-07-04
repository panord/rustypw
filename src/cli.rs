use std::io::stdin;
use std::string::String;

pub fn error(msg: &str) {
    println!("Error: {}", msg);
}

pub fn yesorno(msg: &str) -> bool {
    let mut ans = String::new();
    println!("{} [y/n]", msg);
    stdin()
        .read_line(&mut ans)
        .expect("Failed reading from stdin");

    match ans.to_ascii_lowercase().replace("\n", "").as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            println!("Please enter y or n");
            yesorno(msg)
        }
    }
}

pub mod xclip {
    use std::io::prelude::*;
    use std::process::Command;

    pub fn to_clipboard(s: &str) {
        let mut clip = Command::new("xclip")
            .stdin(std::process::Stdio::piped())
            .arg("-selection")
            .arg("clipboard")
            .spawn()
            .expect("Failed getting pw");

        clip.stdin
            .as_mut()
            .unwrap()
            .write_all(s.as_bytes())
            .expect("Failed to open stdin");
    }
}
