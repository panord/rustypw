use std::env;
use std::io::prelude::*;
use std::process::Command;
use std::result::Result;
use std::string::String;
extern crate rpassword;

mod jconf;
mod session;

use jconf::BwID;
use jconf::PwEntry;

fn usage() {
    println!("Usage: ...");
}

fn unlock() {
    let pass = rpassword::prompt_password_stdout("Please enter your password (hidden):").unwrap();
    let out = Command::new("bw")
        .arg("unlock")
        .arg("--raw")
        .arg(pass)
        .output()
        .expect("Failed to set noisy terminal");

    if !out.status.success() {
        std::io::stdout().write_all(&out.stderr).unwrap();
        std::io::stdout().write_all(&out.stdout).unwrap();
        return;
    }

    session::store_session(std::str::from_utf8(&out.stdout).unwrap());
    println!("Storing session key.. ");
    std::io::stdout().write_all(&out.stdout).unwrap();
    println!()
}

fn get_id<'a>(alias: &str, pws: &'a Vec<PwEntry>) -> Result<&'a str, ()> {
    for pw in pws {
        if pw.alias == alias {
            return Ok(&pw.id);
        }
    }
    Err(())
}

fn get(pws: &Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 3 {
        usage();
        return;
    }

    let alias: &str = &_args[2];
    let id: &str = get_id(alias, pws)
        .expect(format!("Could not find id corresponding to '{}'", alias).as_str());

    let session: String = session::load_session();
    let out = Command::new("bw")
        .arg("get")
        .arg("password")
        .arg(id)
        .arg("--session")
        .arg(session)
        .output()
        .expect("Failed getting pw");

    let mut clip = Command::new("xclip")
        .stdin(std::process::Stdio::piped())
        .arg("-selection")
        .arg("clipboard")
        .spawn()
        .expect("Failed getting pw");

    clip.stdin
        .as_mut()
        .unwrap()
        .write_all(&out.stdout)
        .expect("Failed to open stdin");
}

fn alias(pws: &mut Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 4 {
        usage();
        return;
    }

    let name: &str = &_args[2];
    let alias: &str = &_args[3];

    if get_id(alias, pws).is_ok() {
        print!("Alias already known");
        return;
    }

    let session: String = session::load_session();
    let json = Command::new("bw")
        .arg("get")
        .arg("item")
        .arg(name)
        .arg("--session")
        .arg(session)
        .arg("--pretty")
        .output()
        .expect("Failed aliasing");

    let id = serde_json::from_slice::<BwID>(&json.stdout)
        .expect("Failed getting id")
        .id;
    print!("{}={}\n", alias, id);
    let entry = PwEntry {
        id: id,
        alias: alias.to_string(),
    };
    pws.push(entry);
}

fn rpw_cmd(pws: &mut Vec<PwEntry>, args: Vec<String>) {
    if args.len() < 2 {
        usage();
        return;
    }

    match args[1].as_ref() {
        "unlock" => unlock(),
        "get" => get(&pws, args),
        "alias" => alias(pws, args),
        _ => print!("Unknown command {} not implemented", args[1]),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let dir = std::env::home_dir().unwrap().join(".rpw.d");
    std::fs::create_dir_all(&dir).expect("Failed to create rpw dir");

    let path = dir.join("rusty.db");

    jconf::init(&path).expect("Failed to create rpw config");
    let mut pws: Vec<PwEntry> = jconf::read(&path).unwrap();

    println!("\n\n\n\n");
    println!("Rusty Cache starting up!...");
    for arg in &args {
        println!("\t{}", arg);
    }

    rpw_cmd(&mut pws, args);
    jconf::write(&path, pws).unwrap();
    Ok(())
}
