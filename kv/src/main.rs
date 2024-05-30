use anyhow::{Result, anyhow};
use std::io::{self, Write};

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
    fn get(&self, key: &str) -> Result<String>;
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
            return Err(anyhow!("not enough args"))
        }
        match args[1].as_str() {
            "set" => {
                if args.len() < 4 {
                    return Err(anyhow!("make a usage thing"))
                }
                self.database.set(&args[2], &args[3])?;
                return Ok(())
            },
            "get" => {
                let v = self.database.get(&args[2])?;
                writeln!(output, "{}", v)?;
                return Ok(())
            },
            _ => return Err(anyhow!("make a usage thing")),
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
        println!("you called set");
        Ok(())
    }
    fn get(&self, key: &str) -> Result<String> {
        println!("you called get");
        Ok("foo".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}