use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::string::String;

pub fn error(msg: &str) {
    println!("Error: {}", msg);
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

    match ans.to_ascii_lowercase().replace('\n', "").as_str() {
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

    #[cfg(target_os = "macos")]
    pub fn to_clipboard(s: &str) {
        let mut clip = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed getting pw");

        clip.stdin
            .as_mut()
            .unwrap()
            .write_all(s.as_bytes())
            .expect("Failed to open stdin");
    }

    #[cfg(target_os = "linux")]
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

pub mod cli {
    use clap::{Parser, Subcommand};

    #[derive(Parser, Debug, Clone)]
    pub struct OpenArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(short, long)]
        pub password: String,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct ClearArgs {
        #[arg(short, long)]
        pub sec: u64,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct GetArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(index = 2)]
        pub alias: String,
        #[arg(short, long)]
        pub sec: u32,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct ListArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(short, long)]
        pub password: String,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct ImportArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(index = 2)]
        pub file_path: String,
        #[arg(short, long)]
        pub password: Option<String>,
        #[arg(long)]
        pub verify: Option<String>,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct ExportArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(index = 2)]
        pub file_path: String,
        #[arg(short, long)]
        pub password: String,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct NewArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(short, long)]
        pub password: Option<String>,
        #[arg(long)]
        pub verify: Option<String>,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct DelArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(short, long)]
        pub password: Option<String>,
        #[arg(long)]
        pub verify: Option<String>,
    }

    #[derive(Parser, Debug, Clone)]
    pub struct AddArgs {
        #[arg(index = 1)]
        pub vault: String,
        #[arg(index = 2)]
        pub alias: String,
        #[arg(short, long)]
        pub password: Option<String>,
        #[arg(short, long)]
        pub new_password: Option<String>,
    }

    #[derive(Subcommand, Debug, Clone)]
    pub enum Command {
        Open(OpenArgs),
        Clear(ClearArgs),
        Get(GetArgs),
        List(ListArgs),
        Import(ImportArgs),
        Export(ExportArgs),
        New(NewArgs),
        Delete(DelArgs),
        Add(AddArgs),
    }

    #[derive(Parser, Debug)]
    #[clap(about, version, author)]
    pub struct RpwCli {
        #[command(subcommand)]
        pub command: Command,
    }

    impl RpwCli {
        pub fn cli() -> Self {
            RpwCli::parse()
        }
    }
}
