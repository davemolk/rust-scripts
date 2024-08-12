use games::Game;

fn main() {
    let game = Game::new();
    if let Err(e) = game.run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}