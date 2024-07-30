mod buffer;
mod line;
use super::documentstatus::DocumentStatus;
use super::editorcommand::{Direction, EditorCommand};
use super::fileinfo::FileInfo;
use super::terminal::{Position, Size, Terminal};
use super::{NAME, VERSION};
use buffer::Buffer;
use std::cmp::min;

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
    margin_bottom: usize,
}

#[derive(Clone, Copy, Default)]
pub struct Location {
    grapheme_index: usize,
    line_index: usize,
}

impl View {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::get_size().unwrap_or_default();
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Size {
                height: size.height.saturating_sub(margin_bottom),
                width: size.width,
            },
            margin_bottom,
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Insert(c) => self.insert_char(c),
            EditorCommand::Delete => self.delete(),
            EditorCommand::Backspace => self.backspace(),
            EditorCommand::Enter => self.enter(),
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Save => self.save(),
            EditorCommand::Quit => {}
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || self.size.height == 0 {
            return;
        }
        let Size { width, height } = self.size;
        if width <= 0 || height <= 0 {
            return;
        }
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.row;
        for r in 0..height - 2 {
            if let Some(line) = self.buffer.lines.get(r.saturating_add(top)) {
                let _ = Self::render_line(
                    r,
                    &line.get_visible_graphemes(
                        self.scroll_offset.col..self.scroll_offset.col.saturating_add(width),
                    ),
                );
            } else if r == vertical_center && self.buffer.is_empty() {
                let _ = Self::render_line(r, &Self::build_welcome_msg(width));
            } else {
                let _ = Self::render_line(r, &"~".to_string());
            }
        }
        self.needs_redraw = false;
    }

    pub fn move_text_location(&mut self, direction: Direction) {
        let Location {
            mut grapheme_index,
            mut line_index,
        } = self.text_location;
        let Size { height, width: _ } = self.size;
        match direction {
            Direction::Up => {
                line_index = line_index.saturating_sub(1);
            }
            Direction::Down => {
                line_index = line_index.saturating_add(1);
            }
            Direction::Left => {
                grapheme_index = grapheme_index.saturating_sub(1);
            }
            Direction::Right => {
                grapheme_index = grapheme_index.saturating_add(1);
            }
            Direction::PageUp => {
                line_index = line_index.saturating_sub(height).saturating_sub(1);
            }
            Direction::PageDown => {
                line_index = line_index.saturating_add(height).saturating_sub(1);
            }
            Direction::Home => {
                grapheme_index = 0;
            }
            Direction::End => {
                grapheme_index = match self.buffer.lines.get(line_index) {
                    Some(line) => line.graphemes_len(),
                    None => 0,
                };
                //x = self.buffer.lines.get(y).map_or(0, Line::len);
            }
        }
        grapheme_index = match self.buffer.lines.get(line_index) {
            Some(line) => min(line.graphemes_len(), grapheme_index),
            None => 0,
        };
        line_index = min(line_index, self.buffer.get_size());
        self.text_location = Location {
            grapheme_index,
            line_index,
        };
        self.scroll_location_into_view();
    }

    //updates scroll_offset when scrolling
    fn scroll_location_into_view(&mut self) {
        let Size { height, width } = self.size;
        //convert text location to a position on the grid
        let Position { col, row } = self.text_location_to_position();
        let mut offset_changed = false;
        let Position {
            col: mut scroll_offset_x,
            row: mut scroll_offset_y,
        } = self.scroll_offset;

        if row < scroll_offset_y {
            scroll_offset_y = row;
            offset_changed = true;
        } else if row >= scroll_offset_y.saturating_add(height) {
            scroll_offset_y = scroll_offset_y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        if col < scroll_offset_x {
            scroll_offset_x = col;
            offset_changed = true;
        } else if col >= scroll_offset_x.saturating_add(width) {
            scroll_offset_x = col.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        self.needs_redraw = offset_changed;
        self.scroll_offset = Position {
            col: scroll_offset_x,
            row: scroll_offset_y,
        };
    }

    fn build_welcome_msg(width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_msg.len();
        let remaining_width = width.saturating_sub(1);
        if remaining_width < len {
            return "~".to_string();
        }
        format!("{:<1}{:^remaining_width$}", "~", welcome_msg)
    }

    fn resize(&mut self, to: Size) {
        self.size = Size {
            width: to.width,
            height: to.height.saturating_sub(self.margin_bottom),
        };
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line.");
    }

    pub fn get_caret_position(&self) -> Position {
        self.text_location_to_position()
            .subtract(self.scroll_offset)
    }

    fn text_location_to_position(&self) -> Position {
        let Location {
            grapheme_index,
            line_index,
        } = self.text_location;
        let total_width = match self.buffer.lines.get(line_index) {
            Some(line) => line.get_previous_width(grapheme_index),
            None => 0,
        };
        Position {
            row: line_index,
            col: total_width,
        }
    }

    fn insert_char(&mut self, c: char) {
        let Location {
            mut grapheme_index,
            line_index,
        } = self.text_location;
        self.buffer.insert_in_line(line_index, c, grapheme_index);
        grapheme_index = grapheme_index.saturating_add(1);
        self.text_location = Location {
            grapheme_index,
            line_index,
        };
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }

    fn delete(&mut self) {
        let Location {
            grapheme_index,
            line_index,
        } = self.text_location;
        if line_index != self.buffer.get_size() - 1
            || grapheme_index < self.buffer.get_line_length(line_index) - 1
        {
            if grapheme_index == self.buffer.get_line_length(line_index) - 1 {
                let next_line_index = line_index.saturating_add(1);
                self.buffer.merge(line_index, next_line_index);
            } else {
                self.buffer.delete(line_index, grapheme_index);
            }
            self.scroll_location_into_view();
            self.needs_redraw = true;
        }
    }

    fn backspace(&mut self) {
        let Location {
            mut grapheme_index,
            mut line_index,
        } = self.text_location;
        if grapheme_index != 0 || line_index != 0 {
            if grapheme_index == 0 {
                let last_line_index = line_index.saturating_sub(1);
                grapheme_index = self.buffer.get_line_length(last_line_index);
                self.buffer.merge(last_line_index, line_index);
                line_index = last_line_index;
            } else {
                grapheme_index = grapheme_index.saturating_sub(1);
                self.buffer.delete(line_index, grapheme_index);
            }
            self.text_location = Location {
                grapheme_index,
                line_index,
            };
            self.scroll_location_into_view();
            self.needs_redraw = true;
        }
    }

    fn enter(&mut self) {
        let Location {
            mut grapheme_index,
            mut line_index,
        } = self.text_location;
        self.buffer.enter(line_index, grapheme_index);
        grapheme_index = 0;
        line_index = line_index.saturating_add(1);
        self.text_location = Location {
            grapheme_index,
            line_index,
        };
        self.scroll_location_into_view();
        self.needs_redraw = true;
    }

    fn save(&mut self) {
        let _ = self.buffer.save();
    }

    pub fn get_current_document_status(&self) -> DocumentStatus {
        DocumentStatus {
            num_lines: self.buffer.get_size(),
            current_caret_line: self.text_location.line_index,
            file_name: format!("{}", self.buffer.file_info),
            is_modified: self.buffer.is_modified,
        }
    }
}
