use std::str::FromStr;

pub struct Command {
    name: String,
    msg: String,
    missing: bool,
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
        args: &[String],
    ) -> Result<T, <T as std::str::FromStr>::Err> {
        let add = format!("\tMissing required argument '{}'\n", key.to_string());
        for (i, arg) in args.iter().enumerate() {
            if args.len() < i + 2 {
                break;
            }
            if key == arg {
                return T::from_str(&args[i + 1]);
            }
        }
        self.msg = format!("{} {}", self.msg, add);
        self.missing = true;
        T::from_str("")
    }

    pub fn usage(&self) -> String {
        return format!(
            "Error command '{}'\n {}",
            self.name.clone(),
            self.msg.clone()
        );
    }
}
