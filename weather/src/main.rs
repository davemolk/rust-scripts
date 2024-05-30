mod client;
use clap::Parser;

fn main() {
    let args = client::Args::parse();
    let client = client::Client::new(args);
    if let Err(e) = client.run() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}