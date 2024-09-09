use std::collections::HashSet;

use rand::Rng;

use super::point::{MovingPoint, Point};


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let variants = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        let index = rng.gen_range(0..variants.len());
        variants[index]
    }
}

pub fn detect_collision(point: Point, points: Vec<Point>) -> bool {
    for p in points {
        if point == p {
            return true;
        }
    }
    false
}
pub fn detect_moving_collision(point: Point, points: &Vec<MovingPoint>) -> bool {
    for p in points {
        if point == p.position {
            return true;
        }
    }
    false
}