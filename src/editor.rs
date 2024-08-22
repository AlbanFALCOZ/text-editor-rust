#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

mod terminal;

use crate::editor::terminal::{Position, Size, Terminal};
use crossterm::event::{read, Event, Event::Key, KeyCode::Char, KeyModifiers};
use std;
use std::io::Error;


pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { should_quit: false }
    }

    pub fn set_up(&mut self) -> Result<(), Error> {
        Terminal::set_up()?;
        Terminal::set_size(Terminal::get_size()?)?;
        Self::print_rows()
    }

    pub fn quit(&mut self) -> Result<(), Error> {
        Terminal::terminate()
    }

    pub fn run(&mut self) {
        self.set_up().unwrap();
        let result = self.repl();
        self.quit().unwrap();
        result.unwrap();
    }

    pub fn repl(&mut self) -> Result<(), Error> {
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

    pub fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Key(event) => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == Char('q') {
                    self.should_quit = true;
                }
                /*Terminal::move_cursor_to(Position::default())?;
                let string: &str = stringify!(event);
                Terminal::print(string)?;*/
                Ok(())
            }
            Event::Resize(..) => {
                //Terminal::clear_screen()?;
                //Terminal::move_cursor_to(Position::default())?;
                //Self::print_rows()?;
                //Self::draw_welcome_message()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            println!("Goodbye ! ~~");
        }
        else {
            Self::print_rows()?;
            Terminal::move_cursor_to(Position::default())?;
        };
        Terminal::show_cursor()?;
        Terminal::execute()?;
        Ok(())
    }

    pub fn print_rows() -> Result<(), Error> {
        let Size { height, .. } = Terminal::get_size()?;
        for current_row in 0..height {
            Terminal::clear_line()?;
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            if current_row + 1 < height {
                Terminal::print("\r\n")?;
            }
        }
        Ok(())
    }

    pub fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }

    pub fn draw_welcome_message() -> Result<(), Error> {
        let width = Terminal::get_size()?.width as usize;
        let mut version = "Rust terminal version 0.5".to_string();
        let len = version.len();
        let padding = (width-len)/2;
        let spaces = " ".repeat(padding);
        version = format!("~{spaces}{version}");
        version.truncate(width);
        Terminal::print(version)?;
        Ok(())
    }
}
