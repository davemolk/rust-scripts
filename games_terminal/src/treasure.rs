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

use super::{
    Difficulty,
    Direction,
};

const WIDTH: u16 = 40;
const BANNER_HEIGHT: u16 = 7;
const BOARD_HEIGHT: u16 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MovingPoint {
    position: Point,
    direction: Direction,
    speed: u16,
    // todo: customize based on the difficulty and the game dimensions
    wait_to_draw: u8,
}

impl MovingPoint {
    // todo need to avoid collisions w/ cacti, rocks
    // also want to slow down the speed somehow
    fn update_position(&mut self, board_width: u16, board_height: u16) {
        self.wait_to_draw = 3;
        match self.direction {
            Direction::Up => {
                if self.position.y > 1 {
                    self.position.y = self.position.y.saturating_sub(self.speed);
                } else {
                    self.direction = Direction::Down;
                }
            }
            Direction::Down => {
                if self.position.y < board_height {
                    self.position.y = self.position.y.saturating_add(self.speed);
                } else {
                    self.direction = Direction::Up;
                }
            }
            Direction::Left => {
                if self.position.x > 1 {
                    self.position.x = self.position.x.saturating_sub(self.speed);
                } else {
                    self.direction = Direction::Right;
                }
            }
            Direction::Right => {
                if self.position.x < board_width {
                    self.position.x = self.position.x.saturating_add(self.speed);
                } else {
                    self.direction = Direction::Left;
                }
            }
        }
    }

}

