mod terminal;
mod view;

use crate::editor::terminal::{Position, Size, Terminal};
use crate::editor::view::View;
use crossterm::event::{read, Event, Event::Key, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::io::Error;
use std::panic::{set_hook, take_hook};

#[derive(Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

/// This represents our Editor
/// It manages all the events and printing that happen in the terminal
/// It relies on our Terminal and the functions of the crossterm crate to work
pub struct Editor {
    should_quit: bool,
    cursor_location: Location,
    view: View,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::set_up()?;
        let mut view = View::default();
        let args: Vec<String> = std::env::args().collect();
        if let Some(first_arg) = args.get(1) {
            view.load(first_arg);
        }
        Ok(Self {
            should_quit: false,
            cursor_location: Location::default(),
            view,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => {
                    self.evaluate_event(event);
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read the event {:?}",err)                        
                    }

                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        match event {
            Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q' | 'Q'), KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }
                (
                    KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Right
                    | KeyCode::Left
                    | KeyCode::End
                    | KeyCode::Home
                    | KeyCode::PageUp
                    | KeyCode::PageDown,
                    _,
                ) => {
                    self.move_caret(code);
                }
                _ => {}
            },
            Event::Resize(width_u16, height_u16) => {
                //Systems where usize < u16 will cause problems
                #[allow(clippy::as_conversions)]
                let width = width_u16 as usize;
                //Systems where usize < u16 will cause problems
                #[allow(clippy::as_conversions)]
                let height = height_u16 as usize;
                self.view.resize(Size { width, height });
            }
            _ => {}
        }
    }

    fn move_caret(&mut self, key_code: KeyCode) {
        let Location { mut x, mut y } = self.cursor_location;
        let Size { width, height } = Terminal::get_size().unwrap_or_default();
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
        self.cursor_location = Location { x, y };
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_cursor();
        self.view.render();
        let _ = Terminal::move_cursor_to(Position::from(&self.cursor_location));
        let _ = Terminal::show_cursor();
        let _ = Terminal::execute();
    }
}


impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye ! ~~");
        } 
    }
}


impl From<&Location> for Position {
    fn from(location: &Location) -> Position {
        Position {
            col: location.x,
            row: location.y,
        }
    }
}