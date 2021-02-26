extern crate rpassword;
mod cli;
mod config;
mod files;
mod vault;

use clap::{value_t, ArgMatches};
use config::Config;
use rustyline::Editor;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::result::Result;
use std::string::String;
use vault::{LockedVault, UnlockedVault};

struct ProgramState {
    cancelp: Option<Child>,
}

impl ProgramState {
    fn new() -> Self {
        ProgramState { cancelp: None }
    }
}

fn open(args: &ArgMatches, state: &mut ProgramState, config: &Config) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let pass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));

    let name = vault.name;
    let app = cli::build();
    let largs = vec!["--vault", &name, "--password", &pass];
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(&format!("{}{}", &name, ">> "));
        if readline.is_ok() {
            let mut cmd = vec!["rpw"];
            let line = readline.unwrap();
            if line.trim().is_empty() {
                continue;
            }
            cmd.extend(line.split_whitespace());
            cmd.extend(&largs);

            let matches = app.clone().get_matches_from_safe(cmd);
            match matches {
                Ok(m) => dispatch(&m, state, &config),
                Err(msg) => println!("{}", msg),
            };
        } else {
            println!("{}", readline.unwrap_err());
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

    let lv = UnlockedVault::new(&vault).lock(&pass);
    if lv.exists()
        && !cli::yesorno(&format!(
            "Vault '{}' already exists, would you like to overwrite it?",
            vault
        ))
    {
        println!("Aborting, not creating vault '{}'.", vault);
    }
    lv.save();
    println!("New vault {} created", vault);
}

fn add(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let alias = value_t!(args.value_of("alias"), String).unwrap();
    let mpass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));
    let npass = value_t!(args.value_of("new-password"), String)
        .unwrap_or_else(|_| cli::password("Please enter new password (prompt_hidden):"));

    let mut uv = vault.unlock(&mpass).unwrap();
    uv.insert(alias, npass);
    uv.lock(&mpass).save();
}

fn export(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let fpath = value_t!(args.value_of("file-path"), PathBuf).unwrap();
    let mpass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));
    let uv = vault.unlock(&mpass).unwrap();
    match &uv.export(&fpath) {
        Ok(_) => println!("Exported vault {}", &fpath.display()),
        Err(msg) => cli::error(&msg),
    }
}

fn dispatch(matches: &ArgMatches, state: &mut ProgramState, config: &Config) {
    match matches.subcommand() {
        ("open", Some(sargs)) => open(sargs, state, config),
        ("new", Some(sargs)) => new(sargs),
        ("export", Some(args)) => export(args),
        ("import", Some(args)) => import(args),
        ("add", Some(sargs)) => add(sargs),
        ("get", Some(args)) => get(args, state, config),
        ("list", Some(args)) => list(args),
        ("clear", Some(args)) => clear(args),
        ("delete", Some(args)) => delete(args),
        _ => {}
    }
}

fn import(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let fpath = value_t!(args.value_of("file"), PathBuf).unwrap();
    let mpass = value_t!(args.value_of("password"), String)
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
            uv.lock(&mpass).save();
        }
        Err(msg) => cli::error(&msg),
    };
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

fn list(args: &ArgMatches) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let mpass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (prompt_hidden):"));
    let uv = vault.unlock(&mpass).unwrap();
    let ids: Vec<&String> = uv.pws.iter().map(|p| p.0).collect();

    println!("Stored passwords");
    for id in ids {
        println!("\t{}", id);
    }
}

fn get(args: &ArgMatches, state: &mut ProgramState, config: &Config) {
    let vault = value_t!(args.value_of("vault"), LockedVault).unwrap();
    let mpass = value_t!(args.value_of("password"), String)
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
        Err(msg) => cli::error(&msg),
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
