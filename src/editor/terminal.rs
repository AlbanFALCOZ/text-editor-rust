use crossterm::style::{Color, Colors};
use crossterm::terminal::ClearType;
use crossterm::{queue, Command};
use std::fmt::Display;
use std::io::{stdout, Error, Write};
use crate::editor::Location;

#[derive(Copy, Clone, Default, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// This represents the terminal.
/// Systems where usize < u16 might not be working due to conversion u16 as usize
pub struct Terminal {}

impl Terminal {
    pub fn set_up() -> Result<(), Error> {
        Self::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position::default())?;

        Self::set_color(Colors {
            foreground: Option::from(Color::Green),
            background: None,
        })
    }

    pub fn terminate() -> Result<(), Error> {
        Self::disable_raw_mode()?;
        Self::reset_color()?;
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

    /* pub fn set_size(size: Size) -> Result<(), Error> {
        Self::execute_command(crossterm::terminal::SetSize(size.width, size.height))?;
        Ok(())
    }*/

    /// Return the current size of terminal.
    /// Edge cases for system where usize < u16 :
    /// * Any coordinate 'x' will be truncated to usize if 'usize' < 'x' < 'u16'
    pub fn get_size() -> Result<Size, Error> {
        let (width_u16, height_16) = crossterm::terminal::size()?;
        #[allow(clippy::as_conversions)]
        let width = width_u16 as usize;
        #[allow(clippy::as_conversions)]
        let height = height_16 as usize;
        Ok(Size { width, height })
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

    /// Move cursor to the given Position.
    /// # Arguments
    /// * `Position` - the position the cursor will be moved to. `Position.x` and `Position.Y` will be truncated to `u16::Max` if bigger
    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        Self::execute_command(crossterm::cursor::MoveTo(
            position.x as u16,
            position.y as u16,
        ))?;
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

    /*pub fn get_cursor_position() -> std::io::Result<(u16, u16)> {
        crossterm::cursor::position()
    }

    pub fn save_cursor_position() -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::SavePosition)?;
        Ok(())
    }

    pub fn restore_cursor_position() -> Result<(), Error> {
        Self::execute_command(crossterm::cursor::RestorePosition)?;
        Ok(())
    }*/

    fn execute_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}



impl From<&Position> for Location {
    fn from(position: &Position) -> Location {
        Location {
            x: position.x,
            y: position.y,
        }
    }
}
