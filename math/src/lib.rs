use std::collections::HashMap;
use std::io::BufReader;
use std::io;
use std::fs::{File, OpenOptions};
use std::ops::Div;
use rand::Rng;
use rand::seq::SliceRandom;
use colored::Colorize;
use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};
use std::time::{Duration, SystemTime};

mod ascii;

#[derive(Parser, Debug)]
pub struct Args {
    /// largest number to use (default is 10)
    #[arg(short, long)]
    largest: Option<i32>,
    /// number of guesses, default is unlimited
    #[arg(short, long)]
    guesses: Option<i32>,
    /// operations to include (options are addition,
    /// subtraction, multiplication, and division).
    /// 
    /// enter the first initial of what
    /// you want to practice (some combo of asmd)
    /// 
    /// default is addition and subtraction
    #[arg(short, long)]
    #[arg(value_parser = parse_operations)]
    operations: Option<String>,
    /// game difficulty
    #[arg(short, long, value_enum)]
    difficulty: Option<Difficulty>,
}

fn parse_operations(arg: &str) -> Result<String> {
    if arg.trim().len() > 4 {
        return Err(anyhow!("valid input for operation flag is a, s, m, d, or some combination"));
    }
    let allowed: Vec<&str> = vec!["a", "s", "m", "d"];
    for c in arg.trim().to_lowercase().split("").into_iter() {
        if !allowed.contains(&c) {
            return Err(anyhow!("valid input for operation flag is a, s, m, d, or some combination"));
        }
    }
    Ok(arg.to_owned())
}

#[derive(Debug)]
enum Operations {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum Difficulty {
    #[default]
    Beginner,
    AdvancedBeginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug)]
pub struct User {
    name: String,
    score: u16,
    high_scores: HashMap<String, u16>,
    file_name: String,
    args: Args,
    operations: Vec<Operations>,
    show_high_score: bool,
    difficulty: Difficulty,
}

