#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

use crossterm;
use crossterm::style::{Color, Colors};
use crossterm::terminal::ClearType;
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}


impl Position {
    pub fn new(x: u16, y: u16) -> Position {
        Position { x, y }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub length: u16,
    pub height: u16,
}

pub struct Terminal {}

impl Terminal {
    pub fn disable_raw_mode() -> Result<(), Error> {
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn enable_raw_mode() -> Result<(), Error> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn set_size(size: Size) -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::SetSize(size.length, size.height))?;
        Ok(())
    }

    pub fn get_size() -> Result<Size, Error> {
        let result = crossterm::terminal::size()?;
        let size: Size = Size {
            length: result.0,
            height: result.1,
        };
        Ok(size)
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(position: &Position) -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::MoveTo(position.x, position.y))?;
        Ok(())
    }

    pub fn set_color(colors: Colors) -> Result<(), Error> {
        Self::execute_command(crossterm::style::SetColors(colors))?;
        Ok(())
    }

    pub fn reset_color() -> Result<(), Error> {
        Self::execute_command(crossterm::style::ResetColor)?;
        Ok(())
    }

    fn execute_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}
