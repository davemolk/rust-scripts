use std::collections::HashMap;
use std::io::BufReader;
use std::io;
use std::fs::{File, OpenOptions};
use rand::Rng;
use rand::seq::SliceRandom;
use colored::Colorize;
use anyhow::{anyhow, Result};
use serde_derive::{Deserialize, Serialize};
mod ascii;

pub struct User {
    name: String,
    score: u16,
    high_scores: HashMap<String, u16>,
    file_name: String,
}


impl User {
    pub fn new() -> Self {
        println!("what's your name?");
        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();
        let mut cleaned_name = name.trim().to_owned();
        if cleaned_name.is_empty() {
            println!("everyone has a name...let's call you pooh");
            cleaned_name = String::from("pooh");
        }
        let path = String::from("high_scores.json");
        let scores = match Self::load_scores(&path) {
            Ok(s) => s,
            Err(_) => HashMap::new(),
        };
        println!("{:?}", scores);
        User{ name: cleaned_name, score: 0, high_scores: scores, file_name: path}
    }
    pub fn play(&mut self) -> Result<()> {
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
        loop {
            let operations = vec![0, 1];
            let operation = match operations.choose(&mut rand::thread_rng()) {
                Some(o) => o,
                None => &0,
            };
            let num1 = rand::thread_rng().gen_range(1..=largest);
            let num2 = rand::thread_rng().gen_range(1..=largest);
            let keep_playing = self.solve_math_problem(num1, num2, *operation)?;
            if !keep_playing {
                break
            }
        }
        Ok(())
    }
    fn solve_math_problem(&mut self, num1: i32, num2: i32, operation: i32) -> Result<bool> {
        let want = match operation {
            0 => num1 + num2,
            1 => if num1 - num2 < 0 { num2 - num1 } else { num1 - num2 },
            _ => num1 + num2,
        };
        match operation {
            0 => println!("what is {} + {}?", num1, num2),
            1 => if num1 - num2 < 0 { println!("what is {} - {}?", num2, num1) } else { println!("what is {} - {}?", num1, num2) },
            _ => {},
        }
        let mut guess_count = 0;
        loop {
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
                if let Some(score) = self.high_scores.get(&self.name) {
                    if self.score > *score {
                        println!("{}", ascii::NEW_HIGH_SCORE.red());
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
        let praise = vec!["good job!", "awesome!", "right on!", "cowabunga!", "gnarly!", "phat!", "swell!", "bodacious!", "party on!", "sizzling!", "dope!", "super!", "excellent!", "you got it!", "you rock!", "you're awesome!", "you're the best!", "rock on!", "you're doing great!", "kick butt!", "yay!", "hurray!", "oh boy!"];
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
