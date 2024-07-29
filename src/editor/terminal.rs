#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

use std::io::{Error, stdout, Write};
use crossterm;
use crossterm::{queue, Command};
use crossterm::terminal::ClearType;

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

    pub fn clear_screen() -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn move_cursor_to(x: u16, y: u16) -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::MoveTo(x,y))?;
        Ok(())
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    /*pub fn write() -> Result<(), Error> {
        Self::execute_command(write!())
    }*/

    fn execute_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}