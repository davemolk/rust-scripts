use anyhow::{Result, anyhow};
use std::io::{self, Write};
use crossterm::{
    cursor::{self, Hide, MoveTo, Show}, event::{self, KeyCode, KeyEvent}, 
    queue, 
    style::{self, Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize}, 
    
    terminal::{self, enable_raw_mode, Clear, ClearType, SetSize}, 
    ExecutableCommand, 
};
use std::time::Duration;
use rand::Rng;

use crate::Difficulty;

const WIDTH: u16 = 40;
const BANNER_HEIGHT: u16 = 6;
const BOARD_HEIGHT: u16 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

struct Game {
    score: u16,
    width: u16,
    height: u16,
    stdout: io::Stdout,
    normal_terminal: (u16, u16),
    hero: Point,
    coin: Point,
    difficulty: Difficulty,
    lives: u8,
}

impl Game {
    fn new(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Game {
        let w = match width {
            Some(w) => w,
            None => WIDTH
        };
        let h = match height {
            Some(h) => h,
            None => BOARD_HEIGHT,
        };
        let stdout = io::stdout();
        let normal_terminal = crossterm::terminal::size().unwrap();
        let hero = Point { x: w / 2, y: h / 2 };
        Game {
            width: w,
            height: h,
            score: 0,
            stdout,
            normal_terminal,
            hero,
            coin: Self::spawn_coin(&hero, w, h),
            difficulty,
            lives: 3,
        }
    }
    fn run(&mut self) -> Result<()> {
        self.prepare_screen()?;
        self.render()?;
        loop {
            if !self.handle_input()? {
                break;
            }
            self.update_game_state();
            self.render()?;
        }
        Ok(())
    }
    fn prepare_screen(&mut self) -> Result<()>{
        self.stdout
            .execute(SetSize(self.width, self.height))?
            .execute(Clear(ClearType::All))?
            .execute(Hide)?;
        Ok(())
    }
    
    fn render(&mut self) -> Result<()> {
        self.render_screen()?;
        self.draw_hero()?;
        self.draw_coin()?;
        Ok(())
    }
    fn update_game_state(&mut self) {
        if self.hero == self.coin {
            self.score += 1;
            self.coin = Self::spawn_coin(&self.hero, self.width, self.height);
        }
    }
    fn render_screen(&mut self) -> Result<()> {
        self.stdout.execute(SetForegroundColor(Color::DarkRed))?;

        // columns
        for y in 0..self.height + 1 + BANNER_HEIGHT {
            self.stdout
                .execute(MoveTo(0, y))?
                .execute(Print("X"))?
                .execute(MoveTo(self.width + 1, y))?
                .execute(Print("X"))?;
        }
        // rows
        for x in 0..self.width + 2 {
            self.stdout
                .execute(MoveTo(x, 0))?
                .execute(Print("X"))?
                .execute(MoveTo(x, self.height + 1))?
                .execute(Print("X"))?
                .execute(MoveTo(x, self.height + BANNER_HEIGHT))?
                .execute(Print("X"))?;
        }
        // corners
        self.stdout
            .execute(MoveTo(0, 0))?
            .execute(Print("X"))?
            .execute(MoveTo(self.width + 1, self.height + 1))?
            .execute(Print("X"))?
            .execute(MoveTo(self.width + 1, 0))?
            .execute(Print("X"))?
            .execute(MoveTo(0, self.height + 1))?
            .execute(Print("X"))?;
        // empty the background
        self.stdout.execute(ResetColor).unwrap();
        for y in 1..self.height + 1 {
            for x in 1..self.width + 1 {
                self.stdout
                    .execute(MoveTo(x, y)).unwrap()
                    .execute(Print(" ")).unwrap();
            }
        }
        // score
        self.stdout
            .execute(MoveTo((self.width + 2) / 2, self.height + 1 + BANNER_HEIGHT / 2))?
            .execute(Print(self.score))?;
        Ok(())
    }
    fn draw_hero(&mut self) -> Result<()> {
        self.stdout
            .execute(SetForegroundColor(Color::Grey))?
            .execute(MoveTo(self.hero.x, self.hero.y))?
            .execute(Print("@"))?
            .flush()?;
        Ok(())
    }
    fn spawn_coin(player: &Point, max_width: u16, max_height: u16) -> Point {
        let mut x: u16;
        let mut y: u16;
        loop {
            x = rand::thread_rng().gen_range(1..max_width);
            y = rand::thread_rng().gen_range(1..max_height);
            if x != player.x && y != player.y {
                return Point{x, y};
            }
        }
    }
    fn draw_coin(&mut self) -> Result<()> {
        self.stdout
            .execute(SetForegroundColor(Color::DarkYellow))?
            .execute(MoveTo(self.coin.x, self.coin.y))?
            .execute(Print("o"))?
            .flush()?;
        Ok(())
    }
    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()?
            {
                match code {
                    KeyCode::Esc => return Ok(false),
                    KeyCode::Char('q') => return Ok(false),
                    KeyCode::Char('Q') => return Ok(false),
                    KeyCode::Up => if self.hero.y > 1 { self.hero.y -= 1; },
                    KeyCode::Down => if self.hero.y < self.height { self.hero.y += 1; },
                    KeyCode::Left => if self.hero.x > 1 { self.hero.x -= 1; },
                    KeyCode::Right => if self.hero.x < self.width { self.hero.x += 1; },
                    _ => {}
                }
            }
        }
        Ok(true)
    }
    fn cleanup(&mut self) -> Result<()> {
        let (x, y) = self.normal_terminal;
        self.stdout
            .execute(terminal::SetSize(x, y))?
            .execute(Clear(ClearType::All))?
            .execute(Show)?
            .execute(ResetColor)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}


pub fn run_treasure_seek(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Result<()> {
    let mut game = Game::new(width, height, difficulty);
    enable_raw_mode()?;
    game.run()?;
    game.cleanup()?;
    Ok(())
}