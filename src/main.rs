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
use store::PwStore;

fn lock(store: &mut dyn PwStore) {
    match store.lock() {
        Ok(s) => {
            println!("Locking session...\n{}", s);
        }
        Err(s) => cli::error(&s),
    };
}

fn unlock(store: &mut dyn PwStore) {
    let pass = cli::password("Please enter your password (hidden):");
    match store.unlock(&pass) {
        Ok(s) => {
            println!("Storing session...\n{}", s);
        }
        Err(s) => cli::error(&s),
    }
}

fn get(store: &dyn PwStore, _args: Vec<String>) {
    if _args.len() != 3 {
        usage("get");
        return;
    }

    let alias: &str = &_args[2];

    match store.load(&alias) {
        Ok(pw) => cli::xclip::to_clipboard(&pw),
        Err(msg) => cli::error(&msg),
    };
}

fn alias(store: &mut dyn PwStore, _args: Vec<String>) {
    if _args.len() != 4 {
        usage("alias");
        return;
    }

    let rid: &str = &_args[2];
    let alias: &str = &_args[3];

    match store.store(rid, alias) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn phrase(args: Vec<String>) {
    if args.len() != 3 {
        usage("phrase");
        return;
    }

    let len: u8 = args[2].parse().expect("invalid phrase length");

    match bw::phrase(len) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn usage(key: &str) {
    print!("rpw ");
    match key {
        "lock" => print!("lock"),
        "unlock" => print!("unlock"),
        "get" => print!("get <alias>"),
        "alias" => print!("alias <id> <alias>"),
        "phrase" => print!("phrase <length>"),
        _ => print!("lock|unlock|get|alias|phrase"),
    }
    println!("");
}

fn run_command(store: &mut dyn PwStore, args: Vec<String>) {
    if args.len() < 2 {
        usage("");
        return;
    }

    match args[1].as_ref() {
        "lock" => lock(store),
        "unlock" => unlock(store),
        "get" => get(store, args),
        "alias" => alias(store, args),
        "phrase" => phrase(args),
        _ => println!("Unknown command {} not implemented", args[1]),
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
    let rpw_d = dirs::home_dir().unwrap().join(RPW_DIR);
    let path = rpw_d.join(&DB_FNAME);

    let pws: Vec<PwEntry>;

    match jconf::read(&path) {
        Ok(db) => pws = db,
        Err(_) => {
            init_rpw(&rpw_d);
            pws = jconf::read(&path).expect("Failed to read db after initialize");
        }
    };
    let mut store = BwStore { pws: pws };
    run_command(&mut store, args);
    jconf::write(&rpw_d.join(&DB_FNAME), store.pws).unwrap();
    Ok(())
}
