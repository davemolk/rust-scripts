use games::Game;
use clap::Parser;

fn main() {
    let args = games::Args::parse();
    let game = Game::new(args);
    if let Err(e) = game.run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
