use std::cmp::min;
use crate::editor::editorcommand::{Direction, EditorCommand};
use crate::editor::terminal::{Position, Size, Terminal};
use crate::editor::view::buffer::Buffer;
use crate::editor::view::location::Location;

mod buffer;
mod line;
mod location;

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

impl View {
    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }
        let Size { width, height } = self.size;
        if width == 0 || height == 0 {
            return;
        }
        //We allow this because it doesn't matter is the version is exactly at 1/3 of the screen
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.y;
        let left = self.scroll_offset.x;
        let right = self.scroll_offset.x.saturating_add(width);
        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                Self::render_line(current_row, &line.get(left..right));
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                let _ = Terminal::set_color_to_green();
                Self::render_line(current_row, "~");
            }
        }
        let _ = Terminal::reset_color();
        self.needs_redraw = false;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
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

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => {
                self.move_text_location(&direction);
            }
            EditorCommand::Quit => {}
        }
    }

    pub fn move_text_location(&mut self, direction: &Direction) {
        let Location { mut x, mut y } = self.location;
        let Location {
            x: mut x_scroll,
            y: mut y_scroll,
        } = self.scroll_offset;
        let Size { width, height } = self.size;
        let number_of_lines = self.buffer.lines.len();
        match direction {
            Direction::PageUp => {
                if y.saturating_add(y_scroll) < height {
                    y = 0;
                    y_scroll = 0;
                } else {
                    y = y.saturating_sub(1);
                    y_scroll = y_scroll.saturating_sub(height);
                }
                self.needs_redraw = true;
            }
            Direction::PageDown => {
                if y.saturating_add(y_scroll).saturating_add(height) < number_of_lines {
                        y = min(y.saturating_add(1),width.saturating_sub(1));
                        y_scroll = min(y_scroll.saturating_add(height),number_of_lines.saturating_sub(height));
                        self.needs_redraw = true;
                }
                else {
                    y = height.saturating_sub(1);
                }
            }
            Direction::Home => {
                if x_scroll > 0 {
                    self.needs_redraw = true;
                }
                x = 0;
                x_scroll = 0;
            }
            Direction::End => {
                let current_line_len = self.buffer.lines.get(y).unwrap().len();
                if current_line_len.saturating_sub(x_scroll) < width {
                    x = current_line_len.saturating_sub(x_scroll);
                } else {
                    x = width.saturating_sub(1);
                    x_scroll = current_line_len.saturating_sub(width).saturating_add(1);
                    self.needs_redraw = true;
                }
            }
            Direction::Up => {
                if y > 0 {
                    y = y.saturating_sub(1);
                } else {
                    y_scroll = y_scroll.saturating_sub(1);
                    self.needs_redraw = true;
                }
            }
            Direction::Left => {
                //If the cursor is at the beginning of line and the screen hasn't moved
                if x == 0 && x_scroll == 0 {
                    //If we're at the beginning of the file, we do nothing
                    if y.saturating_add(y_scroll) != 0 {
                        if y > 0 {
                            y -= 1;
                        } else {
                            y_scroll = y_scroll.saturating_sub(1);
                        }
                        let current_line_len = self
                            .buffer
                            .lines
                            .get(y.saturating_add(y_scroll))
                            .unwrap()
                            .len();
                        if current_line_len < width {
                            x = current_line_len;
                        } else {
                            x = width.saturating_sub(1);
                            x_scroll = current_line_len.saturating_sub(width).saturating_add(1);
                        }
                        self.needs_redraw = true;
                    }
                }
                //If the cursor is not at the beginning of line
                else if x > 0 {
                    x = x.saturating_sub(1);
                }
                //If the cursor is at the beginning of line and the screen has moved
                else if x_scroll > 0 {
                    x_scroll = x_scroll.saturating_sub(1);
                    self.needs_redraw = true;
                }
            }
            Direction::Right => {
                if x >= width.saturating_sub(1) {
                    x_scroll = x_scroll.saturating_add(1);
                    self.needs_redraw = true;
                } else {
                    x = x.saturating_add(1);
                }
            }
            Direction::Down => 'down: {
                //We let the user go down one line
                if y.saturating_add(y_scroll) >= number_of_lines {
                    break 'down;
                }
                if y >= height.saturating_sub(1) {
                    y_scroll = y_scroll.saturating_add(1);
                    self.needs_redraw = true;
                } else {
                    y = y.saturating_add(1);
                }
            }
        }
        self.location = Location { x, y };
        self.scroll_offset = Location {
            x: x_scroll,
            y: y_scroll,
        };
        //self.scroll_location_into_view();
    }

    /*fn scroll_location_into_view(&mut self) {
        let Location { x, y } = self.location;
        let Size { width, height } = self.size;
        let mut offset_changed = false;

        if y < self.scroll_offset.y {
            self.scroll_offset.y = y;
            offset_changed = true;
        } else if y >= self.scroll_offset.y.saturating_add(height) {
            self.scroll_offset.y = self
                .scroll_offset
                .y
                .saturating_sub(height)
                .saturating_add(1);
            offset_changed = true;
        }

        if x < self.scroll_offset.x {
            self.scroll_offset.x = x;
            offset_changed = true;
        } else if x >= self.scroll_offset.x.saturating_add(width) {
            self.scroll_offset.x = self.scroll_offset.x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        self.needs_redraw = offset_changed;
    }*/

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

    pub fn get_position(&self) -> Position {
        self.location.subtract(&Location::default()).into()
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap(),
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }
}
