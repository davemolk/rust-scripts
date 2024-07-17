use anyhow::Result;
use rand::Rng;
use std::io;
use colored::Colorize;

use crate::ascii;
use crate::util;

struct NumberGuess {
    score: i32,
    guesses: i32,
}

impl NumberGuess {
    fn new() -> Self{
        NumberGuess{ score: 0, guesses: 10 }
    }
    fn play(&mut self) -> Result<bool> {
        println!("try to guess the mystery number (between 0 and 100)");
        println!("you've got {} chances, good luck!", self.guesses);
        let rand_number = rand::thread_rng().gen_range(0..=100);
        loop {
            if self.guesses == 0 {
                return Ok(false);
            }
            println!("enter your guess...");
            let mut guess = String::new();
            io::stdin().read_line(&mut guess)?;
            let g = guess.trim().parse::<i32>()?;
            if g == rand_number {
                println!("you got it!");
                self.score += 1;
                return Ok(true)
            } else if g < rand_number {
                println!("too low");
                self.guesses -= 1;
                println!("you've got {} guesses left", self.guesses);
            } else {
                println!("too high");
                self.guesses -= 1;
                println!("you've got {} guesses left", self.guesses);
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
    let (r, g, b) = util::color();
    println!("{}\n\n", ascii::MYSTERY_NUMBER.truecolor(r, g, b));
    game.run_game_loop()?;
    Ok(())
}