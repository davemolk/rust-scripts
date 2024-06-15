use anyhow::{Result, anyhow};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Write};

pub const USAGE: &str = r"Usage:
kvs list                                    list all keys in db
kvs get    <key>                            get the value for given key
kvs set    <key>    <value>                 set a value for a given key, overwrites any existing value(s)
kvs setk   ...<key> <value>                 set a value to multiple keys, appending in each case
kvs setv   <key>    <value>                 append a new value to the given key
kvs upd    <key>    <new_key>               update a key name
kvs upd    <key>    <value>    <new_value>
kvs remove <key>    <value>                 removes a value from a key
kvs delete <key>                            deletes a key and its value(s)
kvs undo                                    roll back the last set/setv/delete operation
kvs backup <new_file_name>                  makes a copy of the current db file
kvs help                                    prints usage
";

pub struct Runner {
    database: FileDatabase,
}

impl Runner {
    pub fn new(database: FileDatabase) -> Self {
        Runner { database }
    }
    pub fn run(&mut self, output: &mut dyn Write, args: Vec<String>) -> Result<()> {
        if args.is_empty() {
            eprintln!("{USAGE}");
            return Err(anyhow!("not enough args to run"))
        }
        match args[0].as_str() {
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
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for set command"))
                }
                self.database.set(&args[1], &args[2])?;
            },
            "setk" => {
                if args.len() < 2 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args to set multiple keys"))
                }
                let vec_of_str_refs: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
                let slice_of_str_refs: &[&str] = &vec_of_str_refs;
                self.database.set_multiple_keys(slice_of_str_refs)?;
            }
            "setv" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args to set multiple values"))
                }
                self.database.set_multiple_values(&args[1], &args[2])?;
            }
            "upd" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args to update key name"))
                }
                if args.len() == 3 {
                    self.database.update_key(&args[1], &args[2])?;
                } else {
                    // self.database.update_value(&args[1], &args[2], args[3])
                }
                
            }
            "duplicate" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args to duplicate a key"))
                }
                self.database.duplicate(&args[1], &args[2])?;
            }
            "get" => {
                if args.len() < 2 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for get"))
                }
                match self.database.get(&args[1]) {
                    Err(e) => return Err(e),
                    Ok(v) => {
                        for s in v {
                            writeln!(output, "{}", s)?
                        }
                    },
                }
            },
            "delete" => {
                if args.len() < 2 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for delete command"))
                }
                if let Err(e) =  self.database.delete(&args[1]) {
                    return Err(e);
                };
            }
            "remove" => {
                if args.len() < 3 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for remove command"))
                }
                if let Err(e) = self.database.remove(&args[1], &args[2]) {
                    return Err(e);
                };
            }
            "undo" => {
                if let Err(e) = self.database.undo() {
                    return Err(e);
                }
            }
            "backup" => {
                if args.len() < 2 {
                    eprintln!("{USAGE}");
                    return Err(anyhow!("not enough args for backup command"))
                }
                if let Err(e) = self.database.backup(&args[1]) {
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
    fn set_multiple_values(&mut self, key: &str, value: &str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        self.restore(key, value)?;
        Ok(())
    }
    fn update_key(&mut self, key: &str, updated_key: &str) -> Result<()> {
        // TODO will need to delete new key on undo, might need a new mechanism for this...
        self.save_to_tmp(key)?;
        if !self.data.contains_key(key) {
            return Err(anyhow!("key not found"));
        }
        // get values from key and insert in new key
        let values = self.get(key)?;
        self.data.insert(updated_key.to_owned(), Self::values_to_insert(values));
        // remove old key
        self.delete(key)?;
        self.save_to_db()?;
        Ok(())
    }
    fn duplicate(&mut self, key: &str, new_key: &str) -> Result<()> {
        // save so we can run "undo"
        self.save_to_tmp(key)?;
        // get values for first key
        let values = self.get(key)?;
        // use set_multiple_values in case there are already values at the new
        for value in Self::values_to_insert(values) {
            self.set_multiple_values(new_key, &value)?;
        }
        self.save_to_db()?;
        Ok(())
    }
    fn values_to_insert(from_db: Vec<&String>) -> Vec<String> {
        from_db.iter().cloned().map(|e| e.to_owned()).collect()
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
    fn set_multiple_keys(&mut self, args: &[&str]) -> Result<()> {
        if let Some(value) = args.last() {
            for key in &args[..args.len() - 1] {
                self.set_multiple_values(key, value)?;
            }
        };
        Ok(())
    }
    fn backup(&self, file_name: &str) -> Result<()> {
        let _ = fs::File::create_new(file_name)?;
        std::fs::copy(self.file.as_str(), file_name)?;
        Ok(())
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
        if Self::is_file_empty(&tmp_name)? {
            // would happen if the set is the first operation (so undo
            // file is created but contains nothing). so let's get the 
            // key and delete it.
            let mut key_iter = self.data.keys();
            if let Some(key) = key_iter.next() {
                self.delete_key_with_no_backup(&key.to_owned())?;
            };
            
        }
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
    use std::io::Read;

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
        let key = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], second_value);
        d.cleanup().unwrap();
    }
    #[test]
    fn set_multiple_values_appends_to_existing_value() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set_multiple_values(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(v[1], second_value);
        d.cleanup().unwrap();
    }
    #[test]
    fn undo_for_set_multiple_values() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set_multiple_values(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set_multiple_values(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(v[1], second_value);
        d.file_database.undo().unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(1, v.len());
        d.cleanup().unwrap();
    }
    #[test]
    fn undo_for_set_value() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(1, v.len());
        d.file_database.undo().unwrap();
        assert!(d.file_database.data.is_empty(), "db wasn't empty");
        d.cleanup().unwrap();
    }
    #[test]
    fn undo_for_set_value_multiple_times() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set(key, second_value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], second_value);
        assert_eq!(1, v.len());
        d.file_database.undo().unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], first_value);
        assert_eq!(1, v.len());
        d.cleanup().unwrap();
    }
    #[test]
    fn backup() {
        let mut d = TestDB::new();
        let key = "foo";
        let value = "bar";
        // set in normal db
        d.file_database.set(key, value).unwrap();
        let v = d.file_database.get(key).unwrap();
        assert_eq!(v[0], value);
        // create a backup
        let new_db = "backup.db";
        d.file_database.backup(new_db).unwrap();
        // read backup
        let mut f = fs::File::open(&new_db).unwrap();
        let mut backup_json = String::new();
        fs::File::read_to_string(&mut f, &mut backup_json).unwrap();
        // deserialize into map
        let c: HashMap<String, Vec<String>> = serde_json::from_str(&backup_json).unwrap();
        assert_eq!(d.file_database.data, c);
        d.cleanup().unwrap();
        fs::remove_file(new_db).unwrap();
    }
    #[test]
    fn set_multiple_keys() {
        let mut d = TestDB::new();
        let key1 = "key1";
        let key2 = "key2";
        let key3 = "key3";
        let value: &str = "value";
        let keys:&[&str] = &[key1, key2, key3, value];
        d.file_database.set_multiple_keys(keys).unwrap();
        let key1_value = d.file_database.get(key1).unwrap();
        assert_eq!(*key1_value[0], value.to_owned());
        let key2_value = d.file_database.get(key2).unwrap();
        assert_eq!(*key2_value[0], value.to_owned());
        let key3_value = d.file_database.get(key3).unwrap();
        assert_eq!(*key3_value[0], value.to_owned());
        d.cleanup().unwrap();
    }
    #[test]
    fn update_key() {
        let mut d = TestDB::new();
        let key1 = "key1";
        let key2 = "key2";
        let value = "value".to_string();
        d.file_database.set(key1, &value).unwrap();
        d.file_database.update_key(key1, key2).unwrap();
        let value_at_upd_key = d.file_database.get(key2).unwrap();
        assert_eq!(*value_at_upd_key[0], value);
        // make sure old key no longer exists
        assert!(d.file_database.get(key1).is_err());
        let want = anyhow::Error::msg("not found");
        let got = d.file_database.get(key1).unwrap_err();
        assert_eq!(got.to_string(), want.to_string());
        d.cleanup().unwrap();
    }
    #[test]
    fn duplicate() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set_multiple_values(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set_multiple_values(key, second_value).unwrap();
        let new_key = "new";
        d.file_database.duplicate(key, new_key).unwrap();
        let v = d.file_database.get(key).unwrap();
        let v2 = d.file_database.get(new_key).unwrap();
        assert_eq!(v[0], v2[0]);
        assert_eq!(v[1], v2[1]);
        d.cleanup().unwrap();
    }
    #[test]
    fn duplicate_to_key_with_preexisting_values() {
        let mut d = TestDB::new();
        let key = "foo";
        let first_value = "bar";
        d.file_database.set_multiple_values(key, first_value).unwrap();
        let second_value = "baz";
        d.file_database.set_multiple_values(key, second_value).unwrap();
        let new_key = "new";
        let new_value1 = "bar2";
        let new_value2 = "baz2";
        d.file_database.set_multiple_values(new_key, new_value1).unwrap();
        d.file_database.set_multiple_values(new_key, new_value2).unwrap();
        let new_key_value = d.file_database.get(new_key).unwrap();
        assert_eq!(new_key_value.len(), 2);
        d.file_database.duplicate(key, new_key).unwrap();
        let v2 = d.file_database.get(new_key).unwrap();
        assert_eq!(v2.len(), 4); 
        d.cleanup().unwrap();
    }
}