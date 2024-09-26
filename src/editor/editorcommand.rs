use crate::editor::editorcommand::Direction::{Down, End, Home, Left, PageDown, PageUp, Right, Up};
use crate::editor::terminal::Size;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}

pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Insert(char),
    Delete,
    Backspace,
    Quit,
}

impl TryFrom<Event> for EditorCommand {
    type Error = String;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('q' | 'Q'), KeyModifiers::CONTROL) => Ok(Self::Quit),
                (KeyCode::Char(char), KeyModifiers::NONE) => Ok(Self::Insert(char)),
                (KeyCode::Backspace, KeyModifiers::NONE) => Ok(Self::Backspace),
                (KeyCode::Delete, KeyModifiers::NONE) => Ok(Self::Delete),
                (KeyCode::Up, _) => Ok(Self::Move(Up)),
                (KeyCode::Down, _) => Ok(Self::Move(Down)),
                (KeyCode::Left, _) => Ok(Self::Move(Left)),
                (KeyCode::Right, _) => Ok(Self::Move(Right)),
                (KeyCode::PageDown, _) => Ok(Self::Move(PageDown)),
                (KeyCode::PageUp, _) => Ok(Self::Move(PageUp)),
                (KeyCode::Home, _) => Ok(Self::Move(Home)),
                (KeyCode::End, _) => Ok(Self::Move(End)),
                _ => Err(format!("Key code not supported {code:?}")),
            },
            //Systems where usize < u16 will cause problems
            #[allow(clippy::as_conversions)]
            Event::Resize(width_u16, height_u16) => Ok(Self::Resize(Size {
                width: width_u16 as usize,
                height: height_u16 as usize,
            })),
            _ => Err(format!("Event not supported {event:?}")),
        }
    }
}
