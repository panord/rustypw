extern crate rpassword;
mod cli;
mod store;

use std::env;
use std::result::Result;
use std::string::String;

fn usage(key: &str) {
    match key {
        "new" => print!("new <vault_name>"),
        "get" => print!("get <vault_name> <id>"),
        "add" => print!("add <alias> <length>"),
        "delete" => print!("delete <vault_name>"),
        _ => print!("new|get|add"),
    }
    println!("");
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

    match store::local::get_pw(vault, id, &pass) {
        Ok(pw) => cli::xclip::to_clipboard(&pw),
        Err(msg) => cli::error(&msg),
    };
}

fn run_command(args: &[String]) {
    if args.len() < 2 {
        usage("");
        return;
    }

    match args[1].as_ref() {
        "new" => new(&args[1..]),
        "get" => get(&args[1..]),
        "delete" => delete(&args[1..]),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    run_command(&args);
    Ok(())
}
