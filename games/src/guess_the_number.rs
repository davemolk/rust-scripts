use anyhow::Result;
use rand::Rng;
use std::io;
use std::cmp::Ordering;
use colored::Colorize;

use super::{
    ascii,
    color,
};

struct NumberGuess {
    score: i32,
    guesses: i32,
}

impl NumberGuess {
    fn new() -> Self{
        NumberGuess{ score: 0, guesses: 10 }
    }
    fn get_user_input(&self) -> i32 {
        loop {
            let mut guess = String::new();
            io::stdin().read_line(&mut guess).expect("failed to read input");
            println!();
            match guess.trim().parse::<i32>() {
                Ok(g) => 
                    return g,
                Err(_) => {
                    eprintln!("please try again");
                    continue
                }
            };
        }
    }
    fn play(&mut self) -> Result<bool> {
        println!("try to guess the mystery number (between 0 and 100)");
        println!("you've got {} chances, good luck!\n", self.guesses);
        let rand_number = rand::thread_rng().gen_range(0..=100);
        loop {
            if self.guesses == 0 {
                let (r, g, b) = color();
                println!("{} ", "thanks for playing!".truecolor(r, g, b));
                println!("{} {}{}\n", "you scored".truecolor(r, g, b), self.score, ", awesome job!".truecolor(r, g, b));
                return Ok(false);
            }
            println!("enter your guess...");
            let g = self.get_user_input();            
            match g.cmp(&rand_number) {
                Ordering::Equal => {
                    let (r, g, b) = color();
                    println!("{}", "you got it!".truecolor(r, g, b));
                    self.score += 1;
                    return Ok(true)
                },
                Ordering::Less => {
                    println!("too low");
                    self.guesses -= 1;
                    if self.guesses != 0 {
                        println!("you've got {} guesses left\n", self.guesses);
                    }
                },
                _ => {
                    println!("too high");
                    self.guesses -= 1;
                    if self.guesses != 0 {
                        println!("you've got {} guesses left\n", self.guesses);
                    }
                }
            }
        }
    }
    fn run_game_loop(&mut self) -> Result<()> {
        loop {
            let keep_playing = self.play()?;
            if !keep_playing {
                break
            }
        }
        Ok(())
    }
}

pub fn run_number_guess() -> Result<()> {
    let mut game = NumberGuess::new();
    let (r, g, b) = color();
    println!("{}\n\n", ascii::MYSTERY_NUMBER.truecolor(r, g, b));
    game.run_game_loop()?;
    Ok(())
}