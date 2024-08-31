mod editorcommand;
mod terminal;
mod view;

use crate::editor::editorcommand::EditorCommand;
use crate::editor::terminal::{ Terminal};
use crate::editor::view::View;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::io::Error;
use std::panic::{set_hook, take_hook};

/// This represents our Editor
/// It manages all the events and printing that happen in the terminal
/// It relies on our Terminal and the functions of the crossterm crate to work
pub struct Editor {
    should_quit: bool,
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
                        panic!("Could not read the event {err:?}")
                    }
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match event {
            Event::Key(KeyEvent { kind, .. }) => kind == KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            match EditorCommand::try_from(event) {
                Ok(command) => if matches!(command, EditorCommand::Quit) {
                    self.should_quit = true;
                }
                    else {
                    self.view.handle_command(command);
                },
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not handle command : {err:?}")
                    }
                }
            }
        }
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_cursor();
        self.view.render();
        let _ = Terminal::move_cursor_to(self.view.get_position());
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
