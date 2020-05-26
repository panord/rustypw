use serde::{Deserialize, Serialize};
use std::env;
use std::fs::*;
use std::io::prelude::*;
use std::io::stdin;
use std::process::Command;
use std::result::Result;
use std::string::String;
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
        .read_to_string(&mut ans)
        .expect("Failed reading from stdin");
    match ans.to_ascii_lowercase().as_str() {
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

fn login(args: Vec<String>) {
    println!("Loggin in...");
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

fn command(pws: Vec<PwEntry>, args: Vec<String>) {
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

fn write_db(fname: &str, entries: Vec<PwEntry>) -> Result<(), &'static str> {
    let json = serde_json::to_string(&entries).expect("Failed to serialize passwords");
    // This isn't a nice way to do it .. but wth!
    let mut db = File::create(fname).expect("Failed to create database");

    db.write_all(&json.as_bytes())
        .expect("Failed writing database");
    Ok(())
}

fn db_create_if_yes(fname: &str) -> Result<bool, String> {
    if !prompt_yesorno(&format!("Would you like to create {} ?", fname)) {
        return Ok(false);
    }

    match File::create(fname) {
        Ok(_) => Ok(true),
        Err(_) => Err(format!("Failed to create database {}", fname)),
    }
}

fn read_db(fname: String) -> Result<Vec<PwEntry>, String> {
    match File::open(&fname) {
        Ok(f) => {
            let db: Vec<PwEntry>;
            Ok(serde_json::from_reader::<File, Vec<PwEntry>>(f)
                .expect("Failed deserializing database"))
        }
        Err(_) => {
            db_create_if_yes(&fname)?;
            read_db(fname)
        }
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let pws: Vec<PwEntry> = read_db(format!("{}{}", DIR, "bw.db")).unwrap();

    println!("\n\n\n\n");
    println!("Rusty Cache starting up!...");
    for arg in &args {
        println!("\t{}", arg);
    }

    command(pws, args);
    println!("Rusty Cache exiting Exiting");
    Ok(())
}
