use serde::{Deserialize, Serialize};
use std::env;
use std::fs::*;
use std::io::prelude::*;
use std::io::stdin;
use std::process::Command;
use std::result::Result;
use std::string::String;
use std::path::PathBuf;
use std::path::Path;
extern crate rpassword;

static DIR: &'static str = "~/.bw.d/";

#[derive(Serialize, Deserialize)]
struct PwEntry {
    alias: String,
    id: String,
}

fn prompt_yesorno(msg: &str) -> bool {
    let mut ans = String::new();
    println!("{} [y/n]", msg);
    stdin()
        .read_line(&mut ans)
        .expect("Failed reading from stdin");
    match ans.to_ascii_lowercase().replace("\n", "").as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            println!("Please enter y or n");
            prompt_yesorno(msg)
        }
    }
}

fn usage() {
    println!("Usage: ...");
}

fn store_session(session: &str) {
    let fname = Path::new("/tmp/session");
    let mut file = File::create(fname).expect("Failed to create session");
    file.write_all(session.as_bytes());
}

fn load_session(session: &str) -> String{
    let fname: &Path = Path::new("/tmp/session");
    std::fs::read_to_string(fname).expect("failed to load session")
}

fn login(args: Vec<String>) {
    let pass = rpassword::prompt_password_stdout("Please enter your password (hidden):").unwrap();
    let out = Command::new("bw")
    .arg("unlock")
    .arg("--raw")
    .arg(pass)
    .output()
    .expect("Failed to set noisy terminal");

    if !out.status.success() {
        std::io::stdout().write_all(&out.stderr).unwrap();
        println!();
        return;
    }
    store_session(std::str::from_utf8(&out.stdout).unwrap());
    println!("Storing session key.. ");
    std::io::stdout().write_all(&out.stdout).unwrap();
    println!()
}

fn get_id(alias: &str, pws: Vec<PwEntry>) -> Result<String, ()> {
    for pw in pws {
        if pw.alias == alias {
            return Ok(pw.id);
        }
    }
    Err(())
}

fn get(pws: Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 3 {
        usage();
        return;
    }

    let alias: &str = &_args[2];
    let id: String = get_id(alias, pws)
        .expect(format!("Could not find id corresponding to '{}'", alias).as_str());
    let mut args: Vec<String> = vec![id];

    args.extend(_args);
    Command::new("bw get password")
        .args(args)
        .output()
        .expect("Failed getting pw args[1]");
}

fn rpw_cmd(pws: Vec<PwEntry>, args: Vec<String>) {
    if args.len() < 2 {
        usage();
        return;
    }

    match args[1].as_ref() {
        "login" => login(args),
        "get" => get(pws, args),
        _ => print!("Unknown command {} not implemented", args[1]),
    }
}

fn write_db(fname: &Path, entries: Vec<PwEntry>) -> Result<(), &'static str> {
    let json = serde_json::to_string(&entries).expect("Failed to serialize passwords");
    // This isn't a nice way to do it .. but wth!
    let mut db = File::create(fname).expect(&format!("Failed to create database {}", fname.display()));

    db.write_all(&json.as_bytes())
        .expect("Failed writing database");
    Ok(())
}

fn db_create_if_yes(fname: &Path) -> Result<bool, String> {
    println!("Could not find database {}", fname.display());
    if !prompt_yesorno(&format!("Would you like to create {} ?", fname.display())) {
        return Ok(false);
    }
    println!("Creating {}", fname.display());
    File::create(fname).expect(&format!("Failed to create database {}", fname.display()));
    return Ok(true);
}

fn read_db(fname: &Path) -> Result<Vec<PwEntry>, String> {
    match File::open(&fname) {
        Ok(f) => {
            let db: Vec<PwEntry>;
            println!("{}", fname.display());
            Ok(serde_json::from_reader::<File, Vec<PwEntry>>(f)
                .expect("Failed deserializing database"))
        }
        Err(_) => {
            db_create_if_yes(&fname)?;
            write_db(&fname, vec!())?;
            read_db(&fname)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let dir = std::env::home_dir().unwrap().join(".rpw.d");
    std::fs::create_dir_all(&dir);

    let path = dir.join("rusty.db");
    let pws: Vec<PwEntry> = read_db(&path).unwrap();

    println!("\n\n\n\n");
    println!("Rusty Cache starting up!...");
    for arg in &args {
        println!("\t{}", arg);
    }

    rpw_cmd(pws, args);
    Ok(())
}
