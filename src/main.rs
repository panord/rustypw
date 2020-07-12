use std::env;
use std::result::Result;
use std::string::String;
extern crate rpassword;

mod cli;
mod jconf;
mod session;
mod store;

extern crate dirs;

use jconf::PwEntry;

fn unlock() {
    let pass = rpassword::prompt_password_stdout("Please enter your password (hidden):").unwrap();
    let session = store::unlock(&pass);
    match session {
        Ok(s) => {
            println!("Storing session...\n{}", s);
            session::store_session(&s);
        }
        Err(s) => cli::error(&s),
    }
}

fn get(pws: &Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 3 {
        usage("get");
        return;
    }

    let alias: &str = &_args[2];
    let session: String = session::load_session();

    match store::get_pw_by_alias(&pws, &alias, &session) {
        Ok(pw) => cli::xclip::to_clipboard(&pw),
        Err(msg) => cli::error(&msg),
    };
}

fn alias(pws: &mut Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 4 {
        usage("alias");
        return;
    }

    let name: &str = &_args[2];
    let alias: &str = &_args[3];
    let session: String = session::load_session();

    match store::alias(pws, name, alias, &session) {
        Ok(msg) => println!("{}", msg),
        Err(msg) => cli::error(&msg),
    };
}

fn usage(key: &str) {
    print!("rpw ");
    match key {
        "unlock" => print!("unlock"),
        "get" => print!("get <alias>"),
        "alias" => print!("alias <name> <alias>"),
        _ => print!("unlock | alias | get"),
    }
    println!("");
}

fn run_command(pws: &mut Vec<PwEntry>, args: Vec<String>) {
    if args.len() < 2 {
        usage("");
        return;
    }

    match args[1].as_ref() {
        "unlock" => unlock(),
        "get" => get(pws, args),
        "alias" => alias(pws, args),
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

    let mut pws: Vec<PwEntry>;

    match jconf::read(&path) {
        Ok(db) => pws = db,
        Err(_) => {
            init_rpw(&rpw_d);
            pws = jconf::read(&path).expect("Failed to read db after initialize");
        }
    };

    run_command(&mut pws, args);
    jconf::write(&rpw_d.join(&DB_FNAME), pws).unwrap();
    Ok(())
}
