#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

use crossterm;
use crossterm::style::{Color, Colors};
use crossterm::terminal::ClearType;
use crossterm::{queue, Command};
use std::fmt::{Display};
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone, Default, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {}

impl Terminal {
    pub fn set_up() -> Result<(), Error> {
        Self::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position::default())?;
        //TODO
        //not working
        //Self::set_size(Size{length: 180, HEIGHT: 38})?;
        //Self::execute()?;

        Self::set_color(Colors {
            foreground: Option::from(Color::Green),
            background: None,
        })
    }

    pub fn terminate() -> Result<(), Error> {
        Self::disable_raw_mode()?;
        Self::clear_screen()?;
        Self::reset_color()?;
        Self::move_cursor_to(Position::default())?;
        Self::execute()?;
        Ok(())
    }

    pub fn disable_raw_mode() -> Result<(), Error> {
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn enable_raw_mode() -> Result<(), Error> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(())
    }

    pub fn set_size(size: Size) -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::SetSize(size.width, size.height))?;
        Ok(())
    }

    pub fn get_size() -> Result<Size, Error> {
        let (width, height) = crossterm::terminal::size()?;
        Ok(Size {width, height})
    }

    pub fn clear_screen() -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn print<T: Display>(string: T) -> Result<(), Error> {
        Self::execute_command(crossterm::style::Print(string))?;
        Ok(())
    }

    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
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

    pub fn hide_cursor() -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::Show)?;
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
