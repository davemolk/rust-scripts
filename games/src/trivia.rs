use anyhow::anyhow;
use anyhow::Result;
use rand::seq::SliceRandom;
use std::fs;
use std::io;
use colored::Colorize;
use serde_derive::Deserialize;

use super::{
    ascii,
    color,
};

#[derive(Debug)]
struct Trivia {
    score: i32,
    guesses: i32,
    questions: Vec<Question>,
    remaning_questions: usize,
}

#[derive(Debug, Deserialize, Clone)]
struct Question {
    question: String,
    options: [String; 4],
    correct_answer_index: usize, 
    #[serde(default = "default_seen")]
    seen: bool,
}

fn default_seen() -> bool {
    false
}

impl Trivia {
    fn new(questions: Vec<Question>) -> Self{
        let remaning_questions = questions.len();
        Trivia{
            score: 0,
            guesses: 3,
            questions,
            remaning_questions,
        }
    }
    fn play(&mut self) -> Result<()> {
        loop {
            if self.remaning_questions == 0 {
                println!("you got all the questions correct! nice job :)\n");
                return Ok(())
            }
            let question = match self.generate_question() {
                Some(q) => q,
                None => return Err(anyhow!("failed to generate question")),
            };
            println!("{}\n", question.question);
            for (idx, opt) in question.options.iter().enumerate() {
                println!("{}: {}", idx+1, opt);
            }
            println!();
            'outer: loop {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                println!();
                let guess = input.trim();
                if guess.is_empty() {
                    eprintln!("please enter the number of your guess");
                    continue
                }
                if guess == "q" {
                    println!("thanks for playing\n\n");
                    return Ok(());
                }
                let idx: usize = guess.parse::<usize>()?;
                match idx {
                    1..=4 => {},
                    _ => return Err(anyhow!("answer must be a number between 1 and 4"))
                }
                // correct for 0-indexing
                if idx == question.correct_answer_index + 1 {
                    self.score += 1;
                    println!("you got it! now you have {} points\n", self.score);
                    for q in self.questions.iter_mut() {
                        if question.question == q.question {
                            q.seen = true;
                            self.remaning_questions -= 1;
                            break 'outer;
                        }
                    }
                }
                self.guesses -= 1;
                if self.guesses == 0 {
                    println!("thanks for playing! you scored {}", self.score);
                    return Ok(());
                }
                println!("you almost got it -- you have {} guesses remaining.\nlet's try again!\n", self.guesses);
            }
            
            
        }
    }
    fn generate_question(&mut self) -> Option<Question> {
        let mut rng = rand::thread_rng();
        let mut question: &Question;
        loop {
            question = self.questions.choose(&mut rng)?;
            if !question.seen {
                break
            }
        }
        Some(Question{
            question: question.question.to_owned(),
            options: question.options.to_owned(),
            correct_answer_index: question.correct_answer_index,
            seen: false,
        })
    }
}

pub fn run_trivia() -> Result<()> {
    let input = fs::read_to_string("./src/trivia.json")?;
    let questions: Vec<Question> = serde_json::from_str(&input)?;
    let mut game = Trivia::new(questions);
    let (r, g, b) = color();
    println!("{}\n\n", ascii::TRIVIA.truecolor(r, g, b));
    if let Err(e) = game.play() {
        eprintln!("{}", e);
        std::process::exit(1);
    };
    Ok(())
}
