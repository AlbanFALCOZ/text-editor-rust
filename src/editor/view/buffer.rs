use crate::editor::view::line::Line;
use crate::editor::view::Location;
use std::io::Error;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
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
        }
        Ok(Self { lines })
    }

    pub fn insert_char(&mut self, character: char, at: &Location) {
        if at.line_index > self.lines.len() {
            return;
        }
        if at.line_index == self.lines.len() {
            self.lines.push(Line::from(&character.to_string()));
        } else if let Some(line) = self.lines.get_mut(at.line_index) {
            line.insert_character(character, at.grapheme_index);
        }
    }

    pub fn delete(&mut self, at: &Location) {
        if let Some(line) = self.lines.get(at.line_index) {
            if at.grapheme_index >= line.grapheme_count()
                && self.lines.len() > at.line_index.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_index.saturating_add(1));
                //We checked that the line at line_index existed
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].append(&next_line);
            } else if at.grapheme_index < line.grapheme_count() {
                //We checked that the line at line_index existed
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_index].delete(at.grapheme_index);
            }
        }
    }

    pub fn insert_line(&mut self, at: &Location) {
        if let Some(line_to_split) = self.lines.get_mut(at.line_index) {
            let line_sliced = line_to_split.split_at(at.grapheme_index);
            self.lines
                .insert(at.line_index.saturating_add(1), line_sliced);
        }
    }
}
