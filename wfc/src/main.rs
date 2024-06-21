use wfc::find_frequencies;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = find_frequencies(args) {
        eprintln!("failed to find frequencies: {}", e);
        std::process::exit(1);
    }
}