impl User {
    pub fn new(args: Args) -> Self {
        println!("what's your name?");
        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();
        let mut cleaned_name = name.trim().to_owned();
        if cleaned_name.is_empty() {
            println!("everyone has a name...let's call you pooh");
            cleaned_name = String::from("pooh");
        }
        let path = String::from("high_scores.json");
        let high_scores = match Self::load_scores(&path) {
            Ok(s) => { s },
            Err(_) => HashMap::new(),
        };
        let mut difficulty = Difficulty::Beginner;
        if let Some(ref d) = args.difficulty {
            difficulty = *d;
        }
        let mut operations = match difficulty {
            Difficulty::Beginner => vec![Operations::Addition, Operations::Subtraction],
            Difficulty::AdvancedBeginner => vec![Operations::Addition, Operations::Subtraction],
            Difficulty::Intermediate => vec![Operations::Addition, Operations::Subtraction, Operations::Multiplication, Operations::Division],
            Difficulty::Advanced => vec![Operations::Addition, Operations::Subtraction, Operations::Multiplication, Operations::Division],
            Difficulty::Expert => vec![Operations::Addition, Operations::Subtraction, Operations::Multiplication, Operations::Division],
        };
        if let Some(ref o) = args.operations {
            operations = Self::parse_operations(&o);
        }
        User{ args, name: cleaned_name, score: 0, high_scores, file_name: path, operations, show_high_score: true, difficulty }
    }
    fn parse_operations(args: &str) -> Vec<Operations> {
        if args.is_empty() {
            return vec![Operations::Addition, Operations::Subtraction];
        }
        let mut ops = vec![];
        let args = args.to_lowercase();
        if args.contains("a") {
            ops.push(Operations::Addition);
        }
        if args.contains("s") {
            ops.push(Operations::Subtraction);
        }
        if args.contains("m") {
            ops.push(Operations::Multiplication);
        }
        if args.contains("d") {
            ops.push(Operations::Division);
        }
        // don't error on bad input, just return default
        if ops.is_empty() {
            return vec![Operations::Addition, Operations::Subtraction];
        }
        ops
    }
    pub fn play(&mut self) -> Result<()> {
        let start_time = SystemTime::now();
        if self.high_scores.contains_key(&self.name) {
            println!("\nwelcome back {}! let's see if you can beat your previous high score of {:?}\n", self.name, self.high_scores.get(&self.name).unwrap());
        } else {
            println!("hi {}", self.name);
        }
        println!("press q when you're done playing...");
        println!("{}{}{}{}{} {}{}{}{}{}\n",
            "l".bright_red(), 
            "e".truecolor(255, 103, 0),
            "t".truecolor(255, 165, 0),
            "'".bright_yellow(),
            "s".bright_green(),
            "p".green(),
            "l".bright_blue(),
            "a".blue(),
            "y".bright_purple(),
            "!".purple(
            ),
        );
        self.run_game_loop()?;
        let end_time = SystemTime::now();
        if let Ok(dur) = end_time.duration_since(start_time) {
            let seconds = dur.as_secs() % 60;
            let minutes = (dur.as_secs() / 60) & 60;
            println!("you played for {} minutes and {} seconds and answered {} questions correctly, great job!", minutes, seconds, self.score);
        }
        Ok(())
    }
    fn load_scores(path: &str) ->Result<HashMap<String,u16>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        let reader = BufReader::new(file);
        let scores: HashMap<String, u16> = serde_json::from_reader(reader)?;
        Ok(scores)
    }
    fn run_game_loop(&mut self) -> Result<()> {
        let mut largest = 10;
        if let Some(n) = self.args.largest {
            largest = n;
        }
        loop {
            let num1 = rand::thread_rng().gen_range(1..=largest);
            let num2 = rand::thread_rng().gen_range(1..=largest);
            let keep_playing = self.solve_math_problem(num1, num2)?;
            if !keep_playing {
                break
            }
        }
        Ok(())
    }
    fn choose_operation(&self) -> &Operations {
        match self.operations.choose(&mut rand::thread_rng()) {
            Some(o) => { o },
            None => { &Operations::Addition},
        }
    }
    fn solve_math_problem(&mut self, num1: i32, num2: i32) -> Result<bool> {
        let op = self.choose_operation();
        let mut div_total = 0;
        // shadow in case it's 0 and division
        let mut num2 = num2;
        let want = match op {
            Operations::Addition => num1 + num2,
            Operations::Subtraction => if num1 - num2 < 0 { num2 - num1 } else { num1 - num2 },
            Operations::Multiplication => num1 * num2,
            Operations::Division => {
                if num2 == 0 {
                    num2 = 2;
                }
                div_total = num1 * num2;
                num1
            },
        };
        match op {
            Operations::Addition => println!("what is {} + {}?", num1, num2),
            Operations::Subtraction => if num1 - num2 < 0 { println!("what is {} - {}?", num2, num1) } else { println!("what is {} - {}?", num1, num2) },
            Operations::Multiplication => println!("what is {} * {}?", num1, num2),
            Operations::Division => println!("what is {} / {}?", div_total, num2),
        }
        let mut guess_count = 0;
        loop {
            if let Some(g) = self.args.guesses {
                if guess_count > g {
                    println!("all out of guesses, better luck next time!");
                    return Ok(false);
                }
            }
            let mut guess = String::new();
            io::stdin().read_line(&mut guess)?;
            if guess.trim().is_empty() {
                println!("please enter a guess");
                continue
            }
            if guess.trim().chars().nth(0).unwrap() == 'q' {
                let (r, g, b) = Self::color();
                println!("{}", ascii::THANKS_FOR_PLAYING.truecolor(r, g, b));
                // save current score
                self.high_scores.entry(self.name.to_owned())
                    .and_modify(|s| { *s = if self.score > *s { self.score } else { *s } })
                    .or_insert(self.score);
                let file = File::create(&self.file_name)?;
                    match serde_json::to_writer_pretty(file, &self.high_scores) {
                        Err(e) => return Err(anyhow!("failed to save high scores {}", e)),
                        Ok(_) => {},
                    }
                return Ok(false)
            }
            let guess: i32 = match guess.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("please enter a number for your guess");
                    continue
                },
            };
            if guess == want {
                self.score += 1;
                self.praise();
                if self.show_high_score {
                    if let Some(score) = self.high_scores.get(&self.name) {
                        if self.score > *score {
                            let (r, g, b) = Self::color();
                            println!("{}", ascii::NEW_HIGH_SCORE.truecolor(r, g, b));
                            self.show_high_score = false;
                        }
                    }
                }
                return Ok(true)
            }
            println!("try again...");
            guess_count += 1;
        }
    }
    fn praise(&self) {
        let (r, g, b) = Self::color();
        let praise = vec!["good job!", "awesome!", "right on!", "cowabunga!", "gnarly!", "phat!", "swell!", "bodacious!", "party on!", "sizzling!", "dope!", "party time!", "super!", "excellent!", "you got it!", "you rock!", "you're awesome!", "you're the best!", "rock on!", "you're doing great!", "kick butt!", "yay!", "hurray!", "oh boy!"];
        match praise.choose(&mut rand::thread_rng()) {
            Some(c) => println!("{}{} {}\n", c.truecolor(r, g, b), " now your score is".truecolor(r, g, b), self.score),
            None => println!("great job! now your score is {}\n", self.score),
        }
    }
    fn color() -> (u8, u8, u8) {
        let r = rand::thread_rng().gen_range(0..=255);
        let g = rand::thread_rng().gen_range(0..=255);
        let b = rand::thread_rng().gen_range(0..=255);
        (r, g, b)
    }
}
