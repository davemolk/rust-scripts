use anyhow::Result;
use clap::ValueEnum;
use rand::{self, Rng};
use std::fmt;
use std::io;
use colored::Colorize;
use clap::Parser;

mod treasure;
mod point;
mod coins;
mod timer;
mod input;
mod movement;
mod render;

const WIDTH: u16 = 30;
const BANNER_HEIGHT: u16 = 7;
const BOARD_HEIGHT: u16 = 10;

#[derive(Debug, Clone, Parser)]
#[command(version)]
pub struct Args {
    /// terminal width
    #[arg(short)]
    x: Option<u16>,
    /// terminal height
    #[arg(short)]
    y: Option<u16>,
    /// difficulty level
    #[arg(short, long, value_enum)]
    difficulty: Option<Difficulty>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
enum Difficulty {
    Warmup,
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Difficulty::Warmup => "warmup",
            Difficulty::Beginner => "beginner",
            Difficulty::Intermediate => "intermediate",
            Difficulty::Advanced => "advanced",
            Difficulty::Expert => "expert",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Copy, Clone)]
enum GameOptions {
    Coins,
    TreasureSeeker,
    Quit,
}

impl GameOptions {
    fn all() -> &'static[GameOptions] {
        &[
            GameOptions::Coins,
            GameOptions::TreasureSeeker,
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
            GameOptions::Coins => "Coins",
            GameOptions::TreasureSeeker => "Treasure Seeker",
            GameOptions::Quit => "Quit",
        };
        write!(f, "{s}")
    }
}

pub struct Game {
    args: Args,
    options: &'static [GameOptions],
    difficulty: Difficulty,
}

impl Game {
    pub fn new(args: Args) -> Self {
        let difficulty = match args.difficulty {
            Some(d) => d,
            None => Difficulty::Beginner,
        };
        Game{
            args,
            options: GameOptions::all(),
            difficulty,
        }
    }
    pub fn run(&self) -> Result<()> {
        let (r, g, b) = color();
        println!("{}", GAMES.truecolor(r, g, b));
        match self.prompt_user() {
            GameOptions::Coins => coins::run_coins(self.args.x, self.args.y, self.difficulty)?,
            GameOptions::TreasureSeeker => treasure::run_treasure_seek(self.args.x, self.args.y, self.difficulty)?,
            GameOptions::Quit => {
                println!("thanks for playing!");
                return Ok(());
            },
        }
        Ok(())
    }
    fn prompt_user(&self) -> GameOptions {
        println!("pick a game to play\n");
        GameOptions::list_options();
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("failed to read input");
            println!();
            let Ok(mut choice) = input.trim().parse::<usize>() else {
                eprintln!("invalid choice, please try again");
                continue
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

const GAMES: &str = r"
   ____ _____ _____ ___  ___  _____
  / __ `/ __ `/ __ `__ \/ _ \/ ___/
 / /_/ / /_/ / / / / / /  __(__  ) 
 \__, /\__,_/_/ /_/ /_/\___/____/  
/____/                             
";

pub fn color() -> (u8, u8, u8) {
    let r = rand::thread_rng().gen_range(0..=255);
    let g = rand::thread_rng().gen_range(0..=255);
    let b = rand::thread_rng().gen_range(0..=255);
    (r, g, b)
}
