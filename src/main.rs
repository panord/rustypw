extern crate rpassword;

use clap::{value_t, ArgMatches};
use config::Config;
use rlib::*;
use rustyline::{error::ReadlineError, Editor};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::result::Result;
use std::string::String;
pub use vault::{LockedVault, UnlockedVault};

struct ProgramState {
    cancelp: Option<Child>,
    locked_vault: Option<LockedVault>,
    master_pw: Option<String>,
}

impl ProgramState {
    fn new() -> Self {
        ProgramState {
            cancelp: None,
            locked_vault: None,
            master_pw: None,
        }
    }
}

fn open(args: &ArgMatches, state: &mut ProgramState, config: &Config) {
    let lv = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let name = lv.name.clone();
    state.locked_vault = Some(lv);
    state.master_pw = Some(
        value_t!(args.value_of("password"), String)
            .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):")),
    );

    match state
        .locked_vault
        .as_ref()
        .unwrap()
        .unlock(state.master_pw.as_ref().unwrap())
    {
        Err(e) => {
            println!("{}", e);
            return;
        }
        Ok(v) => println!("Unlocking {}", v.name),
    };

    let app = cli::build();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(&format!("{}{}", &name, ">> "));
        match readline {
            Ok(line) => {
                let mut cmd = vec!["rpw"];
                if line.trim().is_empty() {
                    continue;
                }
                cmd.extend(line.split_whitespace());

                let matches = app.clone().get_matches_from_safe(cmd);
                match matches {
                    Ok(m) => dispatch(&m, state, &config),
                    Err(msg) => println!("{}", msg),
                };
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(msg) => {
                println!("{}, exiting", msg);
                break;
            }
        }
    }
}

fn new(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), String).unwrap();
    let pass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please choose vault password (prompt_hidden):"));
    let vfied = value_t!(args.value_of("verify"), String)
        .unwrap_or_else(|_| cli::password("Verify vault password (prompt_hidden):"));

    if pass != vfied {
        println!("Passwords do not match");
        return;
    }

    let lv = UnlockedVault::new(&vault).lock(&pass).unwrap();
    if lv.exists()
        && !cli::yesorno(&format!(
            "Vault '{}' already exists, would you like to overwrite it?",
            vault
        ))
    {
        println!("Aborting, not creating vault '{}'.", vault);
    }
    lv.save().unwrap();
    println!("New vault {} created", vault);
}

fn delete(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let pass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));
    let vfied = value_t!(args.value_of("verify"), String)
        .unwrap_or_else(|_| cli::password("Verify vault password (prompt_hidden):"));

    if pass != vfied {
        println!("Passwords do not match");
        return;
    }

    match &vault.delete() {
        Ok(_) => println!("Deleted vault {}", &vault.name),
        Err(msg) => cli::error(&msg),
    }
}

fn add(args: &ArgMatches, state: &mut ProgramState) {
    if state.locked_vault.is_none() {
        state.locked_vault = Some(value_t!(args.value_of("vault"), LockedVault).unwrap());
    }

    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let alias = value_t!(args.value_of("alias"), String).unwrap();
    let npass = value_t!(args.value_of("new-password"), String)
        .unwrap_or_else(|_| cli::password("Please enter new password (prompt_hidden):"));

    let mut uv = vault.unlock(&mpass).unwrap();
    uv.insert(alias, npass);
    uv.lock(&mpass).unwrap().save().unwrap();
}

fn export(args: &ArgMatches, state: &mut ProgramState) {
    let fpath = value_t!(args.value_of("file-path"), PathBuf).unwrap();
    if state.locked_vault.is_none() {
        state.locked_vault = Some(value_t!(args.value_of("vault"), LockedVault).unwrap());
    }
    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let uv = vault.unlock(&mpass).unwrap();
    match &uv.export(&fpath) {
        Ok(_) => println!("Exported vault {}", &fpath.display()),
        Err(msg) => println!("{}", &msg),
    }
}

fn dispatch(matches: &ArgMatches, state: &mut ProgramState, config: &Config) {
    match matches.subcommand() {
        ("open", Some(sargs)) => open(sargs, state, config),
        ("new", Some(sargs)) => new(sargs),
        ("delete", Some(args)) => delete(args),
        ("export", Some(args)) => export(args, state),
        ("import", Some(args)) => import(args, state),
        ("add", Some(sargs)) => add(sargs, state),
        ("get", Some(args)) => get(args, state, config),
        ("list", Some(args)) => list(args, state),
        ("clear", Some(args)) => clear(args),
        _ => {}
    }
}

fn import(args: &ArgMatches, state: &mut ProgramState) {
    let fpath = value_t!(args.value_of("file"), PathBuf).unwrap();

    if state.locked_vault.is_none() {
        state.locked_vault = Some(value_t!(args.value_of("vault"), LockedVault).unwrap());
    }
    let vault = state.locked_vault.as_ref().unwrap();

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let mut uv = vault.unlock(&mpass).unwrap();
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
            uv.lock(&mpass).unwrap().save().unwrap();
        }
        Err(e) => println!("{}", &e),
    };
}

fn list(args: &ArgMatches, state: &mut ProgramState) {
    if state.locked_vault.is_none() {
        state.locked_vault = Some(value_t!(args.value_of("vault"), LockedVault).unwrap());
    }

    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let uv = vault.unlock(&mpass).unwrap();
    let ids: Vec<&String> = uv.pws.iter().map(|p| p.0).collect();

    println!("Stored passwords");
    for id in ids {
        println!("\t{}", id);
    }
}

fn get(args: &ArgMatches, state: &mut ProgramState, config: &Config) {
    if state.locked_vault.is_none() {
        state.locked_vault = Some(value_t!(args.value_of("vault"), LockedVault).unwrap());
    }

    let vault = state.locked_vault.as_ref().unwrap();

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let sec = value_t!(args.value_of("sec"), u64).unwrap_or_else(|_| config.clear_copy_timeout);
    let id = value_t!(args.value_of("alias"), String).unwrap();
    let uv = vault.unlock(&mpass).unwrap();
    match uv.get(id.to_string()) {
        Ok(pass) => {
            cli::xclip::to_clipboard(&pass);
            println!("Clearing clipboard in {} seconds", sec);
            if state.cancelp.is_some() {
                state
                    .cancelp
                    .as_mut()
                    .unwrap()
                    .kill()
                    .expect("Failed to kill process");
            }
            state.cancelp = Some(do_clear(sec));
        }
        Err(msg) => println!("{}", msg),
    };
}

fn do_clear(sleep: u64) -> Child {
    Command::new("rpw")
        .arg("clear")
        .arg(sleep.to_string())
        .spawn()
        .expect("Failed getting pw")
}

fn clear(args: &ArgMatches) {
    let sec = value_t!(args.value_of("sec"), u64).unwrap();
    let dur = std::time::Duration::from_secs(sec);
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
}

fn main() {
    let mut state = ProgramState::new();
    let config: Result<Config, Config> = Config::load().or_else(|_| Ok(Config::new().save()));

    let app = cli::build();
    let matches = app.clone().get_matches_safe();
    match matches {
        Ok(m) => dispatch(&m, &mut state, &config.unwrap()),
        Err(msg) => println!("{}", msg),
    };
    std::fs::create_dir_all(&files::rpwd()).expect("Failed to create rpw dir");
}
