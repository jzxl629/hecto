use super::line::Line;
use std::fs;
use std::io::Error;
#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn load(file_name: &str) -> Result<Self, Error> {
        let file_contents = fs::read_to_string(file_name)?;
        let mut lines = Vec::new();
        for line in file_contents.lines() {
            lines.push(Line::from(line));
        }
        Ok(Self { lines })
    }

    pub fn size(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.lines.len()
    }

    pub fn get_line_length(&self, line_index: usize) -> usize {
        if line_index == self.lines.len() {
            return 0;
        }
        self.lines[line_index].graphemes_len()
    }

    pub fn insert_in_line(&mut self, line_index: usize, c: char, grapheme_index: usize) {
        if line_index == self.lines.len() {
            //inserts a new line
            self.lines.push(Line::from(""));
        }
        match self.lines.get_mut(line_index) {
            Some(line) => line.insert(c, grapheme_index),
            None => (),
        }
    }
}
