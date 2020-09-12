extern crate rpassword;
mod cli;
mod command;
mod files;
mod store;

use command::Command;
use std::collections::HashMap;
use std::env;
use std::result::Result;
use std::string::String;
use store::local::UnlockedVault;

fn open(args: HashMap<String, String>) {
    let mut command = Command::new("open");
    let vres = command.require::<String>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    let pass = cli::password("Please choose your password (hidden):");
    let uvres = store::local::open(&vault, &pass);
    if uvres.is_err() {
        cli::error("Could not find vault");
        return;
    }

    let uv: &mut UnlockedVault = &mut uvres.unwrap();
    loop {
        cli::prompt(&format!("{}", &vault));
        let args = cli::wait_command();
        if args.len() == 0 {
            continue;
        }
    }
}

fn new(args: HashMap<String, String>) {
    let mut command = Command::new("new");
    let rvault = command.require::<String>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let rpass =
        command.hidden::<String>("--password", &args, "Please choose your password (hidden):");
    let rvfied = command.hidden::<String>("--verify", &args, "Verify your password (hidden):");

    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let vault = rvault.unwrap();
    let pass = rpass.unwrap();
    let vfied = rvfied.unwrap();

    match store::local::new(&vault, &pass, &vfied) {
        Ok(_) => println!("{}", format!("New vault {} created", vault)),
        Err(msg) => cli::error(&msg),
    }
}

fn add(args: HashMap<String, String>) {
    let mut command = Command::new("add");
    let vres = command.require::<String>("vault", &args);
    let ares = command.require::<String>("alias", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    let alias = ares.unwrap();

    let mpass = cli::password("Please enter your vault password (hidden):");
    let pw = cli::password("New password (hidden):");

    match store::local::add(&vault, &alias, &mpass, &pw) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn delete(args: HashMap<String, String>) {
    let mut command = Command::new("delete");
    let vres = command.require::<String>("vault", &args);

    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    match store::local::delete(&vault) {
        Ok(_) => println!("Deleted vault {}", vault),
        Err(msg) => cli::error(&msg),
    }
}

fn get(args: HashMap<String, String>) {
    let mut command = Command::new("get");
    let vres = command.require::<String>("vault", &args);
    let idres = command.require::<String>("pw", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    let sec = command.default::<u64>("sec", &args, 5);
    let id = idres.unwrap();
    let pass = cli::password("Please enter your password (hidden):");
    let uv = store::local::open(&vault, &pass);
    if uv.is_err() {
        cli::error("Could not find vault");
        return;
    }

    match uv.unwrap().get(id.to_string()) {
        Ok(pw) => {
            cli::xclip::to_clipboard(&pw);
            println!("Clearing clipboard in {} seconds", sec);
            cli::xclip::clear(sec);
        }
        Err(msg) => cli::error(&msg),
    };
}

fn clear(args: HashMap<String, String>) {
    let mut command = Command::new("get");
    let secres = command.require::<u64>("sec", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let dur = std::time::Duration::from_secs(secres.unwrap());
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
}

fn run_command(args: HashMap<String, String>) {
    match args.get("rpw") {
        Some(command) => match command.as_ref() {
            "open" => open(args),
            "new" => new(args),
            "add" => add(args),
            "get" => get(args),
            "clear" => clear(args),
            "delete" => delete(args),
            _ => println!("Unknown command or context {} not implemented", command),
        },
        None => {
            println!("open|new|get|add|clear|delete");
        }
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    run_command(command::arg_map(&args));
    Ok(())
}
