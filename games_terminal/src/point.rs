use super::movement::Direction;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovingPoint {
    pub position: Point,
    pub direction: Direction,
    pub speed: u16,
    pub wait_to_draw: u8, // todo prob a better way to do this, this field and the above seem redundant
}

impl MovingPoint {
    pub fn default() -> Self {
        MovingPoint{ position: Point{x:0, y:0}, direction: Direction::random(), speed: 0, wait_to_draw: 0 }
    }
    pub fn update_position(&mut self, board_width: u16, board_height: u16, wait_to_draw: u8, points: Vec<Point>) {
        self.wait_to_draw = wait_to_draw;
        match self.direction {
            Direction::Up => {
                if self.position.y > 1 && !Self::collision(Point{x: self.position.x, y: self.position.y - 1}, points) {
                    self.position.y = self.position.y.saturating_sub(self.speed);
                } else {
                    self.direction = Direction::Down;
                }
            }
            Direction::Down => {
                if self.position.y < board_height && !Self::collision(Point{x: self.position.x, y: self.position.y + 1}, points) {
                    self.position.y = self.position.y.saturating_add(self.speed);
                } else {
                    self.direction = Direction::Up;
                }
            }
            Direction::Left => {
                if self.position.x > 1 && !Self::collision(Point{x: self.position.x - 1, y: self.position.y}, points) {
                    self.position.x = self.position.x.saturating_sub(self.speed);
                } else {
                    self.direction = Direction::Right;
                }
            }
            Direction::Right => {
                if self.position.x < board_width && !Self::collision(Point{x: self.position.x + 1 , y: self.position.y}, points) {
                    self.position.x = self.position.x.saturating_add(self.speed);
                } else {
                    self.direction = Direction::Left;
                }
            }
        }
    }
    fn collision(point: Point, points: Vec<Point>) -> bool {
        for p in points {
            if point == p {
                return true;
            }
        }
        false
    }
}