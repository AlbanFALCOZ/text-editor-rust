mod terminal;

use crate::editor::terminal::{Position, Size, Terminal};
use crossterm::event::{read, Event, Event::Key, KeyCode::Char, KeyModifiers};
use std::io::Error;

/// This represents our Editor
/// It manages all the events and printing that happen in the terminal
/// It relies on our Terminal and the functions of the crossterm crate to work
pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { should_quit: false }
    }

    pub fn run(&mut self) {
        Terminal::set_up().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }

    pub fn evaluate_event(&mut self, event: &Event) {
        match event {
            Key(event) => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == Char('q') {
                    self.should_quit = true;
                }
                /*Terminal::move_cursor_to(Position::default())?;
                let string: &str = stringify!(event);
                Terminal::print(string)?;*/
            }
            Event::Resize(..) => {
                //Terminal::clear_screen()?;
                //Terminal::move_cursor_to(Position::default())?;
                //Self::print_rows()?;
                //Self::draw_welcome_message()?;
            }
            _ => (),
        }
    }

    pub fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::reset_color()?;
            Terminal::print("Goodbye ! ~~")?;
        } else {
            Self::print_rows()?;
            Terminal::move_cursor_to(Position::default())?;
        };
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    /// Print the lines
    /// It prints tilde '~' at the beginning of each line
    /// Print the terminal version at 1/3 of the screen
    /// The clippy warning is disabled because it doesn't matter if the version is exactly at 1/3 of our screen
    pub fn print_rows() -> Result<(), Error> {
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

    pub fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }

    /// Print the welcome message
    /// The clippy warning is disabled because it doesn't matter if the version is not exactly centred
    pub fn draw_welcome_message() -> Result<(), Error> {
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
