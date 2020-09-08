extern crate rpassword;
mod cli;
mod store;

use std::env;
use std::result::Result;
use std::string::String;

fn usage(key: &str) {
    match key {
        "open" => print!("open <vault_name>"),
        "new" => print!("new <vault_name>"),
        "get" => print!("get <vault_name> <id>"),
        "add" => print!("add <vault_name> <alias>"),
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
        cli::prompt(&format!("{}", &name));
        let args = cli::wait_command();
        if args.len() < 2 {
            continue;
        }
        run_command(
            &[
                vec!["rpw".to_string(), args[0].clone(), name.to_string()],
                args[1..].to_vec(),
            ]
            .concat(),
        );
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

fn add(args: &[String]) {
    if args.len() < 3 {
        usage("add");
        return;
    }

    let vault: &str = &args[1];
    let alias: &str = &args[2];
    let pw: String = cli::password("Please enter your new password (hidden):");
    let mpass = cli::password("Please enter your password (hidden):");
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
