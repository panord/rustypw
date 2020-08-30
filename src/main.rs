extern crate rpassword;
mod cli;
mod store;

use std::env;
use std::process::Command;
use std::result::Result;
use std::string::String;

fn usage(key: &str) {
    match key {
        "open" => print!("open <vault_name>"),
        "new" => print!("new <vault_name>"),
        "get" => print!("get <vault_name> <id>"),
        "add" => print!("add <vault_name> <alias> <length>"),
        "clear" => print!("clear <seconds>"),
        "delete" => print!("delete <vault_name>"),
        _ => print!("open|new|get|add|clear|delete"),
    }
    println!("");
}

fn open(args: &[String]) {
    if args.len() < 2 {
        usage("open");
        return;
    }
    let pass = cli::password("Please choose your password (hidden):");
    let name: &str = &args[1];

    match store::local::open(&name, &pass) {
        Ok(_) => println!("{}", format!("Vault {} opened", name)),
        Err(msg) => cli::error(&msg),
    }

    loop {
        cli::prompt(&format!("{}", name));
        let args = cli::wait_command();
        run_command(&args);
    }
}

fn new(args: &[String]) {
    if args.len() < 2 {
        usage("new");
        return;
    }

    let pass = cli::password("Please choose your password (hidden):");
    let vfied = cli::password("Please verify your password (hidden):");

    let name: &str = &args[1];

    match store::local::new(&name, &pass, &vfied) {
        Ok(_) => println!("{}", format!("New vault {} created", name)),
        Err(msg) => cli::error(&msg),
    }
}

pub fn password(len: u8) -> Result<String, String> {
    let out = Command::new("pwgen")
        .arg("-N")
        .arg("1")
        .arg(len.to_string())
        .output()
        .expect("Failed getting pw");

    match out.status.code().unwrap() {
        0 => Ok(std::str::from_utf8(&out.stdout).unwrap().to_string()),
        _ => Err(std::str::from_utf8(&out.stderr).unwrap().to_string()),
    }
}

fn add(args: &[String]) {
    if args.len() < 3 {
        usage("add");
        return;
    }

    let vault: &str = &args[1];
    let alias: &str = &args[2];
    let mpass = cli::password("Please enter your password (hidden):");
    let pw: String = password(15).expect("Failed generating password");
    match store::local::add(vault, alias, &mpass, &pw) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn delete(args: &[String]) {
    if args.len() < 2 {
        usage("delete");
        return;
    }
    let vault: &str = &args[1];
    match store::local::delete(vault) {
        Ok(_) => println!("Deleted vault {}", vault),
        Err(msg) => cli::error(&msg),
    }
}

fn get(args: &[String]) {
    if args.len() < 3 {
        usage("get");
        return;
    }

    let vault: &str = &args[1];
    let id: &str = &args[2];
    let pass = cli::password("Please enter your password (hidden):");
    let clear_in = 5;

    let uv = store::local::open(vault, &pass);
    if uv.is_err() {
        cli::error("Could not find vault");
        return;
    }
    match uv.unwrap().get(id.to_string()) {
        Ok(pw) => {
            cli::xclip::to_clipboard(&pw);
            println!("Clearing clipboard in {} seconds", clear_in);
            cli::xclip::clear(clear_in);
        }
        Err(msg) => cli::error(&msg),
    };
}

fn clear(args: &[String]) {
    if args.len() < 2 {
        usage("clear");
        return;
    }
    let dur = std::time::Duration::from_secs(args[1].parse().unwrap());
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
}

fn run_command(args: &[String]) {
    if args.len() < 2 {
        usage("");
        return;
    }

    match args[1].as_ref() {
        "open" => open(&args[1..]),
        "new" => new(&args[1..]),
        "add" => add(&args[1..]),
        "get" => get(&args[1..]),
        "clear" => clear(&args[1..]),
        "delete" => delete(&args[1..]),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    run_command(&args);
    Ok(())
}
