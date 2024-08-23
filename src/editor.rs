mod terminal;
mod view;

use crate::editor::terminal::{Position, Size, Terminal};
use crossterm::event::{
    Event, Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read,
};
use std::io::Error;
use crate::editor::view::View;

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
    view: View,
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
            self.view.render()?;
            Terminal::move_cursor_to(Position::from(&self.cursor_location))?;
        };
        Terminal::show_cursor()?;
        Terminal::execute()?;
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
