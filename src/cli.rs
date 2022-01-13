use clap::{App, Arg, SubCommand};
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

pub fn build() -> clap::App<'static, 'static> {
    let mut app = App::new("rpw - the rusty password manager")
        .version("2021")
        .author("Patrik Lundgren <patrik.lundgren@outlook.com>")
        .about(
            "rpw is a small cli-only password manager for your terminal
            copy pasting needs.",
        );

    app = app.subcommand(
        SubCommand::with_name("open")
            .about("Open a password encrypted vault.")
            .arg(Arg::with_name("vault").required(true).takes_value(true)),
    );

    app = app.subcommand(
        SubCommand::with_name("clear")
            .about("Clear the clipboard register.")
            .arg(Arg::with_name("sec").takes_value(true).required(true)),
    );

    app = app.subcommand(
        SubCommand::with_name("get")
            .about("Decrypt the vault and fetch a password to the clipboard.")
            .arg(Arg::with_name("vault").long("vault").short("v"))
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            )
            .arg(Arg::with_name("alias").required(true).takes_value(true)),
    );

    app = app.subcommand(
        SubCommand::with_name("list")
            .about("List the stored passwords of a vault by alias.")
            .arg(
                Arg::with_name("vault")
                    .long("vault")
                    .short("v")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            ),
    );

    app = app.subcommand(
        SubCommand::with_name("export")
            .about("Export the vault to a plain-text json format.")
            .arg(
                Arg::with_name("vault")
                    .long("vault")
                    .short("v")
                    .takes_value(true),
            )
            .arg(Arg::with_name("file-path").required(true).takes_value(true))
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            ),
    );

    app = app.subcommand(
        SubCommand::with_name("new")
            .about("Create a new password encrypted vault.")
            .arg(
                Arg::with_name("vault")
                    .long("vault")
                    .short("v")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("verify")
                    .long("verify")
                    .short("v")
                    .takes_value(true),
            ),
    );

    app = app.subcommand(
        SubCommand::with_name("delete")
            .about("Delete an existing vault.")
            .arg(
                Arg::with_name("vault")
                    .long("vault")
                    .short("v")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("verify")
                    .long("verify")
                    .short("v")
                    .takes_value(true),
            ),
    );

    app = app.subcommand(
        SubCommand::with_name("add")
            .about("Add a password to the vault.")
            .arg(
                Arg::with_name("vault")
                    .long("vault")
                    .short("v")
                    .takes_value(true),
            )
            .arg(Arg::with_name("alias").required(true).takes_value(true))
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .short("p")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("new-password")
                    .long("new-password")
                    .short("n")
                    .takes_value(true),
            ),
    );

    app
}
