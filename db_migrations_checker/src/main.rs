use std::{fs, process::exit};
use std::collections::HashSet;
use regex::Regex;

const MIGRATION_DIR: &str = "./migrations";

fn main() {
    let mut migration_files = Vec::new();

    for entry in fs::read_dir(MIGRATION_DIR).unwrap() {
        let dir = entry.unwrap().path();
        if let Some(extension) = dir.extension() {
            if extension == "sql" && dir.is_file()  {
                if let Some(name) = dir.file_name() {
                    migration_files.push(name.to_str().unwrap().to_string());
                }
            }
        }
    }    
    
    if migration_files.len()%2 != 0 {
        println!("missing a migration file (total number is not even)");
        exit(1)
    }
    
    // sort so we make sure up and down migrations are next to each other
    migration_files.sort();
    
    // get rid of multiple spaces so handling multiple drops is easier
    let re_no_multi_spaces = Regex::new(r"\s{2,}").unwrap();

    let re_up: Regex = Regex::new(r"^(?<up_name>\d{5}_[\w]+)\.up\.sql$").unwrap();
    let re_down = Regex::new(r"^(?<down_name>\d{5}_[\w]+)\.down\.sql$").unwrap();

    let re_create = Regex::new(r"create (\w+) (?:if not exists\s+)?(\w+)").unwrap();
    let re_drop= Regex::new(r"drop (\w+) (?:if exists\s+)?([\w,\s]+)").unwrap();
    // couldn't figure out how to handle multiple column drops w/ the re_drop...
    let re_drop_column = Regex::new(r"drop column (?:if exists\s+)?([\w]+)").unwrap();
    let re_add = Regex::new(r"add (\w+) (?:if not exists\s+)?(\w+)").unwrap();

    // things that we are going to check and make sure are handled appropriately in the
    // up and down migrations.
    let db_entities = vec!["table", "type", "column", "view"];
    let mut i = 1;
    while i < migration_files.len() {
        // validate up-format
        let up_name = &migration_files[i];
        if !re_up.is_match(&up_name) {
            println!("{up_name} is not formatted correctly");
            exit(1)
        }
        // validate down-format
        let down_name = &migration_files[i-1];
        if !re_down.is_match(&down_name) {
            println!("{down_name} is not formatted correctly");
            exit(1)
        }
        // make sure up and down names match
        let up = re_up.captures(&up_name).unwrap().get(1).unwrap().as_str();
        let down = re_down.captures(&down_name).unwrap().get(1).unwrap().as_str();

        if up != down {
            println!("migration names {up} and {down} don't match");
            exit(1)
        }
        
        // handle up-migration
        let mut set = HashSet::new();
        
        // todo add path
        let up_path = format!("{MIGRATION_DIR}/{up_name}");
        let up_file = fs::read_to_string(&up_path).expect("read up-migration to a string");
        let up_cleaned = re_no_multi_spaces.replace_all(&up_file, " ").to_lowercase();
        for (_, [entity, name]) in re_create.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) {
                set.insert(get_key(entity, name));
            }
        }
        for (_, [entity, name]) in re_add.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) {
                set.insert(get_key(entity, name));
            }
        }
        for (_, [name]) in re_drop_column.captures_iter(&up_cleaned).map(|c| c.extract()) {
            // hard-code column because that's all we're looking for
            set.insert(get_key("column", name));
        }
        for (_, [entity, name]) in re_drop.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) && entity != "column" {
                set.insert(get_key(entity, name));
            }
        }

        // handle down-migration
        let down_path = format!("{MIGRATION_DIR}/{down_name}");
        let down_file = fs::read_to_string(&down_path).expect("read up-migration to a string");
        let down_cleaned = re_no_multi_spaces.replace_all(&down_file, " ").to_lowercase();
        let migration_num = (i + 1) / 2;
        for (_, [entity, name]) in re_create.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key(entity, name)) {
                println!("{name} not created for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key(entity, name));
        }
        for (_, [name]) in re_drop_column.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key("column", name)) {
                println!("{name} not dropped for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key("column", name));
        }
        for (_, [entity, name]) in re_add.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key(entity, name)) {
                println!("{name} not added for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key(entity, name));
        }
        for (_, [entity, name]) in re_drop.captures_iter(&down_cleaned).map(|c| c.extract()) {
            // we handled previously with re_drop_column, so skip here
            if entity == "column" {
                continue
            }
            // handle the case where multiple entities are listed (comma-separated)
            let names: Vec<&str> = name.split(",").collect();
            for name in names {
                // get rid of cascade and any leading whitespace
                let mut name = name.trim_end_matches(" cascade");
                name = name.trim_start();
                if name == "" {
                    continue
                }
                if !set.contains(&get_key(entity, name)) {
                    println!("{name} not dropped in down migration {migration_num} (added in the up-migration)");
                    exit(1)
                }
                set.remove(&get_key(entity, name));
            }
        }

        if set.len() > 0 {
            for e in set {
                println!("{e} is missing from the down migration {migration_num}");
            }
            exit(1)
        }

        // confirm we have no duplicate numbers and no gaps
        let num_up = up.split("_").next().unwrap().parse::<usize>().unwrap();
        let num_down = down.split("_").next().unwrap().parse::<usize>().unwrap();
        if num_up + num_down - 1 != i {
            println!("migration numbers are wrong, missing {}", num_up - 1);
            exit(1)
        }
        i += 2;
    }
}

fn get_key(entity: &str, name: &str) -> String {
    format!("{entity}-{name}")
}