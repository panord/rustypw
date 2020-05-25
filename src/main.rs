use serde::{Deserialize, Serialize};
use std::env;
use std::fs::*;
use std::io::prelude::*;
use std::process::Command;
use std::result::Result;
use std::string::String;

static DIR: &'static str = "~/.bw.d/";

#[derive(Serialize, Deserialize)]
struct PwEntry {
    alias: String,
    id: String,
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
    let mut file = File::open(fname).expect(format!("Failed to open database {}", fname).as_str());
    file.write_all(&json.as_bytes()) .expect("Failed writing database");
    Ok(())
}

fn read_db(fname: String) -> Result<Vec<PwEntry>, &'static str> {
    let db: String = std::fs::read_to_string(fname).expect("Failed reading database");
    let entries: Vec<PwEntry> = serde_json::from_str(&db).expect("Failed deserializing database");
    Ok(entries)
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    println!("\n\n\n\n");
    println!("Rusty Cache starting up!...");
    let pw_db: String = format!("{}{}", DIR, "bw.db");
    for arg in &args {
        println!("\t{}", arg);
    }

    let pws: Vec<PwEntry> = read_db(pw_db).unwrap();
    command(pws, args);
    println!("Rusty Cache exiting Exiting");
    Ok(())
}
