use crossterm::style::{Color, Colors};
use crossterm::terminal::ClearType;
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

#[derive(Copy, Clone, Default, Debug)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub fn saturating_sub(self, other: Self) -> Self {
        Self {
            row: self.row.saturating_sub(other.row),
            col: self.col.saturating_sub(other.col),
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

/// This represents the terminal.
/// Systems where usize < u16 might not be working due to conversion u16 as usize
pub struct Terminal {}

impl Terminal {
    pub fn set_up() -> Result<(), Error> {
        Self::enter_alternate_screen()?;
        Self::enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(Position::default())
    }

    pub fn terminate() -> Result<(), Error> {
        Self::show_cursor()?;
        Self::reset_color()?;
        Self::leave_alternate_screen()?;
        Self::execute()?;
        Self::disable_raw_mode()?;
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

    pub fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(crossterm::terminal::EnterAlternateScreen)?;
        Ok(())
    }

    pub fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(crossterm::terminal::LeaveAlternateScreen)?;
        Ok(())
    }

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
        Self::queue_command(crossterm::terminal::Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(crossterm::terminal::Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(crossterm::style::Print(string))?;
        Ok(())
    }

    pub fn print_row(at_row: usize, line: &str) -> Result<(), Error> {
        Self::move_cursor_to(Position {
            row: at_row,
            col: 0,
        })?;
        Self::clear_line()?;
        Self::print(line)?;
        Ok(())
    }

    pub fn set_color_to_green() -> Result<(), Error> {
        Self::set_color(Colors {
            foreground: Option::from(Color::Green),
            background: None,
        })?;
        Ok(())
    }

    /// Move cursor to the given Position.
    /// # Arguments
    /// * `Position` - the position the cursor will be moved to. `Position.x` and `Position.Y` will be truncated to `u16::Max` if bigger
    pub fn move_cursor_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        Self::queue_command(crossterm::cursor::MoveTo(
            position.col as u16,
            position.row as u16,
        ))?;
        Ok(())
    }

    pub fn set_color(colors: Colors) -> Result<(), Error> {
        Self::queue_command(crossterm::style::SetColors(colors))?;
        Ok(())
    }

    pub fn reset_color() -> Result<(), Error> {
        Self::queue_command(crossterm::style::ResetColor)?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), Error> {
        Self::queue_command(crossterm::cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), Error> {
        Self::queue_command(crossterm::cursor::Show)?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }

    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }
}
