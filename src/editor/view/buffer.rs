use super::line::Line;
use super::FileInfo;
use std::fs;
use std::fs::File;
use std::io::Error;
use std::io::Write;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub file_info: FileInfo,
    pub is_modified: bool,
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
        Ok(Self {
            lines,
            file_info: FileInfo::from(file_name),
            is_modified: false,
        })
    }

    pub fn get_size(&self) -> usize {
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
            self.lines.push(Line::from(""));
        }
        match self.lines.get_mut(line_index) {
            Some(line) => {
                line.insert(c, grapheme_index);
                self.is_modified = true;
            }
            None => (),
        }
    }

    pub fn insert_new_line(&mut self, index: usize) {
        self.lines.insert(index, Line::from(""));
    }

    pub fn enter(&mut self, line_index: usize, grapheme_index: usize) {
        if grapheme_index == 0 {
            self.insert_new_line(line_index);
        } else {
            //if enter is pressed within the line, split the line and create a new line from the remainder
            if let Some(line) = self.lines.get(line_index) {
                if grapheme_index < line.graphemes_len() {
                    let next_line_index = line_index.saturating_add(1);
                    let next_line = self.lines[line_index].split(grapheme_index);
                    self.lines.insert(next_line_index, next_line);
                } else {
                    let next_line_index = line_index.saturating_add(1);
                    self.insert_new_line(next_line_index);
                }
            }
        }
        self.is_modified = true;
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.file_info.path {
            let mut file = File::create(path)?;
            for (_, line) in self.lines.iter().enumerate() {
                writeln!(file, "{}", &line.line_to_string())?;
            }
        }
        self.is_modified = false;
        Ok(())
    }

    pub fn delete(&mut self, line_index: usize, grapheme_index: usize) {
        if line_index < self.lines.len() {
            match self.lines.get_mut(line_index) {
                Some(line) => {
                    line.delete(grapheme_index);
                    self.is_modified = true;
                }
                None => (),
            }
        }
    }

    pub fn merge(&mut self, line_index: usize, merge_to_index: usize) {
        let removed_line = self.lines.remove(merge_to_index);
        match self.lines.get_mut(line_index) {
            Some(line) => {
                line.append(&removed_line);
                self.is_modified = true;
            }
            None => (),
        }
    }
}
