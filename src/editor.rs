mod terminal;

use crate::editor::terminal::{Position, Size, Terminal};
use crossterm::event::{
    read, Event, Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use std::io::Error;

#[derive(Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

/// This represents our Editor
/// It manages all the events and printing that happen in the terminal
/// It relies on our Terminal and the functions of the crossterm crate to work
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_location: Location,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::set_up().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = *event
        {
            match code {
                KeyCode::Char('q' | 'Q') if modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up |
                KeyCode::Down |
                KeyCode::Right |
                KeyCode::Left |
                KeyCode::End |
                KeyCode::Home |
                KeyCode::PageUp|
                KeyCode::PageDown => {
                    self.move_caret(code)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn move_caret(&mut self, key_code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.cursor_location;
        let Size { width, height } = Terminal::get_size()?;
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = y.saturating_add(1);
            }
            KeyCode::Right => {
                x = x.saturating_add(1);
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width;
            }
            KeyCode::PageDown => {
                y = 0;
            }
            KeyCode::PageUp => {
                y = height;
            }
            _ => (),
        };
        self.cursor_location = Location{x,y};
        Ok(())

    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::move_cursor_to(Position::default())?;
            Terminal::reset_color()?;
            Terminal::print("Goodbye ! ~~")?;
        } else {
            Terminal::move_cursor_to(Position::default())?;
            Self::print_rows()?;
            Terminal::move_cursor_to(Position::from(&self.cursor_location))?;
        };
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    /// Print the lines.
    /// It prints tilde '~' at the beginning of each line.
    /// Print the terminal version at 1/3 of the screen.
    /// The clippy warning is disabled because it doesn't matter if the version is exactly at 1/3 of our screen.
    fn print_rows() -> Result<(), Error> {
        let Size { height, .. } = Terminal::get_size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
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
        Terminal::print(version)?;
        Ok(())
    }
}

impl From<&Location> for Position {
    fn from(location: &Location) -> Position {
        Position {
            x: location.x,
            y: location.y,
        }
    }
}
