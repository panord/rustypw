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
use store::LockedVault;

fn open(args: HashMap<String, String>) {
    let mut command = Command::new("open");
    let vname = command.require::<String>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    // Is it better to store this or to expose the full db? Probably neither.
    // Perhaps we can store in intel enclave or something?
    let pass = cli::password("Please enter vault password (hidden):");
    let name = vname.unwrap();
    loop {
        cli::prompt(&format!("{}", name.clone()));
        let args = cli::wait_command();
        if args.len() == 0 {
            continue;
        }

        let mut hargs = command::arg_map(&args[1..]);
        hargs.insert("rpw".to_string(), args[0].clone());
        hargs.insert("vault".to_string(), name.clone());
        hargs.insert("--password".to_string(), pass.clone());
        run_command(hargs);
    }
}

fn new(args: HashMap<String, String>) {
    let mut command = Command::new("new");
    let rvault = command.require::<String>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let rpass = command.hidden::<String>(
        "--password",
        &args,
        "Please choose vault password (hidden):",
    );
    let rvfied = command.hidden::<String>("--verify", &args, "Verify vault password (hidden):");

    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }
    let vault = rvault.unwrap();
    let pass = rpass.unwrap();
    let vfied = rvfied.unwrap();

    match store::new(&vault, &pass, &vfied) {
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

    let nres = command.hidden::<String>("--new-password", &args, "New password (hidden):");
    let mres = command.hidden::<String>("--password", &args, "Enter vault password (hidden):");
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    let alias = ares.unwrap();
    let npass = nres.unwrap();
    let mpass = mres.unwrap();

    match store::add(&vault, &alias, &mpass, &npass) {
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
    match store::delete(&vault) {
        Ok(_) => println!("Deleted vault {}", vault),
        Err(msg) => cli::error(&msg),
    }
}

fn get(args: HashMap<String, String>) {
    let mut command = Command::new("get");
    let vres = command.require::<LockedVault>("vault", &args);
    let idres = command.require::<String>("pw", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let mres =
        command.hidden::<String>("--password", &args, "Please enter vault password (hidden):");
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let sec = command.default::<u64>("sec", &args, 5);
    let id = idres.unwrap();
    let mpass = mres.unwrap();
    let uv = vres.unwrap().unlock(&mpass);
    match uv.get(id.to_string()) {
        Ok(pass) => {
            cli::xclip::to_clipboard(&pass);
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
    std::fs::create_dir_all(&files::rpwd()).expect("Failed to create rpw dir");
    run_command(command::arg_map(&args));
    Ok(())
}
