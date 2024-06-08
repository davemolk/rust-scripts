use anyhow::{Result, anyhow};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Write};

pub const USAGE: &str = r"Usage:
kvs list                    list all keys in db
kvs get    <key>            get the value for given key
kvs set    <key> <value>    set a value for a given key, overwrites any existing value(s)
kvs setm   <key> <value>    append a new value to the given key
kvs delete <key>            deletes a key and its value(s)
kvs remove <key> <value>    removes a value from a key
kvs undo                    roll back the last set/setm/delete operation
kvs help                    prints 
";

pub struct Runner {
    database: FileDatabase,
}

impl Runner {
    pub fn new(database: FileDatabase) -> Self {
        Runner { database }
    }
    pub fn run(&mut self, output: &mut dyn Write, args: Vec<String>) -> Result<()> {
        if args.len() < 2 {
            eprintln!("{USAGE}");
            return Err(anyhow!("not enough args to run"))
        }
        match args[1].as_str() {
            "list" => {
                let keys = self.database.list();
                if keys.is_empty() {
                    eprintln!("db is empty");
                } else {
                    for k in keys {
                        println!("{k}");
                    }
                }
            }
            "set" => {
                if args.len() < 4 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set"))
                }
                self.database.set(&args[2], &args[3])?;
            },
            "setm" => {
                if args.len() < 4 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set multiple"))
                }
                self.database.set_multiple(&args[2], &args[3])?;
            }
            "get" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for get"))
                }
                match self.database.get(&args[2]) {
                    Err(e) => return Err(e),
                    Ok(v) => {
                        for s in v {
                            writeln!(output, "{}", s)?
                        }
                    },
                }
            },
            "delete" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for delete"))
                }
                if let Err(e) =  self.database.delete(&args[2]) {
                    return Err(e);
                };
            }
            "remove" => {
                if args.len() < 4 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for remove"))
                }
                if let Err(e) = self.database.remove(&args[2], &args[3]) {
                    return Err(e);
                };
            }
            "undo" => {
                if let Err(e) = self.database.undo() {
                    return Err(e);
                }
            }
            "help" => {
                eprintln!("{USAGE}");
                return Ok(());
            }
            _ => {
                eprintln!("{USAGE}");
                return Err(anyhow!("command not recognized"))
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileDatabase {
    file: String,
    data: HashMap<String, Vec<String>>
}

impl FileDatabase {
    pub fn connect(file: String) -> Result<Self> {
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
            Err(e) if e.kind() == ErrorKind::NotFound => {
                match fs::File::create(&file) {
                    Err(e) => return Err(e.into()),
                    Ok(_) => Ok(FileDatabase{ file, data: HashMap::new() })
                }
            },
            Err(_e) => return Err(anyhow!("unable to connect to db")),
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
    fn get_tmp_name(&self) -> String {
        format!("{}.tmp", &self.file)
    }
    fn save_to_tmp(&self, key: &str) -> Result<()> {
        let tmp_name = self.get_tmp_name();
        match fs::File::create(&tmp_name) {
            Err(e) => {
                return Err(anyhow!("failed to write to tmp file {}", e));
            },
            Ok(mut f) => {
                if let Some(value) = self.data.get(key) {
                    for v in value {
                        f.write(format!("{}:::{}\n", key, *v).as_bytes())?;
                    }
                };
            },
        }
        Ok(())
    }
    fn delete_key_with_no_backup(&mut self, key: &str) -> Result<()> {
        let _ = self.data.remove(key);
        self.save_to_db()?;
        Ok(())
    }
    fn restore(&mut self, key: &str, value: &str) -> Result<()> {
        match self.data.get_mut(key) {
            None => {
                let v = vec![value.to_string()];
                self.data.insert(key.to_string(), v);
            },
            Some(v) => {
                v.push(value.to_string());
            }
        }
        self.save_to_db()?;
        Ok(())
    }
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        let v = vec![value.to_string()];
        self.data.insert(key.to_string(), v);
        self.save_to_db()?;
        Ok(())
    }
    fn set_multiple(&mut self, key: &str, value: &str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        self.restore(key, value)?;
        Ok(())
    }
    fn get(&self, key: &str) -> Result<Vec<&String>> {
        match self.data.get(key) {
            Some(v) => {
                let mut values = Vec::from_iter(v);
                values.sort();
                return Ok(values);
            },
            None => return Err(anyhow!("not found")),
        }
    }
    fn delete(&mut self, key:&str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        let _ = self.data.remove(key);
        self.save_to_db()?;
        Ok(())
    }
    fn remove(&mut self, key: &str, value: &str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        if let Some(values ) = self.data.get_mut(key) {
            values.retain(|v| *v != value.to_string());
            let updated_values = values.to_vec();
            let _ = self.delete(key);
            self.data.insert(key.to_string(), updated_values);
            self.save_to_db()?;
        } else {
            // don't need file anymore
            let tmp_name = self.get_tmp_name();
            fs::remove_file(&tmp_name)?;
            return Err(anyhow!("key not found"));
        }
        Ok(())
    }
    fn list(&self) -> Vec<&String> {
        let mut keys = Vec::from_iter(self.data.keys());
        keys.sort();
        keys
    }
    fn undo(&mut self) -> Result<()> {
        let tmp_name = self.get_tmp_name();
        let file = match fs::File::open(&tmp_name) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(anyhow!("unable to undo, no backup found"));
            },
            Err(e) => return Err(e.into()),
        };
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let parts: Vec<&str> = line.split(":::").collect();
            if parts.len() != 2 {
                continue
            }
            // clear the entry so we can restore it from scratch
            if i == 0 {
                self.delete_key_with_no_backup(parts[0])?;
            }
            self.restore(parts[0], parts[1])?;
        }
        fs::remove_file(&tmp_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestDB {
        file_database: FileDatabase,
    }
    impl TestDB {
        fn new() -> Self {
            fs::File::create("foo.kv").unwrap();
            TestDB{
                file_database: FileDatabase { 
                    file: "foo.kv".to_string(),
                    data:  HashMap::new(),
                }
            }
        }
        fn cleanup(&self) -> Result<()> {
            let tmp = format!("{}.tmp", self.file_database.file.to_owned());
            match fs::remove_file(tmp) {
                Err(e) if e.kind() == ErrorKind::NotFound => {},
                Err(_e) => panic!("error deleting tmp"),
                Ok(_) => {},
            }
            match fs::remove_file(&self.file_database.file) {
                Err(e) if e.kind() == ErrorKind::NotFound => {},
                Err(_e) => panic!("error deleting tmp"),
                Ok(_) => {},
            }
            Ok(())
        }
    }
    #[test]
    fn new_store_is_empty() {
        let d = TestDB::new();
        assert!(d.file_database.data.is_empty(), "db wasn't empty");
        d.cleanup().unwrap();
    }
    #[test]
    fn wrong_key_returns_nothing() {
        let d = TestDB::new();
        if let Some(_v) = d.file_database.data.get("blah") {
            panic!("got data but shouldn't");
        }
        d.cleanup().unwrap();
    }
    #[test]
    fn gets_expected_data_for_key() {
        let mut d = TestDB::new();
        let key = "foo";
        let value = "bar";
        d.file_database.set(key, value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], value);
        d.cleanup().unwrap();
    }
    #[test]
    fn set_overwrites_existing_value() {
        let mut d = TestDB::new();
        let key: &str = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], second_value);
        d.cleanup().unwrap();
    }
    #[test]
    fn set_multiple_appends_to_existing_value() {
        let mut d = TestDB::new();
        let key: &str = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set_multiple(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(v[1], second_value);
        d.cleanup().unwrap();
    }
}