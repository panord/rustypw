use anyhow::{anyhow, Context, Result};
use cli::cli::*;
use config::Config;
use rlib::*;
// use rustyline::{error::ReadlineError, Editor};
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

// fn open(args: &OpenArgs, state: &mut ProgramState, config: &Config) -> Result<()> {
//     let lv = value_t!(args.value_of("vault"), LockedVault).context("Could not find vault")?;
//     let name = lv.name.clone();
//     state.locked_vault = Some(lv);
//     state.master_pw = Some(
//         value_t!(args.value_of("password"), String)
//             .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):")),
//     );

//     state
//         .locked_vault
//         .as_ref()
//         .unwrap()
//         .unlock(state.master_pw.as_ref().unwrap())?;
//     let mut rl = Editor::<()>::new();
//     loop {
//         let readline = rl.readline(&format!("{}{}", &name, "$ "));
//         match readline {
//             Ok(line) => {
//                 let mut cmd = vec!["rpw"];
//                 if line.trim().is_empty() {
//                     continue;
//                 }
//                 cmd.extend(line.split_whitespace());

//                 let matches = app.clone().get_matches_from_safe(cmd);
//                 match matches {
//                     Ok(m) => dispatch(&m, state, config),
//                     Err(msg) => println!("{}", msg),
//                 };
//             }
//             Err(ReadlineError::Interrupted) => {
//                 continue;
//             }
//             Err(msg) => {
//                 println!("{}, exiting", msg);
//                 break;
//             }
//         }
//     }
//     Ok(())
// }

fn new(args: &NewArgs) -> Result<()> {
    let vault = &args.vault;
    let pass = args
        .password
        .clone()
        .unwrap_or_else(|| cli::password("Please choose vault password (hidden):"));

    let vfied = args
        .verify
        .clone()
        .unwrap_or_else(|| cli::password("Verify vault password (hidden):"));

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

fn delete(args: &DelArgs) -> Result<()> {
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;

    let pass = args
        .password
        .clone()
        .unwrap_or_else(|| cli::password("Please enter vault password (hidden):"));

    let vfied = args
        .verify
        .clone()
        .unwrap_or_else(|| cli::password("Verify vault password (hidden):"));

    if pass != vfied {
        return Err(anyhow!("Passwords do not match"));
    }

    vault.delete().context("Failed deleting vault.")?;
    println!("Deleted vault {}", &vault.name);
    Ok(())
}

fn add(args: &AddArgs, state: &mut ProgramState) -> Result<()> {
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| args.password.clone())
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let alias = &args.alias;
    let npass = args
        .new_password
        .clone()
        .unwrap_or_else(|| cli::password("Please enter new password (hidden):"));

    let mut uv = vault.unlock(&mpass)?;
    uv.insert(alias.clone(), npass);
    uv.lock(&mpass)?.save()?;
    Ok(())
}

fn export(args: &ExportArgs, state: &mut ProgramState) -> Result<()> {
    let fpath = PathBuf::from(&args.file_path);
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;
    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| args.password.clone())
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let uv = vault.unlock(&mpass)?;
    uv.export(&fpath)?;
    println!("Exported vault {}", &fpath.display());
    Ok(())
}

fn import(args: &ImportArgs, state: &mut ProgramState) -> Result<()> {
    let fpath = PathBuf::from(&args.file_path);
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| args.password.clone())
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

fn list(args: &ListArgs, state: &mut ProgramState) -> Result<()> {
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;

    let mpass = state
        .master_pw
        .clone()
        .ok_or_else(|| args.password.clone())
        .unwrap_or_else(|_| cli::password("Please enter vault password (hidden):"));

    let uv = vault.unlock(&mpass).unwrap();
    let ids: Vec<&String> = uv.pws.iter().map(|p| p.0).collect();

    println!("Stored passwords");
    for id in ids {
        println!("\t{}", id);
    }
    Ok(())
}

fn do_get(
    id: &str,
    mpass: &str,
    sec: u64,
    cp: &mut Option<Child>,
    vault: &LockedVault,
) -> Result<Option<Child>, anyhow::Error> {
    let uv = vault.unlock(&mpass).context("Failed to unlock vault")?;
    let pass = uv.get(id).context("Failed to get password")?;
    cli::xclip::to_clipboard(pass);
    if let Some(cp) = cp {
        ignore!(cp.kill());
    }
    println!("Clearing clipboard in {} seconds", sec);

    Ok(Some(do_clear(sec)))
}

fn get(args: &GetArgs, state: &mut ProgramState, config: &Config) -> Result<()> {
    let id = &args.alias;
    let vault: LockedVault = args
        .vault
        .parse::<LockedVault>()
        .context("Could not find vault")?;

    let mpass = match state.master_pw.clone() {
        None => cli::password("Please enter vault password (hidden):"),
        Some(p) => {
            state.master_pw = Some(p.clone());
            p.clone()
        }
    };
    let sec = config.clear_copy_timeout;

    if let Ok(child) = do_get(&id, &mpass, sec, &mut state.cancelp, &vault) {
        state.cancelp = child;
    }
    Ok(())
}

fn do_clear(sleep: u64) -> Child {
    Command::new("rpw")
        .arg("clear")
        .arg(sleep.to_string())
        .spawn()
        .expect("Failed getting pw")
}

fn clear(args: &ClearArgs) -> Result<()> {
    let sec = args.sec;
    let dur = std::time::Duration::from_secs(sec);
    std::thread::sleep(dur);
    cli::xclip::to_clipboard("cleared");
    Ok(())
}

fn main() {
    let mut state = ProgramState::new();
    let config: Config = Config::load()
        .or_else(|_| Ok::<Config, anyhow::Error>(Config::new()))
        .expect("Failed to load configuration");

    let app = cli::cli::RpwCli::cli();

    match app.command {
        cli::cli::Command::Open(args) => Err(anyhow!("Not Implemented")),
        cli::cli::Command::Clear(args) => clear(&args),
        cli::cli::Command::Get(args) => get(&args, &mut state, &config),
        cli::cli::Command::List(args) => list(&args, &mut state),
        cli::cli::Command::Import(args) => import(&args, &mut state),
        cli::cli::Command::Export(args) => export(&args, &mut state),
        cli::cli::Command::New(args) => new(&args),
        cli::cli::Command::Delete(args) => delete(&args),
        cli::cli::Command::Add(args) => add(&args, &mut state),
    }
    .unwrap();
}
