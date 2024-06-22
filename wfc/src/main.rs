use wfc::run;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(args) {
        eprintln!("failed to find frequencies: {}", e);
        std::process::exit(1);
    }
}