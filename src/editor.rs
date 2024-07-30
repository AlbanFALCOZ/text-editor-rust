#![warn(clippy::all, clippy::pedantic, clippy::print_stdout)]

mod terminal;

use crate::editor::terminal::Terminal;
use crossterm::event::{read, Event::Key, KeyCode::Char, KeyEvent, KeyModifiers};
use std;
use std::io::Error;

pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn default() -> Self {
        Editor { should_quit: false }
    }

    pub fn set_up() -> Result<(), Error> {
        Terminal::enable_raw_mode()?;
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0, 0)?;
        Terminal::execute()
    }

    pub fn quit() -> Result<(), Error> {
        Terminal::disable_raw_mode()?;
        Terminal::clear_screen()?;
        Terminal::move_cursor_to(0,0)?;
        Terminal::execute()?;
        Ok(())
    }

    pub fn run(&mut self) {
        if let Err(err) = self.repl() {
            println!("{err:#?}");
        }
        println!("Goodbye ~~!");
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        Self::set_up()?;
        loop {
            if let Key(KeyEvent {
                code,
                modifiers,
                kind,
                state,
            }) = read()?
            {
                println!("Code {code:?}, keyModifiers : {modifiers:?}, kind : {kind:?}, state : {state:?} \r");
                match code {
                    Char('q') if modifiers == KeyModifiers::CONTROL => {
                        self.should_quit = true;
                    }
                    _ => (),
                }
            }
            if self.should_quit {
                break;
            }
        }
        Self::quit()?;
        Ok(())
    }
}
