extern crate rpassword;
mod cli;
mod command;
mod files;
mod store;

use command::Command;
use rustyline::Editor;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Child;
use std::result::Result;
use std::string::String;
use store::{LockedVault, UnlockedVault};

struct ProgramState {
    cancelp: Option<Child>,
}

impl ProgramState {
    fn new() -> Self {
        ProgramState { cancelp: None }
    }
}

fn open(args: HashMap<String, String>, state: &mut ProgramState) {
    let mut command = Command::new("open");
    let vault = command.require::<LockedVault>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    // Is it better to store this or to expose the full db? Probably neither.
    // Perhaps we can store in intel enclave or something?
    let vault = vault.unwrap();
    let pass = cli::password("Please enter vault password (hidden):");
    if vault.unlock(&pass).is_err() {
        println!("You entered an incorrect password");
        return;
    }

    let name = vault.name;

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(&format!("{}{}", name, ">> "));
        if readline.is_ok() {
            let line = readline.unwrap();
            let args: Vec<String> = line.split_whitespace().map(String::from).collect();
            let mut hargs = command::arg_map(&args);
            if args.len() == 0 {
                continue;
            }
            hargs.insert("rpw".to_string(), args[0].clone());
            hargs.insert("vault".to_string(), name.clone());
            hargs.insert("--password".to_string(), pass.clone());
            run_command(hargs, state);
        } else {
            println!("{}", readline.unwrap_err());
        }
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
    if pass != vfied {
        println!("Passwords do not match");
        return;
    }

    if files::exists(&vault)
        && !cli::yesorno(&format!(
            "Vault '{}' already exists, would you like to overwrite it?",
            vault
        ))
    {
        return;
    }

    let uv = UnlockedVault::new(&vault);
    uv.lock(&pass).save();
    println!("{}", format!("New vault {} created", vault))
}

fn add(args: HashMap<String, String>) {
    let mut command = Command::new("add");
    let vres = command.require::<LockedVault>("vault", &args);
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

    let npass = nres.unwrap();
    let mpass = mres.unwrap();
    let alias = ares.unwrap();
    let mut uv = vres.unwrap().unlock(&mpass).unwrap();
    uv.insert(alias, npass);
    uv.lock(&mpass).save();
}

fn export(args: HashMap<String, String>) {
    let mut command = Command::new("export");
    let vres = command.require::<LockedVault>("vault", &args);
    let fres = command.require::<PathBuf>("file", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let mres = command.hidden::<String>("--password", &args, "Enter vault password (hidden):");
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let mpass = mres.unwrap();
    let fpath = fres.unwrap();
    let uv = vres.unwrap().unlock(&mpass).unwrap();
    match &uv.export(&fpath) {
        Ok(_) => println!("Exported vault {}", &fpath.display()),
        Err(msg) => cli::error(&msg),
    }
}

fn import(args: HashMap<String, String>) {
    let mut command = Command::new("import");
    let vres = command.require::<LockedVault>("vault", &args);
    let fres = command.require::<PathBuf>("file", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let mres = command.hidden::<String>("--password", &args, "Enter vault password (hidden):");
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let mpass = mres.unwrap();
    let fpath = fres.unwrap();
    let mut uv = vres.unwrap().unlock(&mpass).unwrap();
    let dupres = &uv.import(&fpath);
    match dupres {
        Ok(dup) => {
            println!("Imported {} into vault", &fpath.display());
            dup.iter().for_each(|p| {
                if cli::yesorno(&format!(
                    "Would you like to overwrite duplicate '{}'?",
                    p.id
                )) {
                    uv.insert(p.id.clone(), p.pw.clone());
                }
            });
            uv.lock(&mpass).save();
        }
        Err(msg) => cli::error(&msg),
    };
}

fn delete(args: HashMap<String, String>) {
    let mut command = Command::new("delete");
    let vres = command.require::<LockedVault>("vault", &args);
    if !command.is_ok() {
        println!("{}", command.usage());
        return;
    }

    let vault = vres.unwrap();
    match &vault.delete() {
        Ok(_) => println!("Deleted vault {}", &vault.name),
        Err(msg) => cli::error(&msg),
    }
}

fn list(args: HashMap<String, String>) {
    let mut command = Command::new("get");
    let vres = command.require::<LockedVault>("vault", &args);
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

    let mpass = mres.unwrap();
    let uv = vres.unwrap().unlock(&mpass).unwrap();
    let ids: Vec<&String> = uv.pws.iter().map(|p| p.0).collect();

    println!("Stored passwords");
    for id in ids {
        println!("\t{}", id);
    }
}

fn get(args: HashMap<String, String>, state: &mut ProgramState) {
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
    let uv = vres.unwrap().unlock(&mpass).unwrap();
    match uv.get(id.to_string()) {
        Ok(pass) => {
            cli::xclip::to_clipboard(&pass);
            println!("Clearing clipboard in {} seconds", sec);
            if state.cancelp.is_some() {
                state.cancelp.as_mut().unwrap().kill().expect("Failed to kill process");
            }
            state.cancelp = Some(cli::xclip::clear(sec));
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

fn run_command(args: HashMap<String, String>, state: &mut ProgramState) {
    match args.get("rpw") {
        Some(command) => match command.as_ref() {
            "open" => open(args, state),
            "new" => new(args),
            "export" => export(args),
            "import" => import(args),
            "add" => add(args),
            "get" => get(args, state),
            "list" => list(args),
            "clear" => clear(args),
            "delete" => delete(args),
            "help" => println!("open|list|new|get|export|import|add|clear|delete"),
            _ => println!("Unknown command or context {} not implemented", command),
        },
        None => println!("open|list|new|get|export|import|add|clear|delete"),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    let mut state = ProgramState::new();
    std::fs::create_dir_all(&files::rpwd()).expect("Failed to create rpw dir");
    run_command(command::arg_map(&args), &mut state);
    Ok(())
}
