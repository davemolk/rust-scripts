use std::io;
use std::env;

use kvs::{FileDatabase, Runner};

fn main() {
    let file_database = match FileDatabase::connect(String::from("kvs.db")) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1)
        },
        Ok(f) => f,
    };
    let mut runner = Runner::new(file_database);
    if let Err(e) = runner.run(&mut io::stdout(), env::args().collect()) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}