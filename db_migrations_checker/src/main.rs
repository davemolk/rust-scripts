use std::{fs, process::exit};
use std::collections::HashMap;
use regex::Regex;

fn main() -> std::io::Result<()> {
    let mut migration_files = Vec::new();

    // todo make dynamic
    for entry in fs::read_dir("./migrations")? {
        let dir = entry.unwrap().path();
        if let Some(extension) = dir.extension() {
            if extension == "sql" && dir.is_file()  {
                let f = String::from(dir.file_name().unwrap().to_str().unwrap());
                migration_files.push(f);
            }
        }
    }    
    
    if migration_files.len()%2 != 0 {
        println!("not even");
        exit(1)
    }
    
    // sort so we can compare up and down pairs
    migration_files.sort();
    
    let re_up = Regex::new(r"^(?<up_name>\d{5}_[\w\d_]+)\.up\.sql$").unwrap();
    let re_down = Regex::new(r"^(?<down_name>\d{5}_[\w\d_]+)\.down\.sql$").unwrap();

    // these are what we care about...ignore things like indexes, functions, etc.
    let db_structure = vec!["table", "type"];
    let mut i = 1;
    while i < migration_files.len() {
        // validate up format
        let up_name = migration_files[i].as_str();
        if !re_up.is_match(&up_name) {
            println!("{up_name} is not formatted correctly, exiting");
            exit(1)
        }
        // validate down format
        let down_name = migration_files[i-1].as_str();
        if !re_down.is_match(&down_name) {
            println!("{down_name} is not formatted correctly, exiting");
            exit(1)
        }
        // make sure up and down names match
        let up = re_up.captures(&up_name).unwrap().get(1).unwrap().as_str();
        let down = re_down.captures(&down_name).unwrap().get(1).unwrap().as_str();

        if up != down {
            println!("migration names {up} and {down} don't match");
            exit(1)
        }

        
        let mut m = HashMap::new();
        let re_create = Regex::new(r"^create (\w+) (?:if not exists )?(\w+)").unwrap();
        let re_drop= Regex::new(r"^drop (\w+) (?:if exists )?(\w+)").unwrap();
        // todo add path
        let up_path = format!("./migrations/{up_name}");
        
        for line in fs::read_to_string(&up_path).unwrap().lines() {
            let lower = line.to_lowercase();
            if re_create.is_match(&lower) {
                if db_structure.contains(&re_create.captures(&lower).unwrap().get(1).unwrap().as_str()) {
                    let entity = re_create.captures(&lower).unwrap().get(2).unwrap().as_str().to_string();
                    m.insert(entity, "create");
                }
                
            }
        }

        let down_path = format!("./migrations/{down_name}");
        for line in fs::read_to_string(&down_path).unwrap().lines() {
            let lower = line.to_lowercase();
            if re_drop.is_match(&lower) {
                // need error handling
                let entity: String = re_drop.captures(&lower).unwrap().get(2).unwrap().as_str().to_string();
                if !m.contains_key(&entity) {
                    println!("{entity} missing for down migration");
                    exit(1)
                }
                m.remove(&entity).unwrap();
            }
        }

        if m.len() > 0 {
            for k in m.keys() {
                println!("{k} is missing from the down migration")
            }
            exit(1)
        }


        // confirm we have no duplicate numbers and no gaps
        let num_up = up.split("_").next().unwrap().parse::<usize>().unwrap();
        let num_down = down.split("_").next().unwrap().parse::<usize>().unwrap();
        if num_up + num_down - 1 != i {
            println!("migration numbers are wrong for {up} and {down}");
            exit(1)
        }
        i += 2;
    }
    Ok(())
}
