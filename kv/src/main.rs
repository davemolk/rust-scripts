use anyhow::{anyhow, Context, Result};
use std::io::{self, Write};
use std::fs::OpenOptions;
use std::fs;

const USAGE: &str = r"Usage:
kv list — list all key-value pairs in db
kv get <key> — get the value for given key
kv set <key> <value> — set a value for a given key
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = "database.txt".to_string();
    let file_database = FileDatabase::new(path);
    let runner = Runner::new(file_database);
    if let Err(e) = runner.run(&mut io::stdout(), args) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}

trait Storage {
    fn set(&self, key: &str, value: &str) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<String>>;
}

struct Runner<T: Storage> {
    database: T,
}

impl<T: Storage> Runner<T> {
    fn new(database: T) -> Self {
        Runner { database }
    }
    fn run(&self, output: &mut dyn Write, args: Vec<String>) -> Result<()> {
        if args.len() < 3 {
            eprintln!("{USAGE}");
            return Err(anyhow!("not enough args to run"))
        }
        match args[1].as_str() {
            "set" => {
                if args.len() < 4 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set"))
                }
                self.database.set(&args[2], &args[3])?;
            },
            "get" => {
                if let Some(v) = self.database.get(&args[2])? {
                    writeln!(output, "{}", v)?;
                } else {
                    return Err(anyhow!("not found"));
                }
            },
            _ => {
                eprintln!("{USAGE}");
                return Err(anyhow!("command not recognized"))
            }
        }
        Ok(())
    }
}

struct FileDatabase {
    file: String,
}

impl FileDatabase {
    fn new(path: String) -> Self {
        FileDatabase { file: path }
    }
}

impl Storage for FileDatabase {
    fn set(&self, key: &str, value: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)
            .context("failed to open file")?;
        writeln!(&mut file, "{}:{}", key, value)
            .context("failed to write to file")?;
        Ok(())
    }
    fn get(&self, key: &str) -> Result<Option<String>> {
        let contents = fs::read_to_string(&self.file)
            .context("failed to read file")?;
        let mut last = String::new();
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(":").collect();
            if parts.len() < 2 {
                continue
            }
            if parts[0] == key {
                last = parts[1].to_string();
            }
        }
        if last.is_empty() {
            return Ok(None);
        }
        Ok(Some(last))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockDatabase {
        set_err: Option<String>,
        get_err: Option<String>,
        get_value: Option<String>,
    }
    impl MockDatabase {
        fn new(set_err: Option<String>, get_err: Option<String>, get_value: Option<String>) -> Self {
            MockDatabase {
                set_err,
                get_err,
                get_value,
            }
        }
    }
    impl Storage for MockDatabase {
        fn set(&self, _key: &str, _value: &str) -> Result<()> {
            if let Some(err) = &self.set_err {
                let return_err = String::from(err);
                return Err(anyhow!(return_err));
            }
            Ok(())
        }
    
        fn get(&self, _key: &str) -> Result<Option<String>> {
            if let Some(err) = &self.get_err {
                let return_err = String::from(err);
                return Err(anyhow!(return_err));
            }
            Ok(self.get_value.clone())
        }
    }
    #[test]
    fn test_runner_args_err() {
        let runner = Runner::new(MockDatabase::new(None, None, None));
        let args = vec![];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_err());
    }
    #[test]
    fn test_runner_usage_err() {
        let runner = Runner::new(MockDatabase::new(None, None, None));
        let args = vec!["./kv".to_string(), "help".to_string(), "123".to_string()];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_err());
    }
    #[test]
    fn test_runner_set_missing_arg_err() {
        let runner = Runner::new(MockDatabase::new(None, None, None));
        let args = vec!["./kv".to_string(), "set".to_string(), "123".to_string()];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_err());
    }
    #[test]
    fn test_runner_returns_err_on_set() {
        let set_err = String::from("set err");
        let runner = Runner::new(MockDatabase::new(Some(set_err), None, None));
        let args = vec!["./kv".to_string(), "set".to_string(), "bob".to_string(), "123".to_string()];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_err());
        let want = anyhow::Error::msg("set err");
        let args = vec!["./kv".to_string(), "set".to_string(), "bob".to_string(), "123".to_string()];
        let got = runner.run(&mut output, args).unwrap_err();
        assert_eq!(want.to_string(), got.to_string());
    }
    #[test]
    fn test_runner_returns_err_on_get() {
        let get_err = String::from("get err");
        let get_value = String::from("get value");
        let runner = Runner::new(MockDatabase::new(None, Some(get_err), Some(get_value)));
        let args = vec!["./kv".to_string(), "get".to_string(), "bob".to_string(), "123".to_string()];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_err());
        let want = anyhow::Error::msg("get err");
        let args = vec!["./kv".to_string(), "get".to_string(), "bob".to_string(), "123".to_string()];
        let got = runner.run(&mut output, args).unwrap_err();
        assert_eq!(want.to_string(), got.to_string());
    }
    #[test]
    fn test_runner_get_returns_expected_value() {
        let runner = Runner::new(MockDatabase::new(None, None, Some("get value".to_string())));
        let args = vec!["./kv".to_string(), "get".to_string(), "bob".to_string()];
        let mut output = Vec::<u8>::new();
        assert!(runner.run(&mut output, args).is_ok());
        let got = String::from_utf8_lossy(&output);
        assert_eq!(got, "get value\n".to_string());
    }
}
