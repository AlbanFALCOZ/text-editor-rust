use std::io::Error;
use crate::editor::terminal::{Size, Terminal};
use crate::editor::view::buffer::Buffer;

mod buffer;

#[derive(Default)]
pub struct View {
    buffer: Buffer
}


impl View {

    /// Print the lines.
    /// It prints tilde '~' at the beginning of each line.
    /// Print the terminal version at 1/3 of the screen.
    /// The clippy warning is disabled because it doesn't matter if the version is exactly at 1/3 of our screen.
    pub fn render(&self) -> Result<(), Error> {
        let Size { height, .. } = Terminal::get_size()?;
        Terminal::clear_line()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            if let Some(line) = self.buffer.lines.get(current_row){
                Terminal::print(line)?;
                Terminal::print("\r\n")?;
                continue;
            }
            #[allow(clippy::integer_division)]
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            if current_row.saturating_add(1) < height {
                Terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }

    /// Print the welcome message.
    /// The clippy warning is disabled because it doesn't matter if the version is not exactly centred.
    fn draw_welcome_message() -> Result<(), Error> {
        let width = Terminal::get_size()?.width;
        let mut version = "Rust terminal version 0.5".to_string();
        let len = version.len();
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;
        let spaces = " ".repeat(padding);
        version = format!("~{spaces}{version}");
        version.truncate(width);
        Terminal::print(&version)?;
        Ok(())
    }
}