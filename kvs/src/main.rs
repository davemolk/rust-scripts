use std::path::Path;
use anyhow::{Result, anyhow};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use std::io::{self, BufReader, ErrorKind, Write};

const USAGE: &str = r"Usage:
kvs list — list all key-value pairs in db
kvs get <key> — get the value for given key
kvs set <key> <value> — set a value for a given key
kvs delete <key> — deletes a key and its value
";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file_database = match FileDatabase::connect(String::from("kvs.db")) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        },
        Ok(f) => f,
    };
    let mut runner = Runner::new(file_database);
    if let Err(e) = runner.run(&mut io::stdout(), args) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}

struct Runner<T: Storage> {
    database: T,
}

impl<T: Storage> Runner<T> {
    fn new(database: T) -> Self {
        Runner { database }
    }
    fn run(&mut self, output: &mut dyn Write, args: Vec<String>) -> Result<()> {
        if args.len() < 2 {
            eprintln!("{USAGE}");
            return Err(anyhow!("not enough args to run"))
        }
        match args[1].as_str() {
            "list" => {
                let m = self.database.list();
                for (k, v) in m {
                    println!("{k}: {v}");
                }
            }
            "set" => {
                if args.len() < 4 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set"))
                }
                self.database.set(&args[2], &args[3])?;
            },
            "get" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set"))
                }
                match self.database.get(&args[2]) {
                    Err(e) => return Err(e),
                    Ok(v) => writeln!(output, "{}", v)?,
                }
            },
            "delete" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set"))
                }
                if let Err(e) =  self.database.delete(&args[2]) {
                    return Err(e);
                };
            }
            _ => {
                eprintln!("{USAGE}");
                return Err(anyhow!("command not recognized"))
            }
        }
        Ok(())
    }
}

trait Storage {
    fn list(&self) -> HashMap<String, String>;
    fn set(&mut self, key: &str, value: &str) -> Result<()>;
    fn get(&self, key: &str) -> Result<String>;
    fn delete(&mut self, key: &str) -> Result<()>;
}

#[derive(Serialize, Deserialize)]
struct FileDatabase {
    file: String,
    data: HashMap<String, String>
}

impl FileDatabase {
    fn connect(file: String) -> Result<Self> {
        match fs::File::open(&file) {
            Ok(f) => {
                if Self::is_file_empty(&file)? {
                    // don't try to read it to json, will get eof error
                    return Ok(FileDatabase{ file, data: HashMap::new() });
                }
                let reader = BufReader::new(f);                
                let data = serde_json::from_reader(reader)?;
                Ok(FileDatabase { file, data })
            },
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    match fs::File::create(&file) {
                        Err(e) => return Err(e.into()),
                        Ok(_) => Ok(FileDatabase{ file, data: HashMap::new() })
                    }
                } else {
                    return Err(anyhow!("unable to connect to db"));
                }
            },
        }
    }
    fn save_to_db(&self) -> Result<()> {
        // we know the file exists because we create it if it doesn't
        // during connect
        let file = fs::File::create(&self.file)?;
        serde_json::to_writer_pretty(file,&self.data)?;
        Ok(())
    }
    fn is_file_empty(file: &str) -> Result<bool> {
        let metadata = fs::metadata(file)?;
        Ok(metadata.len() == 0)
    }
}

impl Storage for FileDatabase {
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.data.insert(key.to_string(), value.to_string());
        self.save_to_db()?;
        Ok(())
    }
    fn get(&self, key: &str) -> Result<String> {
        match self.data.get(key) {
            Some(v) => return Ok(v.to_string()),
            None => return Err(anyhow!("not found")),
        }
    }
    fn delete(&mut self, key: &str) -> Result<()> {
        let _ = self.data.remove(key);
        self.save_to_db()?;
        Ok(())
    }
    fn list(&self) -> HashMap<String, String> {
        let mut m = HashMap::new();
        // m.extend(self.data.clone());
        m.clone_from(&self.data);
        m
    }
}