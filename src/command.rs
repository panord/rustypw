use std::collections::HashMap;
use std::str::FromStr;

pub struct Command {
    name: String,
    msg: String,
    missing: bool,
}

pub fn arg_map(args: &[String]) -> HashMap<String, String> {
    let mut am = HashMap::new();
    for (i, arg) in args.iter().enumerate() {
        if args.len() >= i + 2 {
            am.insert(args[i].clone(), args[i + 1].clone());
        }
    }
    return am;
}

impl Command {
    pub fn is_ok(&self) -> bool {
        return self.missing;
    }

    pub fn new(name: &str) -> Command {
        return Command {
            name: name.to_string(),
            msg: "".to_string(),
            missing: false,
        };
    }

    pub fn require<T: FromStr>(
        &mut self,
        key: &str,
        args: &HashMap<String, String>,
    ) -> Result<T, <T as std::str::FromStr>::Err> {
        match args.get(key) {
            None => {
                let add = format!("\tMissing required argument '{}'\n", key.to_string());
                self.msg = format!("{} {}", self.msg, add);
                self.missing = true;
                T::from_str("")
            }
            Some(strval) => T::from_str(strval),
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
