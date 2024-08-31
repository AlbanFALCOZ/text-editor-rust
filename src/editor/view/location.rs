use crate::editor::terminal::Position;

#[derive(Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

impl Location {
    pub fn subtract(&self, other: &Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}

impl From<Location> for Position {
    fn from(location: Location) -> Position {
        Position {
            col: location.x,
            row: location.y,
        }
    }
}
