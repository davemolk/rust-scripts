use std::env;
use std::path::{Path, PathBuf};
use math::User;
use clap::Parser;

fn main() {
    let args = math::Args::parse();
    let path = match find_game_binary(&args.game_binary) {
        None => {
            eprintln!("Error: can't locate game binary");
            std::process::exit(1);
        },
        Some(path) => path,
    };
    let mut user = User::new(args, path);
    if let Err(e) = user.play() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}

fn find_game_binary(from_flag: &Option<String>) -> Option<PathBuf> {
    if let Some(path) = from_dir() {
        return Some(path)
    }
    if let Some(path) = from_env() {
        return Some(path)
    }
    if let Some(path) = from_flag {
        let p = Path::new(path);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }
    None
}

fn from_dir() -> Option<PathBuf> {
    let curr = env::current_dir().ok()?;
    let path = curr.join("game");
    if path.is_file() {
        return Some(path);
    }
    None
}

fn from_env() -> Option<PathBuf> {
    let env_path = env::var("GAME_BINARY_PATH").ok()?;
    let path = Path::new(&env_path);
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    None
}