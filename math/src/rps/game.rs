use rand::Rng;
use rand::seq::SliceRandom;
use anyhow::{anyhow, Result};

struct Game {
    score: i32,
    wrong_count: i32,
}

impl Game {
    fn new() -> Self {
        Game { score: 0, wrong_count: 0 }
    }
    fn play(&mut self) -> Result<()> {
        Ok(())
    }
}