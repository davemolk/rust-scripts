use math::User;
use clap::Parser;

fn main() {
    let args = math::Args::parse();
    let mut user = User::new(args);
    if let Err(e) = user.play() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
