use std::env;
use std::io::prelude::*;
use std::process::Command;
use std::result::Result;
use std::string::String;
extern crate rpassword;

mod jconf;
mod session;
mod store;
mod cli;

extern crate dirs;

use jconf::BwID;
use jconf::PwEntry;

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

fn unlock() {
    let pass = rpassword::prompt_password_stdout("Please enter your password (hidden):").unwrap();
    let session = store::unlock(&pass);
    match session {
        Ok(s) => {
            println!("Storing session...\n{}", s);
            session::store_session(&s);
        },
        Err(s) => cli::error(&s),
    }
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
        usage("get");
        return;
    }

    let alias: &str = &_args[2];
    let id: &str = get_id(alias, pws)
        .expect(format!("Could not find id corresponding to '{}'", alias).as_str());

    let session: String = session::load_session();

    let mut clip = Command::new("xclip")
        .stdin(std::process::Stdio::piped())
        .arg("-selection")
        .arg("clipboard")
        .spawn()
        .expect("Failed getting pw");

    let pw = store::get(&id, &session);
    match pw {
        Ok(pw) => {
            clip.stdin
                .as_mut()
                .unwrap()
                .write_all(pw.as_bytes())
                .expect("Failed to open stdin");
        },
        Err(msg) => cli::error(&msg),
    }
}

fn alias(pws: &mut Vec<PwEntry>, _args: Vec<String>) {
    if _args.len() != 4 {
        usage("alias");
        return;
    }

    let name: &str = &_args[2];
    let alias: &str = &_args[3];
    let session: String = session::load_session();

    if get_id(alias, pws).is_ok() {
        println!("Alias already known");
        return;
    }
    match store::get_item_id(name, &session) {
        Ok(id) => {
            println!("{}={}", alias, id);
            serde_json::from_str::<BwID>(&id).expect("fail").id;
            let entry = PwEntry {
                id: id,
                alias: alias.to_string(),
            };
            pws.push(entry);
        },
        Err(msg) => cli::error(&msg),
    }

}

fn rpw_cmd(pws: &mut Vec<PwEntry>, args: Vec<String>) {
    if args.len() < 2 {
        usage("");
        return;
    }

    match args[1].as_ref() {
        "unlock" => unlock(),
        "get" => get(&pws, args),
        "alias" => alias(pws, args),
        _ => println!("Unknown command {} not implemented", args[1]),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let dir = dirs::home_dir().unwrap().join(".rpw.d");

    std::fs::create_dir_all(&dir).expect("Failed to create rpw dir");

    let path = dir.join("rusty.db");

    jconf::init(&path).expect("Failed to create rpw config");
    let mut pws: Vec<PwEntry> = jconf::read(&path).unwrap();

    rpw_cmd(&mut pws, args);
    jconf::write(&path, pws).unwrap();
    Ok(())
}
