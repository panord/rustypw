use anyhow::{anyhow, Context, Result};
use clap::{value_t, ArgMatches};
use config::Config;
use rlib::*;
use rustyline::{error::ReadlineError, Editor};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::string::String;
use vault::{LockedVault, UnlockedVault};

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

macro_rules! ignore {
    ($x:expr) => {
        let _ = $x;
    };
}

fn open(args: &ArgMatches, state: &mut ProgramState, config: &Config) -> Result<()> {
    let lv = value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?;
    let name = lv.name.clone();
    state.locked_vault = Some(lv);
    state.master_pw = Some(
        value_t!(args.value_of("password"), String)
            .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):")),
    );

    state
        .locked_vault
        .as_ref()
        .unwrap()
        .unlock(state.master_pw.as_ref().unwrap())?;
    let app = cli::build();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline(&format!("{}{}", &name, "$ "));
        match readline {
            Ok(line) => {
                let mut cmd = vec!["rpw"];
                if line.trim().is_empty() {
                    continue;
                }
                cmd.extend(line.split_whitespace());

                let matches = app.clone().get_matches_from_safe(cmd);
                match matches {
                    Ok(m) => dispatch(&m, state, config),
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
    Ok(())
}

fn new(args: &ArgMatches) -> Result<()> {
    let vault = value_t!(args.value_of("vault"), String).unwrap();
    let pass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please choose vault password (hidden):"));
    let vfied = value_t!(args.value_of("verify"), String)
        .unwrap_or_else(|_| cli::password("Verify vault password (hidden):"));

    if pass != vfied {
        return Err(anyhow!("Passwords do not match"));
    }

    let lv = UnlockedVault::new(&vault).lock(&pass)?;
    if lv.exists()
        && !cli::yesorno(&format!(
            "Vault '{}' already exists, would you like to overwrite it?",
            vault
        ))
    {
        println!("Aborting, not creating vault '{}'.", vault);
    }
    lv.save()?;
    println!("New vault {} created", vault);
    Ok(())
}

fn delete(args: &ArgMatches) -> Result<()> {
    let vault = value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?;
    let pass = value_t!(args.value_of("password"), String)
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));
    let vfied = value_t!(args.value_of("verify"), String)
        .unwrap_or_else(|_| cli::password("Verify vault password (hidden):"));

    if pass != vfied {
        return Err(anyhow!("Passwords do not match"));
    }

    vault.delete().context("Failed deleting vault.")?;
    println!("Deleted vault {}", &vault.name);
    Ok(())
}

fn add(args: &ArgMatches, state: &mut ProgramState) -> Result<()> {
    if state.locked_vault.is_none() {
        state.locked_vault =
            Some(value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?);
    }

    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let alias = value_t!(args.value_of("alias"), String).unwrap();
    let npass = value_t!(args.value_of("new-password"), String)
        .unwrap_or_else(|_| cli::password("Please enter new password (hidden):"));

    let mut uv = vault.unlock(&mpass)?;
    uv.insert(alias, npass);
    uv.lock(&mpass)?.save()?;
    Ok(())
}

fn export(args: &ArgMatches, state: &mut ProgramState) -> Result<()> {
    let fpath = value_t!(args.value_of("file-path"), PathBuf).unwrap();
    if state.locked_vault.is_none() {
        state.locked_vault =
            Some(value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?);
    }
    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let uv = vault.unlock(&mpass)?;
    uv.export(&fpath)?;
    println!("Exported vault {}", &fpath.display());
    Ok(())
}

fn import(args: &ArgMatches, state: &mut ProgramState) -> Result<()> {
    let fpath = value_t!(args.value_of("file"), PathBuf).unwrap();

    if state.locked_vault.is_none() {
        state.locked_vault =
            Some(value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?);
    }
    let vault = state.locked_vault.as_ref().unwrap();

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let mut uv = vault.unlock(&mpass).unwrap();
    let dup = &uv.import(&fpath)?;
    println!("Imported {} into vault", &fpath.display());
    dup.iter().for_each(|p| {
        if cli::yesorno(&format!(
            "Would you like to overwrite duplicate '{}'?",
            p.id
        )) {
            uv.insert(p.id.clone(), p.pw.clone());
        }
    });
    uv.lock(&mpass)?.save()?;
    Ok(())
}

fn list(args: &ArgMatches, state: &mut ProgramState) -> Result<()> {
    if state.locked_vault.is_none() {
        state.locked_vault =
            Some(value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?);
    }

    let vault = state.locked_vault.as_ref().unwrap();
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let uv = vault.unlock(&mpass).unwrap();
    let ids: Vec<&String> = uv.pws.iter().map(|p| p.0).collect();

    println!("Stored passwords");
    for id in ids {
        println!("\t{}", id);
    }
    Ok(())
}

fn get(args: &ArgMatches, state: &mut ProgramState, config: &Config) -> Result<()> {
    if state.locked_vault.is_none() {
        state.locked_vault =
            Some(value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?);
    }

    let vault = state.locked_vault.as_ref().unwrap();

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| value_t!(args.value_of("password"), String))
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let sec = value_t!(args.value_of("sec"), u64).unwrap_or_else(|_| config.clear_copy_timeout);
    let id = value_t!(args.value_of("alias"), String).unwrap();
    let uv = vault.unlock(&mpass)?;
    let pass = uv.get(id).context("Failed to get password")?;
    cli::xclip::to_clipboard(pass);
    if let Some(cp) = state.cancelp.as_mut() {
        ignore!(cp.kill());
    }
    println!("Clearing clipboard in {} seconds", sec);
    state.cancelp = Some(do_clear(sec));
    Ok(())
}

fn do_clear(sleep: u64) -> Child {
    Command::new("rpw")
        .arg("clear")
        .arg(sleep.to_string())
        .spawn()
        .expect("Failed getting pw")
}

fn clear(args: &ArgMatches) -> Result<()> {
    let sec = value_t!(args.value_of("sec"), u64).unwrap();
    let dur = std::time::Duration::from_secs(sec);
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
    Ok(())
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
        _ => Err(anyhow!("Unrecognized command")),
    }
    .unwrap_or_else(|e| {
        println!("{}", e);
    });
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
}
