use std::io::Error;
use crate::editor::view::line::Line;
use crate::editor::view::Location;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>
}

impl Buffer {

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let file_content = std::fs::read_to_string(file_name)?;
        let mut lines: Vec<Line> = Vec::new();
        for line in file_content.lines() {
            lines.push(Line::from(line));
        };
        Ok(Self {lines})
    }

    pub fn insert_char(&mut self, character: char, at: &Location) {
        if at.line_index > self.lines.len() {
            return;
        }
        if at.line_index == self.lines.len() {
            self.lines.push(Line::from(&character.to_string()));
        }
        else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_character(character, at.grapheme_index);
        }
    }

    pub fn delete(&mut self, at: &Location){
        if let Some(line) = self.lines.get_mut(at.line_index) {
            line.delete(at.grapheme_index);
        }
    }
}