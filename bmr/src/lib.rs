use anyhow::{Result, anyhow};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, ErrorKind};

mod item;
mod list;
mod storage;


#[derive(Serialize, Deserialize, Debug)]
pub struct DB {
    file: String,
    data: HashMap<String, Vec<String>>
}

impl DB {
    pub fn connect(file: String) -> Result<Self> {
        let f = match File::open(&file) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                File::create(&file).map_err(|e| anyhow!("failed to create db: {}", e))?;
                return Ok(DB { file, data: HashMap::new() });
            }
            Err(e) => return Err(anyhow!("failed to open db: {}", e)),
        };
        
        let reader = BufReader::new(f);
        let data = if Self::is_empty(&file)? {
            HashMap::new()
        } else {
            serde_json::from_reader(reader)?
        };
        Ok(DB { file, data })
    }
    fn is_empty(file: &str) -> Result<bool> {
        let metadata = fs::metadata(file)?;
        Ok(metadata.len() == 0)
    }
    fn save(&mut self) -> Result<()> {
        let json = serde_json::to_string(&self.data)?;
        fs::write(&self.file, json)?;
        Ok(())
    }
    fn overview(&self) {
        for (key, list) in &self.data {
            println!("{} ({})", key, list.len());
        }
    }
    fn create_list(&mut self, key: String) -> Result<()> {
        self.data.insert(key.clone(), Vec::new());
        println!("boomr! created a new list called '{}'", key);
        self.save()
    } 
    fn add_to_list(&mut self, key: String, value: String) -> Result<()> {
        self.data.entry(key).and_modify(|values| values.push(value.clone())).or_insert(vec![value]);
        // println!("boomr! '{}' in '{}' is '{}'. got it", , )
        Ok(())
    }
}


#[derive(Debug)]
pub struct Boomr {
    db: DB,
}

impl Boomr {
    pub fn new(db: DB) -> Self {
        Boomr{ db }
    }
    pub fn run(&mut self, args: Vec<String>) -> Result<()> {
        match args.len() {
            0 => self.db.overview(),
            1 => self.db.create_list(args[0].clone())?,
            _ => return Err(anyhow!("too many arguments")),
        }
        Ok(())
    }
}
