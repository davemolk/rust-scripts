use std::io;
use anyhow::Result;
use crossterm::{
    event::KeyCode, 
    style::Color,
};
use std::time::Duration;
use rand::Rng;

use crate::{input, render};

use super::{
    Difficulty,
    movement::Direction,
    point::{MovingPoint, Point},
    WIDTH,
    BOARD_HEIGHT,
    BANNER_HEIGHT,
    timer::Timer,
};

struct Game {
    score: u16,
    width: u16,
    height: u16,
    lives: u8,
    stdout: io::Stdout,
    difficulty: Difficulty,
    coins: Vec<MovingPoint>,
    player: Point,
    normal_terminal: (u16, u16),
}

impl Game {
    fn new(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Self {
        let w = match width {
            Some(w) => w,
            None => WIDTH
        };
        let h = match height {
            Some(h) => h,
            None => BOARD_HEIGHT,
        };
        let normal_terminal = crossterm::terminal::size().unwrap();
        Game{
            score: 0,
            width: w,
            height: h,
            lives: 1, // do i want this?
            stdout: io::stdout(),
            difficulty,
            player: Point { x: w / 2, y: h / 2 },
            coins: vec![],
            normal_terminal,
        }
    }
    fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => self.move_player(Direction::Up),
            KeyCode::Down => self.move_player(Direction::Down),
            KeyCode::Left => self.move_player(Direction::Left),
            KeyCode::Right => self.move_player(Direction::Right),
            KeyCode::Esc | KeyCode::Char('Q' | 'q') => return false,
            _ => {},
        }
        true
    }
    fn move_player(&mut self, direction: Direction) {
        match direction {
            Direction::Up => if self.player.y > 1 {
                self.player.y -= 1;
            },
            Direction::Down => if self.player.y < self.height {
                self.player.y += 1;
            }
            Direction::Left => if self.player.x > 1 {
                self.player.x -= 1;
            }
            Direction::Right => if self.player.x < self.width {
                self.player.x += 1;
            }
        }
    }
    fn update_state(&mut self) {
        
    }
    fn render(&mut self) -> Result<()> {
        render::render_screen(&mut self.stdout, self.width, self.height, Color::DarkBlue)?;
        render::render_banner(&mut self.stdout, self.height, self.lives, self.score, self.difficulty)?;
        render::draw_point(&mut self.stdout, '@', self.player.x, self.player.y, Color::DarkMagenta)?;

        Ok(())
    }
    fn run(&mut self) -> Result<()> {
        render::prepare_screen(&mut self.stdout, self.width, self.height)?;
        self.render()?;
        let duration = Duration::from_secs(5);
        let timer = Timer::new(duration);
        loop {
            if timer.has_expired() {
                break
            }
            if let Some(key) = input::poll_for_event() {
                if !self.handle_input(key.code) {
                    break
                }
            }
            // update state
            self.render()?;
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
}

pub fn run_coins(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Result<()> {
    let mut game = Game::new(width, height, difficulty);
    game.run()?;
    let (x, y) = game.normal_terminal;
    render::cleanup(&mut game.stdout, x, y)?;
    Ok(())
}