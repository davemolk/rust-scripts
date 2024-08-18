use std::env;

fn main() {
    let args = env::args().skip(1).collect();
    let client = lobsters::LobsterClient::new();
    if let Err(e) = client.run(args) {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
