use crate::editor::editorcommand::{Direction, EditorCommand};
use crate::editor::terminal::{Position, Size, Terminal};
use crate::editor::view::buffer::Buffer;
use crate::editor::view::line::Line;
use std::cmp::{min, PartialEq};

mod buffer;
mod line;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Location {
    pub grapheme_index: usize,
    pub line_index: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
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
        let top = self.scroll_offset.row;

        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get_visible_graphemes(left..right));
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
            EditorCommand::Insert('\t') => {
                for _ in 0..4 {
                    self.insert_char(' ');
                }
            }
            EditorCommand::Insert(char) => {
                self.insert_char(char);
            }
            EditorCommand::Delete => {
                self.delete();
            }
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Quit => {}
        }
    }

    fn scroll_vertically(&mut self, to: usize) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };
        self.needs_redraw = self.needs_redraw || offset_changed;
    }
    fn scroll_horizontally(&mut self, to: usize) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };
        self.needs_redraw = self.needs_redraw || offset_changed;
    }
    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    #[must_use]
    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_index;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            line.width_until(self.text_location.grapheme_index)
        });
        Position { col, row }
    }

    fn move_text_location(&mut self, direction: &Direction) {
        let Size { height, .. } = self.size;
        match direction {
            Direction::Up => self.move_up(1),
            Direction::Down => self.move_down(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_start_of_line(),
            Direction::End => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }
    fn move_up(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }
    fn move_down(&mut self, step: usize) {
        self.text_location.line_index = self.text_location.line_index.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_right(&mut self) {
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        if self.text_location.grapheme_index < line_width {
            self.text_location.grapheme_index += 1;
        } else if self.text_location.line_index < self.buffer.height().saturating_sub(1) {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }

    // clippy::arithmetic_side_effects: This function performs arithmetic calculations
    // after explicitly checking that the target value will be within bounds.
    #[allow(clippy::arithmetic_side_effects)]
    fn move_left(&mut self) {
        if self.text_location.grapheme_index > 0 {
            self.text_location.grapheme_index -= 1;
        } else if self.text_location.line_index > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }

    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_index = 0;
    }

    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_index = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, |line| {
                min(line.grapheme_count(), self.text_location.grapheme_index)
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.text_location.line_index = min(
            self.text_location.line_index,
            self.buffer.height().saturating_sub(1),
        );
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

    fn insert_char(&mut self, character: char) {
        let old_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        self.buffer.insert_char(character, &self.text_location);
        let new_len = self
            .buffer
            .lines
            .get(self.text_location.line_index)
            .map_or(0, Line::grapheme_count);
        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            self.move_right();
        }
        self.needs_redraw = true;
    }

    fn backspace(&mut self) {
        if self.text_location.line_index == 0 && self.text_location.grapheme_index == 0 {
            return;
        }
        self.move_left();
        self.delete();
    }

    fn delete(&mut self) {
        if self.end_of_file() {
            return;
        }
        if self.start_of_view() {
            self.scroll_offset.row = self.scroll_offset.row.saturating_sub(1);
        }
        self.buffer.delete(&self.text_location);
        self.scroll_text_location_into_view();
        self.needs_redraw = true;
    }

    fn end_of_file(&self) -> bool {
        self.text_location.line_index == self.buffer.height().saturating_sub(1)
            && self.text_location.grapheme_index
                == self
                    .buffer
                    .lines
                    .get(self.text_location.line_index)
                    .unwrap()
                    .grapheme_count()
    }

    fn start_of_view(&self) -> bool {
        self.text_location == Location::default()
    }
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    fn set_up(file_name: &str) -> View {
        let mut view = View::default();
        view.load(file_name);
        view
    }

    #[test]
    fn test_size() {
        let size: Size = Size {
            width: 100,
            height: 10,
        };
        let mut view: View = View::default();
        view.resize(size);
        assert_eq!(view.size, size);
    }

    #[test]
    fn test_offset_change_scroll() {
        let terminal_size: (u16, u16) = crossterm::terminal::size().unwrap();
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(terminal_size.1.into());
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.row, 1);
    }

    #[test]
    fn test_scroll_to_end() {
        let mut view: View = set_up(".\\src\\editor.rs");
        let mut height = view.buffer.height();
        while height > 0 {
            view.move_down(1);
            height = height.saturating_sub(1);
        }
        assert_eq!(
            view.buffer.height().saturating_sub(1),
            view.text_location.line_index
        );
    }

    #[test]
    fn test_must_scroll() {
        let mut view: View = View::default();
        assert!(view.needs_redraw);
        view.needs_redraw = false;
        view.load(".\\src\\editor.rs");
        view.handle_command(EditorCommand::Move(Direction::PageDown));
        assert!(view.needs_redraw);
    }

    #[test]
    fn test_scroll_down_end() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert_eq!(view.text_location.line_index, 0);
        view.move_down(view.buffer.height().saturating_sub(1));
        let line_index = view.text_location.line_index;
        view.move_down(1);
        assert_eq!(line_index, view.text_location.line_index);
    }

    #[test]
    fn test_scroll_up_beginning() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert_eq!(view.text_location.line_index, 0);
        view.move_up(1);
        assert_eq!(view.text_location.line_index, 0);
    }

    #[test]
    fn test_screen_scroll_up() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        view.move_down(1);
        let line_width = view
            .buffer
            .lines
            .get(view.text_location.line_index)
            .unwrap()
            .grapheme_count();
        view.move_to_end_of_line();
        view.scroll_text_location_into_view();
        assert_eq!(view.text_location.grapheme_index, line_width);
        view.move_up(1);
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.col, view.text_location.grapheme_index);
        assert_eq!(
            view.scroll_offset.col,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
    }
    #[test]
    fn test_screen_scroll_down() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        view.move_down(1);
        let line_width = view
            .buffer
            .lines
            .get(view.text_location.line_index)
            .unwrap()
            .grapheme_count();
        view.move_to_end_of_line();
        view.scroll_text_location_into_view();
        assert_eq!(view.text_location.grapheme_index, line_width);
        view.move_down(1);
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.col, view.text_location.grapheme_index);
        assert_eq!(
            view.scroll_offset.col,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
    }

    #[test]
    fn test_page_down() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(1);
        view.handle_command(EditorCommand::Move(Direction::PageDown));
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.row, 1);
    }

    #[test]
    fn test_page_up() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.handle_command(EditorCommand::Move(Direction::PageDown));
        view.move_down(1);
        view.scroll_text_location_into_view();
        view.handle_command(EditorCommand::Move(Direction::PageUp));
        assert_eq!(view.scroll_offset.row, 1);
    }

    #[test]
    fn test_move_right() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(2);
        view.move_right();
        assert_eq!(view.text_location.grapheme_index, 1);
        view.move_to_end_of_line();
        view.scroll_text_location_into_view();
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        let line_index: usize = view.text_location.line_index;
        view.move_right();
        view.scroll_text_location_into_view();
        assert_eq!(line_index.saturating_add(1), view.text_location.line_index);
        assert_eq!(view.text_location.grapheme_index, 0);
    }

    #[test]
    fn test_move_right_at_bottom() {
        let mut view = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        while view.text_location.line_index < view.buffer.height().saturating_sub(1) {
            view.handle_command(EditorCommand::Move(Direction::PageDown));
        }
        view.scroll_text_location_into_view();
        view.move_to_end_of_line();
        let grapheme_index = view.text_location.grapheme_index;
        view.move_right();
        assert_eq!(grapheme_index, view.text_location.grapheme_index);
    }

    #[test]
    fn test_move_left() {
        let mut view = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(3);
        view.move_left();
        assert_eq!(view.text_location.line_index, 2);
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        view.move_to_start_of_line();
        view.move_left();
        assert_eq!(view.text_location.line_index, 1);
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
    }

    #[test]
    fn test_move_left_start() {
        let mut view = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_right();
        assert_eq!(view.text_location.grapheme_index, 1);
        view.move_left();
        assert_eq!(view.text_location.grapheme_index, 0);
        view.move_left();
        assert_eq!(view.text_location.grapheme_index, 0);
        assert_eq!(view.text_location.line_index, 0);
    }

    #[test]
    fn test_scroll_offset_move_left() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        view.move_down(1);
        view.move_left();
        view.scroll_text_location_into_view();
        assert_eq!(
            view.scroll_offset.col.saturating_sub(1),
            view.buffer
                .lines
                .first()
                .unwrap()
                .grapheme_count()
                .saturating_sub(Terminal::get_size().unwrap().width)
        );
    }

    #[test]
    fn test_scroll_offset_move_right() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        view.move_down(1);
        view.move_to_end_of_line();
        view.scroll_text_location_into_view();
        view.move_right();
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.col, 0);
    }

    #[test]
    fn test_scroll_offset_move_right_end() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.handle_command(EditorCommand::Move(Direction::PageDown));
        assert_eq!(view.scroll_offset.row, 0);
        view.move_to_end_of_line();
        view.move_right();
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.row, 1);
    }

    #[test]
    fn test_scroll_offset_move_left_start() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.resize(Terminal::get_size().unwrap());
        view.handle_command(EditorCommand::Move(Direction::PageDown));
        view.move_down(1);
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.row, 1);
        view.handle_command(EditorCommand::Move(Direction::PageUp));
        assert_eq!(view.scroll_offset.row, 1);
        view.move_left();
        view.scroll_text_location_into_view();
        assert_eq!(view.scroll_offset.row, 0);
        assert_eq!(
            view.scroll_offset.col.saturating_sub(1),
            view.buffer
                .lines
                .first()
                .unwrap()
                .grapheme_count()
                .saturating_sub(Terminal::get_size().unwrap().width)
        );
    }

    #[test]
    fn test_move_up_position() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(4);
        view.move_to_end_of_line();
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        let line_width = view.text_location.grapheme_index;
        view.move_up(1);
        assert_eq!(line_width, view.text_location.grapheme_index);
        let line_above_width = view
            .buffer
            .lines
            .get(view.text_location.line_index.saturating_sub(1))
            .unwrap()
            .grapheme_count();
        view.move_up(1);
        assert_eq!(line_above_width, view.text_location.grapheme_index);
    }

    #[test]
    fn test_move_down_position() {
        let mut view: View = set_up(".\\src\\editor.rs");
        assert!(!view.buffer.is_empty());
        view.move_down(5);
        view.move_to_end_of_line();
        assert_eq!(
            view.text_location.grapheme_index,
            view.buffer
                .lines
                .get(view.text_location.line_index)
                .unwrap()
                .grapheme_count()
        );
        let line_width = view.text_location.grapheme_index;
        view.move_down(1);
        assert_eq!(line_width, view.text_location.grapheme_index);
        let line_under_width = view
            .buffer
            .lines
            .get(view.text_location.line_index.saturating_add(1))
            .unwrap()
            .grapheme_count();
        view.move_down(1);
        assert_eq!(line_under_width, view.text_location.grapheme_index);
    }

    #[test]
    fn test_delete_character_at_end_line() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        let current_line_width = view.buffer.lines.first().unwrap().grapheme_count();
        let next_line_width = view.buffer.lines.get(1).unwrap().grapheme_count();
        let number_lines = view.buffer.height();
        view.move_to_end_of_line();
        view.handle_command(EditorCommand::Delete);
        assert_eq!(
            view.buffer.lines.first().unwrap().grapheme_count(),
            current_line_width.saturating_add(next_line_width)
        );
        assert_eq!(view.buffer.height(), number_lines.saturating_sub(1));
    }

    #[test]
    fn test_backspace_at_start_line() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        let current_line_width = view.buffer.lines.first().unwrap().grapheme_count();
        let next_line_width = view.buffer.lines.get(1).unwrap().grapheme_count();
        let number_lines = view.buffer.height();
        view.move_down(1);
        view.backspace();
        assert_eq!(
            view.buffer.lines.first().unwrap().grapheme_count(),
            current_line_width.saturating_add(next_line_width)
        );
        assert_eq!(view.buffer.height(), number_lines.saturating_sub(1));
    }

    #[test]
    fn test_backspace_scroll() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        for _ in 0..2 {
            view.handle_command(EditorCommand::Move(Direction::PageDown));
            view.scroll_text_location_into_view();
        }
        while view.text_location.line_index > view.scroll_offset.row {
            view.move_up(1);
        }
        assert!(view.scroll_offset.row > 0);
        view.move_to_start_of_line();
        let line_index: usize = view.text_location.line_index;
        let scroll_offset_row = view.scroll_offset.row;
        view.backspace();
        assert_eq!(line_index.saturating_sub(1), view.text_location.line_index);
        assert_eq!(scroll_offset_row.saturating_sub(1), view.scroll_offset.row);
    }

    #[test]
    fn test_line_width_tab() {
        let mut view: View = set_up(".\\text-test\\test-3.txt");
        assert!(!view.buffer.is_empty());
        let line_width = view.buffer.lines.first().unwrap().grapheme_count();
        view.handle_command(EditorCommand::Insert('\t'));
        assert_eq!(
            line_width.saturating_add(4),
            view.buffer.lines.first().unwrap().grapheme_count()
        );
    }
}
