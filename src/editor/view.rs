use crate::editor::terminal::{Position, Size, Terminal};
use crate::editor::view::buffer::Buffer;
use std::io::Error;

mod buffer;

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
}

impl View {
    pub fn render(&mut self) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = self.size;
        if width == 0 || height == 0 {
            return Ok(());
        }
        //We allow this because it doesn't matter is the version is exactly at 1/3 of the screen
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row) {
                let truncated_line = if line.len() >= width {
                    &line[0..width]
                } else {
                    line
                };
                Self::render_line(current_row, truncated_line)?;
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                Terminal::set_color_to_green()?;
                Self::render_line(current_row, "~")?;
            }
        }
        Terminal::reset_color()?;
        self.needs_redraw = false;
        Ok(())
    }

    fn render_line(at: usize, line_text: &str) -> Result<(), Error> {
        Terminal::move_cursor_to(Position { row: at, col: 0 })?;
        Terminal::clear_line()?;
        Terminal::print(line_text)?;
        Ok(())
    }

    fn build_welcome_message(width: usize) -> String {
        if width == 0 {
            return " ".to_string();
        }
        let mut version = "Rust terminal version 0.5".to_string();
        let len = version.len();
        if width <= len {
            return "~".to_string();
        }
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;
        let spaces = " ".repeat(padding);
        version = format!("~{spaces}{version}");
        version.truncate(width);
        version
    }

    pub fn resize(&mut self, to_size: Size) {
        self.size = to_size;
        self.needs_redraw = true;
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap(),
        }
    }
}
