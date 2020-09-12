use crate::cli;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

pub struct Command {
    name: String,
    msg: String,
    ok: bool,
}

pub fn arg_map(args: &[String]) -> HashMap<String, String> {
    let mut am = HashMap::new();
    for (i, _) in args.iter().enumerate() {
        if args.len() >= i + 2 {
            am.insert(args[i].clone(), args[i + 1].clone());
        }
    }
    return am;
}

#[derive(Debug)]
pub struct ArgParseError {
    arg: String,
    value: String,
}

impl Error for ArgParseError {}
impl fmt::Display for ArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed to parse argument '{}'\n from '{}'",
            self.arg, self.value
        )
    }
}

impl Command {
    pub fn is_ok(&self) -> bool {
        return self.ok;
    }

    pub fn new(name: &str) -> Command {
        return Command {
            name: name.to_string(),
            msg: "".to_string(),
            ok: true,
        };
    }
    fn do_require<T: FromStr>(
        &mut self,
        key: &str,
        args: &HashMap<String, String>,
    ) -> Result<T, ArgParseError> {
        match args.get(key) {
            None => Err(ArgParseError {
                arg: key.to_string(),
                value: "None".to_string(),
            }),
            Some(strval) => {
                let tres = T::from_str(strval);
                match tres {
                    Ok(t) => Ok(t),
                    Err(_err) => Err(ArgParseError {
                        arg: key.to_string(),
                        value: "None".to_string(),
                    }),
                }
            }
        }
    }

    pub fn require<T: FromStr>(
        &mut self,
        key: &str,
        args: &HashMap<String, String>,
    ) -> Result<T, ArgParseError> {
        match self.do_require::<T>(key, args) {
            Ok(arg) => Ok(arg),
            Err(res) => {
                let add = format!("\tMissing required argument '{}'\n", key.to_string());
                self.msg = format!("{} {}", self.msg, add);
                self.ok = false;
                Err(res)
            }
        }
    }

    pub fn default<T: FromStr>(
        &mut self,
        key: &str,
        args: &HashMap<String, String>,
        value: T,
    ) -> T {
        match self.require(key, args) {
            Ok(arg) => arg,
            Err(_) => value,
        }
    }

    pub fn hidden<T: FromStr>(
        &mut self,
        key: &str,
        args: &HashMap<String, String>,
        msg: &str,
    ) -> Result<T, ArgParseError> {
        match self.require::<T>(key, args) {
            Ok(arg) => Ok(arg),
            Err(_) => {
                let pval = cli::password(&format!("{}", msg));
                match T::from_str(&pval) {
                    Ok(t) => Ok(t),
                    Err(_err) => {
                        self.ok = false;
                        Err(ArgParseError {
                            arg: key.to_string(),
                            value: pval,
                        })
                    }
                }
            }
        }
    }

    pub fn usage(&self) -> String {
        return format!(
            "Error command '{}'\n {}",
            self.name.clone(),
            self.msg.clone()
        );
    }
}
