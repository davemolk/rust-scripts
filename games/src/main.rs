use colored::Colorize;
use games::{Game, color};

fn main() {
    let (r, g, b) = color();
    println!("{}", games::ascii::GAMES.truecolor(r, g, b));
    let game = Game::new();
    if let Err(e) = game.run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}