use anyhow::Result;
use std::io;
use crossterm::{
    cursor::MoveTo, 
    event::{KeyCode, KeyEvent}, 
    style::{Color, Print, SetForegroundColor},
    ExecutableCommand, 
    event,
};
use std::time::Duration;
use rand::Rng;
use crate::{movement, render};

use super::{
    Difficulty,
    movement::Direction,
    point::{MovingPoint, Point},
    WIDTH,
    BOARD_HEIGHT,
    BANNER_HEIGHT,
};

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
            coin: MovingPoint::default(),
            difficulty,
            lives: 3,
        }
    }
    fn render(&mut self) -> Result<()> {
        render::render_screen(&mut self.stdout, self.width, self.height, Color::DarkRed)?;
        render::render_banner(&mut self.stdout, self.height, self.lives, self.score, self.difficulty)?;
        render::draw_point(&mut self.stdout, '@', self.hero.x, self.hero.y, Color::DarkMagenta)?;
        render::draw_points(&mut self.stdout, '#', &self.cacti, Color::Green)?;
        render::draw_points(&mut self.stdout, 'a', &self.rocks, Color::DarkGrey)?;
        render::draw_point(&mut self.stdout, 'o', self.coin.position.x, self.coin.position.y, Color::DarkYellow)?;
        Ok(())
    }
    fn update_game_state(&mut self) {
        if self.hero == self.coin.position {
            self.score += 1;
            self.spawn_coin();
        }
        if self.coin.wait_to_draw == 0 {
            let points = self.get_active_points();
            self.coin.update_position(self.width, self.height, self.wait_to_draw(), points);
        } else {
            self.coin.wait_to_draw -= 1;
        }
    }
    fn get_active_points(&self) -> Vec<Point> {
        let mut points = self.cacti.clone();
        points.extend(self.rocks.clone());
        points
    }
    fn spawn_cacti_rocks(&mut self) {
        let mut cacti_rocks = Vec::new();
        let mut x: u16;
        let mut y: u16;
        let desired_length = match self.difficulty {
            Difficulty::Warmup => 0,
            Difficulty::Beginner => 4,
            Difficulty::Intermediate | Difficulty::Advanced => 8,
            Difficulty::Expert => 12,
        };
        loop {
            x = rand::thread_rng().gen_range(1..self.width);
            y = rand::thread_rng().gen_range(1..self.height);
            if x != self.hero.x && y != self.hero.y {
                cacti_rocks.push(Point{x, y});
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
            Difficulty::Intermediate | Difficulty::Advanced => {
                self.rocks = cacti_rocks[0..4].to_vec();
                self.cacti = cacti_rocks[4..].to_vec();
            },
            Difficulty::Expert => {
                self.rocks = cacti_rocks[0..6].to_vec();
                self.cacti = cacti_rocks[6..].to_vec();
            },
        }
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
                        wait_to_draw: 0,
                    };
                    break
                }
                let mut points = self.cacti.clone();
                points.extend(self.rocks.clone());
                if !movement::detect_collision(Point{x, y}, points) {
                    self.coin = MovingPoint{ 
                        position: Point{x, y}, 
                        direction: Direction::random(), 
                        speed: 1,
                        wait_to_draw: self.wait_to_draw()
                    };
                    break
                }
            }
        }
    }
    fn wait_to_draw(&self) -> u8 {
        match self.difficulty {
            Difficulty::Warmup => 0,
            Difficulty::Beginner => 5,
            Difficulty::Intermediate => 3,
            Difficulty::Advanced => 1,
            Difficulty::Expert => 1,
        }
    }
    fn handle_input(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(KeyEvent { code, .. }) = event::read()?
            {
                match code {
                    KeyCode::Esc | KeyCode::Char('Q' | 'q') => return Ok(false),
                    KeyCode::Up => if self.hero.y > 1 && !self.collision_with_damage(Point{x: self.hero.x, y: self.hero.y - 1})? { 
                        self.hero.y -= 1; 
                    },
                    KeyCode::Down => if self.hero.y < self.height && !self.collision_with_damage(Point{x: self.hero.x, y: self.hero.y + 1})? { 
                        self.hero.y += 1; 
                    },
                    KeyCode::Left => if self.hero.x > 1 && !self.collision_with_damage(Point{x: self.hero.x - 1, y: self.hero.y})?{
                        self.hero.x -= 1; 
                    },
                    KeyCode::Right => if self.hero.x < self.width && !self.collision_with_damage(Point{x: self.hero.x + 1, y: self.hero.y})? { 
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
                        let mut blue = true;
                        for i in (3..self.width-4).step_by(2) {
                            let color = if blue { Color::DarkBlue } else { Color::DarkGreen };
                            blue = !blue;
                            self.damage_animation(i, color)?;
                        }
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
    fn damage_animation(&mut self, width: u16, color: Color) -> Result<()> {
        self.stdout
            .execute(SetForegroundColor(Color::DarkRed))?
            .execute(MoveTo(width, self.height - 1 + BANNER_HEIGHT / 2))?
            .execute(Print("ouch!"))?;
        render::render_screen(&mut self.stdout, self.width, self.height, color)?;
        render::render_banner(&mut self.stdout, self.height, self.lives, self.score, self.difficulty)?;

        std::thread::sleep(std::time::Duration::from_millis(200));
        self.stdout
            .execute(MoveTo(width, self.height - 1 + BANNER_HEIGHT / 2))?
            .execute(Print("     "))?;
        Ok(())
    }
    fn run(&mut self) -> Result<()> {
        render::prepare_screen(&mut self.stdout, self.width, self.height)?;
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
            // limit the frame rate
            std::thread::sleep(Duration::from_millis(80));
        }
        Ok(())
    }
}

pub fn run_treasure_seek(width: Option<u16>, height: Option<u16>, difficulty: Difficulty) -> Result<()> {
    let mut game = Game::new(width, height, difficulty);
    if difficulty != Difficulty::Warmup {
        game.spawn_cacti_rocks();
    }
    game.spawn_coin();
    game.run()?;
    let (x, y) = game.normal_terminal;
    render::cleanup(&mut game.stdout, x, y)?;
    Ok(())
}