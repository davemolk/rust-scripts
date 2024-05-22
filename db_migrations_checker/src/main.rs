use std::{fs, process::exit};
use std::collections::HashSet;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_NO_MULTISPACES: Regex = Regex::new(r"\s{2,}").unwrap();
    
    static ref RE_UP: Regex = Regex::new(r"^(?<up_name>\d{5}_[\w]+)\.up\.sql$").unwrap();
    static ref RE_DOWN: Regex = Regex::new(r"^(?<down_name>\d{5}_[\w]+)\.down\.sql$").unwrap();

    static ref RE_CREATE: Regex = Regex::new(r"create (?:or replace )?(\w+) (?:if not exists\s+)?(\w+)").unwrap();
    static ref RE_ADD: Regex = Regex::new(r"add (\w+) (?:if not exists\s+)?(\w+)").unwrap();
    static ref RE_DROP: Regex = Regex::new(r"drop (\w+) (?:if exists\s+)?([\w,\s]+)").unwrap();
    static ref RE_DROP_COLUMN: Regex = Regex::new(r"drop column (?:if exists\s+)?([\w]+)").unwrap();
    static ref RE_RENAME_COLUMN: Regex = Regex::new(r"rename column (?:if exists )?(\w+)").unwrap();
}

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
        eprintln!("missing a migration file (total number is not even)");
        exit(1)
    }
    
    // sort so we make sure up and down migrations are next to each other
    migration_files.sort();

    // things that we are going to check and make sure are handled appropriately in the
    // up and down migrations.
    let db_entities = vec!["table", "type", "column", "view"];
    let mut i = 1;
    while i < migration_files.len() {
        // validate up-format
        let up_name = &migration_files[i];
        if !RE_UP.is_match(&up_name) {
            eprintln!("{up_name} is not formatted correctly");
            exit(1)
        }
        // validate down-format
        let down_name = &migration_files[i-1];
        if !RE_DOWN.is_match(&down_name) {
            eprintln!("{down_name} is not formatted correctly");
            exit(1)
        }
        // make sure up and down names match
        let up = RE_UP.captures(&up_name).unwrap().get(1).unwrap().as_str();
        let down = RE_DOWN.captures(&down_name).unwrap().get(1).unwrap().as_str();

        if up != down {
            eprintln!("migration names {up} and {down} don't match");
            exit(1)
        }
        
        let mut set = HashSet::new();

        // handle up-migration
        let up_path = format!("{MIGRATION_DIR}/{up_name}");
        let up_file = fs::read_to_string(&up_path).expect("read up-migration to a string");
        let up_cleaned = RE_NO_MULTISPACES.replace_all(&up_file, " ").to_lowercase();
        for (_, [entity, name]) in RE_CREATE.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) {
                set.insert(get_key(entity, name));
            }
        }
        for (_, [entity, name]) in RE_ADD.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) {
                set.insert(get_key(entity, name));
            }
        }
        for (_, [name]) in RE_DROP_COLUMN.captures_iter(&up_cleaned).map(|c| c.extract()) {
            // hard-code column because that's all we're looking for
            set.insert(get_key("column", name));
        }
        for (_, [name]) in RE_RENAME_COLUMN.captures_iter(&up_cleaned).map(|c| c.extract()) {
            // hard-code column because that's all we're looking for
            set.insert(get_key("column", name));
        }
        for (_, [entity, name]) in RE_DROP.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_entities.contains(&entity) && entity != "column" {
                let names = remove_cascade(name);
                for name in names {
                    set.insert(get_key(entity, name));
                }
            }
        }

        // handle down-migration
        let down_path = format!("{MIGRATION_DIR}/{down_name}");
        let down_file = fs::read_to_string(&down_path).expect("read up-migration to a string");
        let down_cleaned = RE_NO_MULTISPACES.replace_all(&down_file, " ").to_lowercase();
        let migration_num = (i + 1) / 2;
        for (_, [entity, name]) in RE_CREATE.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key(entity, name)) {
                eprintln!("{name} not created for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key(entity, name));
        }
        for (_, [name]) in RE_DROP_COLUMN.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key("column", name)) {
                eprintln!("{name} not dropped for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key("column", name));
        }
        for (_, [name]) in RE_RENAME_COLUMN.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key("column", name)) {
                eprintln!("{name} not renamed for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key("column", name));
        }
        for (_, [entity, name]) in RE_ADD.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !set.contains(&get_key(entity, name)) {
                eprintln!("{name} not added for down migration {migration_num}");
                exit(1)
            }
            set.remove(&get_key(entity, name));
        }
        for (_, [entity, name]) in RE_DROP.captures_iter(&down_cleaned).map(|c| c.extract()) {
            // we handled previously with RE_DROP_COLUMN, so skip here
            if entity == "column" {
                continue
            }
            let names = remove_cascade(name);
                for name in names {
                    if !set.contains(&get_key(entity, name)) {
                        eprintln!("{name} not dropped in down migration {migration_num} (added in the up-migration)");
                        exit(1)
                    }
                    set.remove(&get_key(entity, name));
                }
        }

        if set.len() > 0 {
            for e in set {
                let name: Vec<&str> = e.split("-").collect();
                eprintln!("{} is missing from the down migration {migration_num}", name[1]);
            }
            exit(1)
        }

        // confirm we have no duplicate numbers and no gaps
        let num_up = up.split("_").next().unwrap().parse::<usize>().unwrap();
        let num_down = down.split("_").next().unwrap().parse::<usize>().unwrap();
        if num_up + num_down - 1 != i {
            eprintln!("migration numbers are wrong, missing {}", num_up - 1);
            exit(1)
        }
        i += 2;
    }
}

fn get_key(entity: &str, name: &str) -> String {
    format!("{entity}-{name}")
}

fn remove_cascade(name: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let names: Vec<&str> = name.split(",").collect();
    for name in names {
        // get rid of cascade and any leading whitespace
        let name = name.trim_end_matches(" cascade").trim_start();
        // name = name.trim_start();
        if name == "" {
            continue
        }
        out.push(name)
    }
    out
}