struct Game {
    score: u16,
    width: u16,
    height: u16,
    stdout: io::Stdout,
    normal_terminal: (u16, u16),
    hero: Point,
    coin: MovingPoint,
    cacti: Vec<Point>,
    rocks: Vec<Point>,
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
            cacti: vec![],
            rocks: vec![],
            // we will update with real data later
            coin: MovingPoint{ position: Point{x:0, y:0}, direction: Direction::random(), speed: 0, wait_to_draw: 0 } ,
            difficulty,
            lives: 3,
        }
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
        self.draw_cacti()?;
        self.draw_rocks()?;
        self.draw_coin()?;
        Ok(())
    }
    fn update_game_state(&mut self) {
        if self.hero == self.coin.position {
            self.score += 1;
            self.spawn_coin();
        }
        if self.coin.wait_to_draw == 0 {
            self.coin.update_position(self.width, self.height);
        } else {
            self.coin.wait_to_draw -= 1;
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
        // banner stuff
        self.stdout
            .execute(MoveTo(3, self.height + BANNER_HEIGHT / 2))?
            .execute(Print(format!("lives remaining: {}", self.lives)))?
            .execute(MoveTo(3, self.height + 1 + BANNER_HEIGHT / 2))?
            .execute(Print(format!("difficulty: {}", self.difficulty)))?
            .execute(MoveTo(3, self.height + 2 + BANNER_HEIGHT / 2))?
            .execute(Print(format!("score: {}", self.score)))?;
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
    fn spawn_cacti_rocks(&mut self) {
        let mut cacti_rocks = Vec::new();
        let mut x: u16;
        let mut y: u16;
        let desired_length = match self.difficulty {
            Difficulty::Warmup => 0,
            Difficulty::Beginner => 4,
            Difficulty::Intermediate => 8,
            Difficulty::Advanced => 8,
            Difficulty::Expert => 12,
        };
        loop {
            x = rand::thread_rng().gen_range(1..self.width);
            y = rand::thread_rng().gen_range(1..self.height);
            if x != self.hero.x && y != self.hero.y {
                cacti_rocks.push(Point{x, y})
            }
            if cacti_rocks.len() == desired_length {
                break
            }
        }
        match self.difficulty {
            Difficulty::Warmup => {},
            Difficulty::Beginner => {
                self.rocks = cacti_rocks[0..3].to_vec();
                self.cacti = cacti_rocks[3..].to_vec();
            },
            Difficulty::Intermediate => {
                self.rocks = cacti_rocks[0..4].to_vec();
                self.cacti = cacti_rocks[4..].to_vec();
            },
            Difficulty::Advanced => {
                self.rocks = cacti_rocks[0..4].to_vec();
                self.cacti = cacti_rocks[4..].to_vec();
            },
            Difficulty::Expert => {
                self.rocks = cacti_rocks[0..6].to_vec();
                self.cacti = cacti_rocks[6..].to_vec();
            },
        }
    }
    fn draw_cacti(&mut self) -> Result<()> {
        for cactus in &self.cacti {
            self.stdout
                .execute(SetForegroundColor(Color::Green))?
                .execute(MoveTo(cactus.x, cactus.y))?
                .execute(Print("#"))?
                .flush()?;
        }        
        Ok(())
    }
    fn draw_rocks(&mut self) -> Result<()> {
        for rock in &self.rocks {
            self.stdout
                .execute(SetForegroundColor(Color::DarkGrey))?
                .execute(MoveTo(rock.x, rock.y))?
                .execute(Print("a"))?
                .flush()?;
        }        
        Ok(())
    }
    fn spawn_coin(&mut self) {
        let mut x: u16;
        let mut y: u16;
        loop {
            x = rand::thread_rng().gen_range(1..self.width);
            y = rand::thread_rng().gen_range(1..self.height);
            if x != self.hero.x && y != self.hero.y {
                // no cacti or rocks
                if self.difficulty == Difficulty::Warmup {
                    self.coin = MovingPoint{ 
                        position: Point{x, y}, 
                        direction: Direction::random(), 
                        speed: 0,
                        wait_to_draw: 3,
                    };
                    break
                }
                if !self.coin_collision(Point{x, y}) {
                    self.coin = MovingPoint{ 
                        position: Point{x, y}, 
                        direction: Direction::random(), 
                        speed: 1,
                        wait_to_draw: 3,
                    };
                    break
                }
            }
        }
    }
    fn draw_coin(&mut self) -> Result<()> {
        self.stdout
            .execute(SetForegroundColor(Color::DarkYellow))?
            .execute(MoveTo(self.coin.position.x, self.coin.position.y))?
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
                    KeyCode::Up => if self.hero.y > 1 && !self.collision_with_damage(Point{x: self.hero.x, y: self.hero.y - 1})? { 
                        self.hero.y -= 1; 
                    },
                    KeyCode::Down => if self.hero.y < self.height && !self.collision_with_damage(Point{x: self.hero.x, y: self.hero.y + 1})? { 
                        self.hero.y += 1; 
                    },
                    KeyCode::Left => if self.hero.x > 1 && !self.collision_with_damage(Point{x: self.hero.x - 1, y: self.hero.y})?{
                        self.hero.x -= 1; 
                    },
                    KeyCode::Right => if self.hero.x < self.width && !self.collision_with_damage(Point{x: self.hero.x - 1, y: self.hero.y})? { 
                        self.hero.x += 1; 
                    },
                    _ => {}
                }
            }
        }
        Ok(true)
    }
    fn collision_with_damage(&mut self, p: Point) -> Result<bool> {
        for c in &self.cacti {
            if c == &p {
                match self.difficulty {
                    Difficulty::Expert | Difficulty::Advanced | Difficulty::Intermediate => {
                        self.stdout
                            .execute(SetForegroundColor(Color::Magenta))?
                            .execute(MoveTo(3, self.height - 1 + BANNER_HEIGHT / 2))?
                            .execute(Print("ouch!"))?;
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        self.stdout
                            .execute(MoveTo(3, self.height - 1 + BANNER_HEIGHT / 2))?
                            .execute(Print("     "))?;
                        self.lives -=1;
                    },
                    _ => {},
                }
                return Ok(true);
            }
        }
        for r in &self.rocks {
            if r == &p {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn coin_collision(&mut self, p: Point) -> bool {
        for c in &self.cacti {
            if c == &p {
                return true;
            }
        }
        for r in &self.rocks {
            if r == &p {
                return true;
            }
        }
        false
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
    fn run(&mut self) -> Result<()> {
        self.prepare_screen()?;
        self.render()?;
        loop {
            if self.lives == 0 {
                eprintln!("thanks for playing!");
                break;
            }
            if !self.handle_input()? {
                break;
            }
            self.update_game_state();
            self.render()?;
        }
        Ok(())
    }
}

pub fn run_treasure_seek(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Result<()> {
    let mut game = Game::new(width, height, difficulty);
    game.spawn_cacti_rocks();
    game.spawn_coin();
    enable_raw_mode()?;
    game.run()?;
    game.cleanup()?;
    Ok(())
}