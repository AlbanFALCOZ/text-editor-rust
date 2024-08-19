#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

mod terminal;

use crate::editor::terminal::{Position, Size, Terminal};
use crossterm::event::{read, Event, Event::Key, KeyCode, KeyCode::Char, KeyModifiers};
use crossterm::style::{Color, Colors};
use std;
use std::io::Error;

pub struct Editor {
    should_quit: bool,
    pos: Position,
}

impl Editor {
    pub fn default() -> Self {
        Editor {
            should_quit: false,
            pos: Position::default(),
        }
    }

    pub fn set_up() -> Result<(), Error> {
        Terminal::enable_raw_mode()?;
        Terminal::clear_screen()?;

        //TODO
        //not working
        //Terminal::set_size(Size{length: 180, height: 38})?;
        //Terminal::execute()?;

        Terminal::set_color(Colors {
            foreground: Option::from(Color::Green),
            background: None,
        })?;
        Self::print_rows(Terminal::get_size()?.height);
        Terminal::move_cursor_to(&Position::default())?;
        Terminal::execute()
    }

    pub fn quit() -> Result<(), Error> {
        Terminal::disable_raw_mode()?;
        Terminal::clear_screen()?;
        Terminal::reset_color()?;
        Terminal::move_cursor_to(&Position::new(0, 0))?;
        Terminal::execute()?;
        Ok(())
    }

    pub fn run(&mut self) {
        Self::set_up().unwrap();
        let result = self.repl();
        Self::quit().unwrap();
        result.unwrap();
        println!("Goodbye ! ~~");
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        Self::set_up()?;
        while !self.should_quit {
            let event = read()?;
            self.evaluate_event(&event)?;
        }

        Self::quit()?;
        Ok(())
    }

    pub fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        match event {
            Key(event) => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == Char('q') {
                    self.should_quit = true;
                }
                Terminal::move_cursor_to(&self.pos)?;
                println!("{:?}", event);
                Ok(())
            }
            Event::Resize(.., height) => {
                Self::print_rows(*height);
                let mut size: Size = Terminal::get_size()?;
                size.length = size.length - 30;
                Self::print_version(Position {
                    x: size.length,
                    y: size.height,
                })?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn print_rows(height: u16) {
        for _empty_line in 0..height / 3 {
            println!();
        }
        for _row in height / 3..height {
            println!("~");
        }
    }

    pub fn print_version(position: Position) -> Result<(), Error> {
        Terminal::move_cursor_to(&position)?;
        println!("Rust terminal ver 0.5");
        Ok(())
    }
}
