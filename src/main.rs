extern crate rpassword;
mod cli;
mod command;
mod store;

use command::Command;
use std::env;
use std::result::Result;
use std::string::String;
use store::local::UnlockedVault;

fn open(args: &[String]) {
    let mut command = Command::new("open");
    let vres = command.require::<String>("vault", args);
    if command.is_ok() {
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
        run_vault_command(uv, &[args[..].to_vec()].concat());
    }
}

fn new(args: &[String]) {
    let mut command = Command::new("new");
    let nres = command.require::<String>("vault", args);
    if command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let name = nres.unwrap();
    let pass = cli::password("Please choose your password (hidden):");
    let vfied = cli::password("Please verify your password (hidden):");

    match store::local::new(&name, &pass, &vfied) {
        Ok(_) => println!("{}", format!("New vault {} created", name)),
        Err(msg) => cli::error(&msg),
    }
}

fn add(args: &[String]) {
    let mut command = Command::new("add");
    let vres = command.require::<String>("vault", args);
    let ares = command.require::<String>("alias", args);
    if command.is_ok() {
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

fn delete(args: &[String]) {
    let mut command = Command::new("delete");
    let vres = command.require::<String>("vault", args);

    if command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    match store::local::delete(&vault) {
        Ok(_) => println!("Deleted vault {}", vault),
        Err(msg) => cli::error(&msg),
    }
}

fn get(args: &[String]) {
    let mut command = Command::new("get");
    let vres = command.require::<String>("vault", args);
    let idres = command.require::<String>("pw", args);
    if command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    let id = idres.unwrap();
    let pass = cli::password("Please enter your password (hidden):");
    let uv = store::local::open(&vault, &pass);
    if uv.is_err() {
        cli::error("Could not find vault");
        return;
    }

    let clear_in = 5;

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
    let mut command = Command::new("get");
    let secres = command.require::<u64>("sec", args);
    if command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let dur = std::time::Duration::from_secs(secres.unwrap());
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
}

fn run_vault_command(uv: &mut UnlockedVault, args: &[String]) {
    if args.len() < 1 {
        return;
    }
    match args[0] {
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn run_command(args: &[String]) {
    if args.len() < 2 {
        println!("open|new|get|add|clear|delete");
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
