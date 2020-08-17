extern crate rpassword;
mod cli;
mod store;

use std::env;
use std::result::Result;
use std::string::String;

use store::BwStore;

fn lock(store: &mut BwStore) {
    match store.lock() {
        Ok(s) => {
            println!("Locking session...\n{}", s);
        }
        Err(s) => cli::error(&s),
    };
}

fn unlock(store: &mut BwStore) {
    let pass = cli::password("Please enter your password (hidden):");
    match store.unlock(&pass) {
        Ok(s) => {
            println!("Storing session...\n{}", s);
        }
        Err(s) => cli::error(&s),
    }
}

fn get(store: &BwStore, _args: &[String]) {
    if _args.len() != 3 {
        usage_remote("get");
        return;
    }

    let alias: &str = &_args[2];

    match store.load(&alias) {
        Ok(pw) => cli::xclip::to_clipboard(&pw),
        Err(msg) => cli::error(&msg),
    };
}

fn alias(store: &mut BwStore, _args: &[String]) {
    if _args.len() != 4 {
        usage_remote("alias");
        return;
    }

    let rid: &str = &_args[2];
    let alias: &str = &_args[3];

    match store.store(rid, alias) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn phrase(args: &[String]) {
    if args.len() != 3 {
        usage_local("phrase");
        return;
    }

    let len: u8 = args[2].parse().expect("invalid phrase length");

    match store::bw::phrase(len) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn usage(key: &str) {
    match key {
        _ => print!("local | remote"),
    }
    println!("");
}

fn usage_remote(key: &str) {
    print!("rpw ");
    match key {
        "lock" => print!("lock"),
        "unlock" => print!("unlock"),
        "get" => print!("get <alias>"),
        "alias" => print!("alias <id> <alias>"),
        _ => print!("remote lock|unlock|get|alias|phrase"),
    }
    println!("");
}

fn usage_local(key: &str) {
    match key {
        "new" => print!("new <vault_name>"),
        "get" => print!("get <vault_name> <id>"),
        "add" => print!("add <alias> <length>"),
        _ => print!("new|get|add"),
    }
}

fn local_new(args: &[String]) {
    if args.len() < 2 {
        usage_local("new");
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

fn local_add(args: &[String]) {
    if args.len() < 4 {
        return usage_local("add");
    }

    let vault: &str = &args[1];
    let alias: &str = &args[2];
    let len: u8 = args[3].parse().expect("invalid phrase length");
    let pass = cli::password("Please enter your password (hidden):");

    match store::bw::phrase(len) {
        Err(msg) => cli::error(&msg),
        Ok(phrase) => {
            match store::local::add(vault, alias, &pass, &phrase) {
                Ok(msg) => println!("{}", msg),
                Err(msg) => cli::error(&msg),
            };
        }
    }
}

fn local_get(args: &[String]) {
    if args.len() < 3 {
        usage_local("get");
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

fn run_local(args: &[String]) {
    if args.len() < 2 {
        usage_local("");
        return;
    }

    match args[1].as_ref() {
        "new" => local_new(&args[1..]),
        "get" => local_get(&args[1..]),
        "add" => local_add(&args[1..]),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn run_remote(args: &[String]) {
    if args.len() < 2 {
        usage_remote("");
        return;
    }

    let mut store = BwStore::new();
    match args[1].as_ref() {
        "lock" => lock(&mut store),
        "unlock" => unlock(&mut store),
        "get" => get(&mut store, &args),
        "phrase" => phrase(&args),
        "alias" => alias(&mut store, &args),
        "help" => usage_remote(""),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn run_command(args: &[String]) {
    if args.len() < 2 {
        usage("");
        return;
    }
    match args[1].as_ref() {
        "remote" => run_remote(&args[1..]),
        "local" => run_local(&args[1..]),
        "help" => usage(""),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    run_command(&args);
    Ok(())
}
