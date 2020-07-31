use std::env;
use std::result::Result;
use std::string::String;
extern crate rpassword;

mod bw;
mod cli;
mod jconf;
mod store;

extern crate dirs;

use store::bw::BwStore;
use store::PwEntry;

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
        usage_remote("phrase");
        return;
    }

    let len: u8 = args[2].parse().expect("invalid phrase length");

    match bw::phrase(len) {
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
        "phrase" => print!("phrase <length>"),
        _ => print!("remote lock|unlock|get|alias|phrase"),
    }
    println!("");
}

fn usage_local(key: &str) {
    match key {
        _ => println!("new|get"),
    }
}

fn run_local(args: &[String]) {
    if args.len() < 2 {
        usage_local("");
        return;
    }

    match args[1].as_ref() {
        "" => println!("Not implemented"),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }
}

fn run_remote(args: &[String]) {
    if args.len() < 2 {
        usage_remote("");
        return;
    }

    // TODO: Move to remote init?
    let pws: Vec<PwEntry>;
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    let path = rpw_d.join(&DB_FNAME);
    match jconf::read(&path) {
        Ok(db) => pws = db,
        Err(_) => {
            init_rpw(&rpw_d);
            pws = jconf::read(&path).expect("Failed to read db after initialize");
        }
    };

    let mut store = BwStore { pws: pws };

    match args[1].as_ref() {
        "lock" => lock(&mut store),
        "unlock" => unlock(&mut store),
        "get" => get(&mut store, &args),
        "alias" => alias(&mut store, &args),
        "phrase" => phrase(&args),
        "help" => usage_remote(""),
        _ => println!("Unknown command or context {} not implemented", args[1]),
    }

    // TODO: Move to.. elsewhere???
    // intit -> cmd -> init
    jconf::write(&rpw_d.join(&DB_FNAME), store.pws).unwrap();
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

const DB_FNAME: &'static str = "rusty.db";
const RPW_DIR: &'static str = ".rpw.d";

fn init_rpw(rpw_d: &std::path::Path) {
    std::fs::create_dir_all(&rpw_d).expect("Failed to create rpw dir");
    let path = rpw_d.join(&DB_FNAME);
    jconf::init(&path).expect("Failed to create rpw config");
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    run_command(&args);
    Ok(())
}
