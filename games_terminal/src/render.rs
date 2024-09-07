use anyhow::Result;
use std::io::{Stdout, Write};
use crossterm::{
    cursor::{Hide, MoveTo, Show}, 
    style::{Color, Print, ResetColor, SetForegroundColor}, 
    terminal::{self, Clear, ClearType, SetSize, enable_raw_mode}, 
    ExecutableCommand, 
};

use crate::{point::Point, Difficulty};

use super::BANNER_HEIGHT;

pub fn prepare_screen(stdout: &mut Stdout, width: u16, height: u16) -> Result<()> {
    enable_raw_mode()?;
    stdout
        .execute(SetSize(width, height))?
        .execute(Clear(ClearType::All))?
        .execute(Hide)?
        .flush()?;
    Ok(())
}

pub fn render_screen(stdout: &mut Stdout, width: u16, height: u16, color: Color) -> Result<()> {
    stdout.execute(SetForegroundColor(color))?;
    // columns
    for y in 0..height + 1 + BANNER_HEIGHT {
        stdout
            .execute(MoveTo(0, y))?
            .execute(Print("X"))?
            .execute(MoveTo(width + 1, y))?
            .execute(Print("X"))?;
    }
    // rows
    for x in 0..width + 2 {
        stdout
            .execute(MoveTo(x, 0))?
            .execute(Print("X"))?
            .execute(MoveTo(x, height + 1))?
            .execute(Print("X"))?
            .execute(MoveTo(x, height + BANNER_HEIGHT))?
            .execute(Print("X"))?;
    }
    // corners
    stdout
        .execute(MoveTo(0, 0))?
        .execute(Print("X"))?
        .execute(MoveTo(width + 1, height + 1))?
        .execute(Print("X"))?
        .execute(MoveTo(width + 1, 0))?
        .execute(Print("X"))?
        .execute(MoveTo(0, height + 1))?
        .execute(Print("X"))?;
    // empty the background
    stdout.execute(ResetColor).unwrap();
    for y in 1..=height {
        for x in 1..=width {
            stdout
                .execute(MoveTo(x, y)).unwrap()
                .execute(Print(" ")).unwrap();
        }
    }
    stdout.flush()?;
    Ok(())
}

pub fn render_banner(stdout: &mut Stdout, height: u16, lives: u8, score: u16, difficulty: Difficulty) -> Result<()> {
    stdout
        .execute(MoveTo(3, height + BANNER_HEIGHT / 2))?
        .execute(Print(format!("lives remaining: {}", lives)))?
        .execute(MoveTo(3, height + 1 + BANNER_HEIGHT / 2))?
        .execute(Print(format!("difficulty: {}", difficulty)))?
        .execute(MoveTo(3, height + 2 + BANNER_HEIGHT / 2))?
        .execute(Print(format!("score: {}", score)))?
        .flush()?;
    Ok(())
}

pub fn draw_point(stdout: &mut Stdout, character: char, width: u16, height: u16, color: Color) -> Result<()> {
    stdout
        .execute(SetForegroundColor(color))?
        .execute(MoveTo(width, height))?
        .execute(Print(character))?
        .flush()?;
    Ok(())
}

pub fn draw_points(stdout: &mut Stdout, character: char, points: &Vec<Point>, color: Color) -> Result<()> {
    for point in points {
        stdout
            .execute(SetForegroundColor(color))?
            .execute(MoveTo(point.x, point.y))?
            .execute(Print(character))?
            .flush()?;
    }
    Ok(())
}

pub fn cleanup(stdout: &mut Stdout, orig_x: u16, orig_y: u16) -> Result<()> {
    stdout
        .execute(terminal::SetSize(orig_x, orig_y))?
        .execute(Clear(ClearType::All))?
        .execute(Show)?
        .execute(ResetColor)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
