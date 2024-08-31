use std::io::Error;
use crate::editor::view::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>
}

impl Buffer {

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let file_content = std::fs::read_to_string(file_name)?;
        let mut lines: Vec<Line> = Vec::new();
        for line in file_content.lines() {
            lines.push(Line::from(line));
        };
        Ok(Self {lines})
    }
}