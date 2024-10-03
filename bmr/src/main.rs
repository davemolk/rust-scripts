use std::env::args;
use bmr::{Boomr, DB};

fn main() {
    let db = match DB::connect("foo.json".to_string()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)    
        }
    };
    let args = args().skip(1).collect();
    if let Err(e) = Boomr::new(db).run(args) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
