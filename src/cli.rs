use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::string::String;

pub fn error(msg: &str) {
    println!("Error: {}", msg);
}

pub fn prompt(pr: &str) {
    stdout()
        .write_all(format!("rpw/{}# ", pr).as_bytes())
        .expect("Failed writing to stdout");
    stdout().flush().expect("Failed to flush stdout");
}

pub fn wait_command() -> Vec<String> {
    let mut cmd = String::new();
    stdin()
        .read_line(&mut cmd)
        .expect("Failed reading from stdin");
    cmd.split_whitespace().map(String::from).collect()
}

pub fn yesorno(msg: &str) -> bool {
    let mut ans = String::new();
    stdout()
        .write_all(format!("{} [y/n] ", msg).as_bytes())
        .expect("Failed writing to stdout");
    stdout().flush().expect("Failed to flush stdout");
    stdin()
        .read_line(&mut ans)
        .expect("Failed reading from stdin");

    match ans.to_ascii_lowercase().replace("\n", "").as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => yesorno("Please eneter y or n"),
    }
}
pub fn password(msg: &str) -> String {
    rpassword::prompt_password_stdout(msg).unwrap()
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

    pub fn clear(sleep: u64) {
        Command::new("rpw")
            .arg("clear")
            .arg(sleep.to_string())
            .spawn()
            .expect("Failed getting pw");
    }
}
