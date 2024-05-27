mod client;
use crate::client::LunchClient;

fn main() {
    let client = LunchClient::new();
    if let Err(e) = client.run() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
