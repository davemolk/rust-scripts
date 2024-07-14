use rand::seq::SliceRandom;
use anyhow::{anyhow, Result};
use std::{io, thread};
use std::time::Duration;

use crate::ascii;

struct RockPaperScissors {
    score: i32,
    lives: i32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Choice {
    Rock,
    Paper,
    Scissors,
}

#[derive(Debug)]
enum Outcome {
    Win,
    Lose,
    Tie,
}

impl RockPaperScissors {
    fn new() -> Self {
        RockPaperScissors { score: 0, lives: 3 }
    }
    fn play(&mut self) -> Result<()> {
        loop {
            let computer_choice = Self::computer_chooses();
            let user_choice = Self::prompt_user()?;
            let parsed = Self::parse_user(&user_choice)?;
            Self::one_two_three(parsed, computer_choice);
            match Self::determine_winner(parsed, computer_choice) {
                Outcome::Tie => {
                    println!("tie! let's play again");
                },
                Outcome::Lose => {
                    println!("you lose :/");
                    self.lives -= 1;
                    if self.lives == 0 {
                        println!("see you again soon");
                        break
                    }
                    println!("now you have {} lives left", self.lives)
                },
                Outcome::Win => {
                    println!("you win!");
                    self.score += 1;
                    println!("you have {} points. let's play!", self.score);
                }
            }
        }
        Ok(())
    }
    fn one_two_three(player: Choice, computer: Choice) {
        println!("{}", ascii::THREE);
        thread::sleep(Duration::from_secs(1));
        // move cursor up 6 lines
        print!("\x1B[6A");
        // clear them
        for _ in 0..6 {
            print!("\x1B[K");
            println!();
        }
        // go back
        print!("\x1B[6A");
        println!("{}", ascii::TWO);
        thread::sleep(Duration::from_secs(1));
        print!("\x1B[6A");
        for _ in 0..6 {
            print!("\x1B[K");
            println!();
        }
        print!("\x1B[6A");
        println!("{}", ascii::ONE);
        thread::sleep(Duration::from_secs(1));
        print!("\x1B[6A");
        for _ in 0..6 {
            print!("\x1B[K");
            println!();
        }
        print!("\x1B[6A");
        let player_art = match player {
            Choice::Paper => ascii::PAPER,
            Choice::Rock => ascii::ROCK,
            Choice::Scissors => ascii::SCISSORS
        };
        let computer_art = match computer {
            Choice::Paper => ascii::PAPER,
            Choice::Rock => ascii::ROCK,
            Choice::Scissors => ascii::SCISSORS
        };
        println!("{} {}", player_art, computer_art);
    }
    fn computer_chooses() -> Choice {
        let choices = vec![Choice::Rock, Choice::Paper, Choice::Scissors];
        match choices.choose(&mut rand::thread_rng()) {
            Some(c) => { *c },
            None => { Choice::Rock },
        }
    }
    fn prompt_user() -> Result<String> {
        println!("enter rock, paper, or scissors...good luck!");
        let mut user_rps = String::new();
        io::stdin().read_line(&mut user_rps)?;
        if user_rps.trim().is_empty() {
            eprintln!("you need to pick something...hmmm...");
            let choices = vec!["rock", "paper", "scissors"];
            match choices.choose(&mut rand::thread_rng()) {
                Some(c) => { 
                    user_rps = (*c.to_owned()).to_string();
                },
                None => { 
                    user_rps = "rock".to_string();
                },
            }
        }
        Ok(user_rps.trim().to_lowercase())
    }
    fn parse_user(s: &str) -> Result<Choice> {
        let choice = match s {
            "rock" => Choice::Rock,
            "r" => Choice::Rock,
            "paper" => Choice::Paper,
            "p" => Choice::Paper,
            "scissors" => Choice::Scissors,
            "s" => Choice::Scissors,
            _ => return Err(anyhow!("failed to parse rock, paper, or scissors from user input")),
        };
        Ok(choice)
    }
    fn determine_winner(player: Choice, computer: Choice) -> Outcome {
        if player == computer {
            return Outcome::Tie;
        }
        if player == Choice::Rock && computer == Choice::Paper || player == Choice::Paper && computer == Choice::Scissors || player == Choice::Scissors && computer == Choice::Rock {
            return Outcome::Lose;
        }
        Outcome::Win
    }
}

pub fn run_rps() -> Result<()> {
    let mut game = RockPaperScissors::new();
    game.play()?;
    Ok(())
}