use rand::{self, Rng};

pub fn color() -> (u8, u8, u8) {
    let r = rand::thread_rng().gen_range(0..=255);
    let g = rand::thread_rng().gen_range(0..=255);
    let b = rand::thread_rng().gen_range(0..=255);
    (r, g, b)
}