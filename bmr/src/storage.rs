use std::io::{Read, Write};
use std::path::PathBuf;
use std::fs::{self, File, OpenOptions};
use serde_json::Value;

use super::{
    HashMap,
    Deserialize,
    Serialize,
    Result,
};
use crate::list::List;
use crate::item::Item;

#[derive(Serialize, Deserialize, Debug, Default)]
struct Storage {
    json_file_path: PathBuf,
    lists: Vec<List>,
}

impl Storage {
    const DEFAULT_JSON_FILE: &'static str = "/.boomr";
    pub fn new(custom_path: Option<String>) -> Self {
        let mut storage = Storage::default();
        storage.json_file_path = match custom_path {
            Some(path) => PathBuf::from(path),
            None => Storage::json_file(),
        };
        storage.bootstrap();
        storage.populate();
        storage
    }
    pub fn json_file() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(format!("{}{}", home, Storage::DEFAULT_JSON_FILE))
    }
    pub fn lists(&self) -> Vec<&List> {
        let mut sorted_lists = self.lists.iter().collect::<Vec<&List>>();
        sorted_lists.sort_by(|a,b| b.items.len().cmp(&a.items.len()));
        sorted_lists
    }
    pub fn list_exists(&self, name: &str) -> bool {
        self.lists.iter().any(|n| n.name == name)
    }
    pub fn items(&self) -> Vec<&Item> {
        self.lists.iter()
            .flat_map(|list| &list.items)
            .collect()
    }
    pub fn item_exists(&self, name: &str) -> bool {
        self.items().iter().any(|item| item.name == name)
    }
    pub fn to_hash(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let mut map = HashMap::new();
        let lists_vec: Vec<HashMap<String, String>> = self.lists.iter().map(|list| {
            let list_hash = list.to_hash();
            list_hash.into_iter().map(|(name, items)| {
                let items_str = items.into_iter().flat_map(|item| item.into_iter()).collect::<HashMap<_, _>>();
                HashMap::from([(name, serde_json::to_string(&items_str).unwrap())])
            }).collect::<Vec<_>>()
        }).flatten().collect();
    
        map.insert("lists".to_string(), lists_vec);
        map
    }    
    fn bootstrap(&self) -> Result<()> {
        let path = &self.json_file_path;
        if !path.exists() || path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            let _ = File::create(&path).and_then(|mut file| file.write_all(b"{}"));
            self.save();
        }
        Ok(())
    }
    fn populate(&mut self) {
        let path = &self.json_file_path;
        let mut file = File::open(path).expect("failed to open json");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("failed to read json");
        let parsed: Value = serde_json::from_str(&data).expect("invalid json");
        if let Some(lists) = parsed.get("lists").and_then(Value::as_array) {
            for list in lists {
                if let Some(list_name) = list.as_object() {
                    for (name, items) in list_name {
                        let mut list_instance = List::new(name.clone());
                        if let Some(items_array) = items.as_array() {
                            for item in items_array {
                                if let Some(item_obj) = item.as_object() {
                                    for (item_name, value) in item_obj {
                                        list_instance.add_item(Item::new(item_name.clone(), value.as_str().unwrap_or("").to_string()));
                                    }
                                }
                            }
                        }
                        self.lists.push(list_instance);
                    }
                }
            }
        }
    }
    pub fn save(&self) {
        let path = &self.json_file_path;
        let json_data = serde_json::to_string_pretty(&self.to_hash()).expect("failed to convert to JSON");
        let mut file = OpenOptions::new().write(true).truncate(true).open(path).expect("unable to open JSON file for writing");
        file.write_all(json_data.as_bytes()).expect("unable to write data to file");
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.to_hash()).expect("failed to convert to JSON")
    }
}