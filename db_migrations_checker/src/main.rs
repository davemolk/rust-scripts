use std::{fs, process::exit};
use std::collections::HashMap;
use regex::Regex;

fn main() {
    let mut migration_files = Vec::new();

    // todo make dynamic
    for entry in fs::read_dir("./migrations").unwrap() {
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
        println!("not even");
        exit(1)
    }
    
    // sort so we can compare up and down pairs
    migration_files.sort();
    
    // handle drops with entities on multiple lines
    let re_no_multi_spaces = Regex::new(r"\s{2,}").unwrap();

    let re_up: Regex = Regex::new(r"^(?<up_name>\d{5}_[\w]+)\.up\.sql$").unwrap();
    let re_down = Regex::new(r"^(?<down_name>\d{5}_[\w]+)\.down\.sql$").unwrap();

    let re_create = Regex::new(r"create (\w+) (?:if not exists\s+)?(\w+)").unwrap();
    // don't include ^ for drop
    let re_drop= Regex::new(r"drop (\w+) (?:if exists\s+)?([\w,\s]+)").unwrap();
    // let re_drop = Regex::new(r"drop (\w+)(?: if exists\s*)?(?:\s*)?([\w,\s]*);?$").unwrap();
    let re_add = Regex::new(r"add (\w+) (?:if not exists\s+)?(\w+)").unwrap();

    // these are what we care about...ignore things like indexes, functions, etc.
    let db_structure = vec!["table", "type", "column"];
    let mut i = 1;
    println!("{:?}", migration_files);
    while i < migration_files.len() {
        let migration_num = (i + 1) / 2;
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
        
        // todo move to hashmap
        let mut m = HashMap::new();
        
        // todo add path
        let up_path = format!("./migrations/{up_name}");
        let up_file = fs::read_to_string(&up_path).expect("read up-migration");
        let up_cleaned = re_no_multi_spaces.replace_all(&up_file, " ").to_lowercase();
        for (_, [entity, name]) in re_create.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_structure.contains(&entity) {
                m.insert(name, "create");
            }
        }
        for (_, [entity, name]) in re_add.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_structure.contains(&entity) {
                m.insert(name, "add");
            }
        }
        for (_, [entity, name]) in re_drop.captures_iter(&up_cleaned).map(|c| c.extract()) {
            if db_structure.contains(&entity) {
                // let name = name.trim_end_matches(" cascade;");
                m.insert(name, "drop");
            }
        }

        println!("m after up {:?}", m);
        // for line in fs::read_to_string(&up_path).unwrap().lines() {
        //     let lower = line.to_lowercase();
        //     if re_create.is_match(&lower) {
        //         if db_structure.contains(&re_create.captures(&lower).unwrap().get(1).unwrap().as_str()) {
        //             let entity: String = re_create.captures(&lower).unwrap().get(2).unwrap().as_str().to_string();
        //             m.insert(entity, "create");
        //         }
        //     }
        //     if re_drop.is_match(&lower) {
        //         if db_structure.contains(&re_drop.captures(&lower).unwrap().get(1).unwrap().as_str()) {
        //             let entity: String = re_drop.captures(&lower).unwrap().get(2).unwrap().as_str().to_string();
        //             let entity = entity.replace(" ", "");
        //             println!("replacing spaces: {entity}");
        //             // prob a way to drop "" here
        //             let foo: Vec<&str> = entity.split(",").collect();
        //             for f in foo {
        //                 if f == "" {
        //                     continue
        //                 }
        //                 m.insert(f.to_string(), "drop");
        //             }
        //             // m.insert(entity, "drop");
        //         }
        //     }
        //     if re_add.is_match(&lower) {
        //         if db_structure.contains(&re_add.captures(&lower).unwrap().get(1).unwrap().as_str()) {
        //             let entity: String = re_add.captures(&lower).unwrap().get(2).unwrap().as_str().to_string();
        //             m.insert(entity, "add");
        //         }
        //     }
        // }

        // println!("after up {:?}", m);

        let down_path = format!("./migrations/{down_name}");
        let down_file = fs::read_to_string(&down_path).expect("read up-migration");
        let down_cleaned = re_no_multi_spaces.replace_all(&down_file, " ").to_lowercase();
        println!("down cleaned: {down_cleaned}");
        for (_, [_, name]) in re_create.captures_iter(&down_cleaned).map(|c| c.extract()) {
            if !m.contains_key(&name) {
                println!("{name} not created for down migration {migration_num}");
                exit(1)
            }
            println!("create match: removing {name}");
            m.remove(&name).unwrap();
        }

        // todo add this in

        // for (_, [entity, name]) in re_add.captures_iter(&down_cleaned).map(|c| c.extract()) {
        //     if db_structure.contains(&entity) {
        //         m.insert(name, "add");
        //     }
        // }
        for (_, [_, name]) in re_drop.captures_iter(&down_cleaned).map(|c| c.extract()) {
            let foo: Vec<&str> = name.split(",").collect();
            println!("foo {:?}", foo);
            for f in foo {
                if f == "" {
                    continue
                }
                let f = f.trim_end_matches(" cascade");
                println!("f {f}");
                if !m.contains_key(f) {
                    println!("{f} not dropped in down migration {migration_num} (added in the up-migration)");
                    exit(1)
                }
                println!("drop match: removing {f}");
                m.remove(f).unwrap();
            }
        }

        println!("after down {:?}", m); 

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
}
