use std::fmt;
use std::fmt::Formatter;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Copy, Clone)]
enum GraphemeWidth {
    Half,
    Full,
}

impl GraphemeWidth {
    fn saturating_add(self, other: usize) -> usize {
        match self {
            Self::Half => other.saturating_add(1),
            Self::Full => other.saturating_add(2),
        }
    }
}

struct TextFragment {
    grapheme: String,
    rendered_width: GraphemeWidth,
    replacement: Option<char>,
}

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        let fragments = Self::str_to_fragments(line_str);
        Self { fragments }
    }
    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .graphemes(true)
            .map(|grapheme| {
                let unicode_width = grapheme.width();
                let rendered_width = match unicode_width {
                    0 | 1 => GraphemeWidth::Half,
                    _ => GraphemeWidth::Full,
                };
                let replacement = match (unicode_width, grapheme) {
                    //Here, we replace all whitespace, except space and tab. Actually, there are some whitespaces that are not removed, add theme here if you want them gone
                    _ if grapheme.contains(char::is_whitespace)
                        && grapheme != " "
                        && grapheme != "\t" =>
                    {
                        Some('␣')
                    }
                    //Here, we check if the character is a control character like Bell or Null
                    _ if grapheme.chars().any(char::is_control) && grapheme != "\t" => Some('▯'),
                    (0, _) => Some('.'),
                    (_, _) => None,
                };
                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    replacement,
                }
            })
            .collect()
    }

    pub fn get_visible_graphemes(&self, range: Range<usize>) -> String {
        if range.start >= range.end {
            return String::new();
        }
        let mut result = String::new();
        let mut current_pos = 0;
        for fragment in &self.fragments {
            let fragment_end = fragment.rendered_width.saturating_add(current_pos);
            if current_pos >= range.end {
                break;
            }
            if fragment_end > range.start {
                if fragment_end > range.end || current_pos < range.start {
                    result.push('⋯');
                } else if let Some(char) = fragment.replacement {
                    result.push(char);
                } else {
                    result.push_str(&fragment.grapheme);
                }
            }
            current_pos = fragment_end;
        }
        result
    }

    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }

    pub fn width_until(&self, grapheme_index: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_index)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    pub fn split_at(&mut self, grapheme_index: usize) -> Self {
        if grapheme_index > self.grapheme_count() {
            return Self::default();
        }
        let remainder = self.fragments.split_off(grapheme_index);
        Self {
            fragments: remainder,
        }
    }

    pub fn insert_character(&mut self, character: char, grapheme_index: usize) {
        let mut result = String::new();

        for (index, fragment) in self.fragments.iter().enumerate() {
            if index == grapheme_index {
                result.push(character);
            }
            result.push_str(&fragment.grapheme);
        }
        if grapheme_index >= self.fragments.len() {
            result.push(character);
        }
        self.fragments = Self::str_to_fragments(&result);
    }

    pub fn delete(&mut self, grapheme_index: usize) {
        let mut result = String::new();
        for (index, fragment) in self.fragments.iter().enumerate() {
            if grapheme_index != index {
                result.push_str(&fragment.grapheme);
            }
        }
        self.fragments = Self::str_to_fragments(&result);
    }

    pub fn append(&mut self, other: &Self) {
        let mut concat = self.to_string();
        concat.push_str(&other.to_string());
        self.fragments = Self::str_to_fragments(&concat);
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let result: String = self
            .fragments
            .iter()
            .map(|fragment| fragment.grapheme.clone())
            .collect();
        write!(formatter, "{result}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_character() {
        let mut line: Line = Line::from("test insert character ");
        let line_width = line.fragments.len();
        line.insert_character('!', line_width);
        assert_eq!(line_width.saturating_add(1), line.fragments.len());
    }

    #[test]
    fn test_suppr_character() {
        let mut line: Line = Line::from("test suppr character");
        let line_width = line.grapheme_count();
        line.delete(0);
        assert_eq!(line_width.saturating_sub(1), line.fragments.len());
    }

    #[test]
    fn test_supp_character_out_of_line() {
        let mut line: Line = Line::from("test suppr character");
        let line_width = line.grapheme_count();
        line.delete(line.grapheme_count());
        assert_eq!(line_width, line.fragments.len());
        line.delete(line.grapheme_count().saturating_add(10));
        assert_eq!(line_width, line.fragments.len());
    }

    #[test]
    fn test_tab() {
        let mut line: Line = Line::from("a");
        assert_eq!(line.grapheme_count(), 1);
        line.insert_character('\t', 0);
        assert_eq!(line.grapheme_count(), 2);
    }
}
