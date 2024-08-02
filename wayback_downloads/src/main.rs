use clap::Parser;

fn main() {
    let args = wayback_downloads::Args::parse();
    if let Err(e) = wayback_downloads::run(args) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
