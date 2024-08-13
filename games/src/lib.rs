use anyhow::Result;
use rand::{self, Rng};
use std::fmt;
use std::io;

pub mod ascii;
mod deck;
mod guess_the_number;
mod hide_and_seek;
mod rock_paper_scissors;
mod trivia;

#[derive(Debug, Copy, Clone)]
enum GameOptions {
    RockPaperScissors,
    GuessTheNumber,
    HideAndSeek,
    Trivia,
    Quit,
}

impl GameOptions {
    fn all() -> &'static [GameOptions] {
        &[
            GameOptions::RockPaperScissors,
            GameOptions::GuessTheNumber,
            GameOptions::HideAndSeek,
            GameOptions::Trivia,
            GameOptions::Quit,
        ]
    }
    fn list_options() {
        for (i, game) in GameOptions::all().iter().enumerate() {
            println!("{}: {}", i+1, game);
        }
    }
}

impl fmt::Display for GameOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameOptions::RockPaperScissors => "Rock Paper Scissors",
            GameOptions::GuessTheNumber => "Guess the Number",
            GameOptions::HideAndSeek => "Hide and Seek",
            GameOptions::Trivia => "Trivia",
            GameOptions::Quit => "Quit",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct Game {
    options: &'static [GameOptions],
}

impl Game {
    pub fn new() -> Self {
        Game{
            options: GameOptions::all(),
        }
    }
    pub fn run(&self) -> Result<()> {
        match self.prompt_user() {
            GameOptions::RockPaperScissors => rock_paper_scissors::run_rps()?,
            GameOptions::GuessTheNumber => guess_the_number::run_number_guess()?,
            GameOptions::HideAndSeek => hide_and_seek::run_hide_and_seek()?,
            GameOptions::Trivia => trivia::run_trivia()?,
            GameOptions::Quit => {
                println!("thanks for playing!");
                return Ok(());
            },
            _ => {}
        };
        Ok(())
    }
    fn prompt_user(&self) -> GameOptions {
        println!("pick a game to play\n");
        GameOptions::list_options();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("failed to read input");
            let mut choice = match input.trim().parse::<usize>() {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("invalid choice, please try again");
                    continue
                }
            };
            choice = choice.saturating_sub(1);
            if choice >= self.options.len() {
                eprintln!("invalid choice, please try again");
                continue
            }
            return self.options[choice]
        }
    }
}



pub fn color() -> (u8, u8, u8) {
    let r = rand::thread_rng().gen_range(0..=255);
    let g = rand::thread_rng().gen_range(0..=255);
    let b = rand::thread_rng().gen_range(0..=255);
    (r, g, b)
}