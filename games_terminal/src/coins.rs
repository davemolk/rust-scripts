use std::io;
use anyhow::Result;
use crossterm::{
    event::KeyCode, 
    style::Color,
};
use std::time::Duration;
use rand::Rng;

use crate::{input, movement, render};

use super::{
    Difficulty,
    movement::Direction,
    point::{MovingPoint, Point},
    WIDTH,
    BOARD_HEIGHT,
    BANNER_HEIGHT,
    DURATION,
    timer::Timer,
};

struct Game {
    score: u16,
    width: u16,
    height: u16,
    lives: u8,
    stdout: io::Stdout,
    difficulty: Difficulty,
    // coins: HashSet<MovingPoint>,
    coins: Vec<MovingPoint>,
    total_coins: usize,
    player: Point,
    normal_terminal: (u16, u16),
    duration: Duration,
}

impl Game {
    fn new(width: Option<u16>, height: Option<u16>, difficulty: Difficulty, duration: Option<Duration>) -> Self {
        let w = match width {
            Some(w) => w,
            None => WIDTH
        };
        let h = match height {
            Some(h) => h,
            None => BOARD_HEIGHT,
        };
        let d = match duration {
            Some(d) => d,
            None => Duration::from_secs(DURATION),
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
            coins: Vec::new(),
            total_coins: 1,
            normal_terminal,
            duration: d,
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
    fn spawn_coins(&mut self) {
        while self.coins.len() < self.total_coins {
            let mut x: u16;
            let mut y: u16;
            loop {
                x = rand::thread_rng().gen_range(1..self.width);
                y = rand::thread_rng().gen_range(1..self.height);
                if x != self.player.x && y != self.player.y {
                    if !movement::detect_moving_collision(Point{x, y}, &self.coins) {
                        self.coins.push(MovingPoint{ 
                            position: Point{x, y}, 
                            direction: Direction::random(), 
                            speed: 1,
                            wait_to_draw: 0,
                        });
                        break
                    }
                }
            }
        }
    }
    fn update_state(&mut self) {
        let mut to_remove: Vec<MovingPoint> = Vec::new();
        self.coins.retain(|coin| {
            if self.player == coin.position {
                to_remove.push(coin.clone());
                self.score += 1;
                // remove coin from set
                false
            } else {
                true
            }
        });
        if self.coins.is_empty() {
            self.total_coins += 1;
            self.spawn_coins();
        }
        self.move_coins();
    }
    fn move_coins(&mut self) {
        for coin in self.coins.iter_mut() {
            coin.update_moving_position(self.width, self.height);
        }
    }
    fn render(&mut self) -> Result<()> {
        render::render_screen(&mut self.stdout, self.width, self.height, Color::DarkBlue)?;
        render::render_banner(&mut self.stdout, self.height, self.lives, self.score, self.difficulty)?;
        render::draw_point(&mut self.stdout, '@', self.player.x, self.player.y, Color::DarkMagenta)?;
        render::draw_moving_points(&mut self.stdout, 'o', &self.coins, Color::DarkYellow)?;
        Ok(())
    }
    fn run(&mut self) -> Result<()> {
        render::prepare_screen(&mut self.stdout, self.width, self.height)?;
        self.spawn_coins();
        self.render()?;
        let timer = Timer::new(self.duration);
        loop {
            if timer.has_expired() {
                // print thanks for playing in banner w/ small sleep
                break
            }
            if let Some(key) = input::poll_for_event() {
                if !self.handle_input(key.code) {
                    break
                }
            }
            self.update_state();
            self.render()?;
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }
}

pub fn run_coins(width: Option<u16>, height: Option<u16>, difficulty: Difficulty, duration: Option<Duration>) -> Result<()> {
    let mut game = Game::new(width, height, difficulty, duration);
    game.run()?;
    let (x, y) = game.normal_terminal;
    render::cleanup(&mut game.stdout, x, y)?;
    Ok(())
}