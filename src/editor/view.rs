mod buffer;
mod line;
use super::editorcommand::{DeleteOption, Direction, EditorCommand};
use super::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use std::cmp::min;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
    scroll_offset: Position,
}

#[derive(Clone, Copy, Default)]
pub struct Location {
    grapheme_index: usize,
    line_index: usize,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        }
    }
}

impl View {
    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Insert(c) => self.insert_char(c),
            EditorCommand::Delete(delete_option) => self.delete_grapheme(delete_option),
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
    }

    pub fn render(&mut self) {
        if !self.needs_redraw {
            return;
        }
        let Size { width, height } = self.size;
        if width == 0 || height == 0 {
            return;
        }
        #[allow(clippy::integer_division)]
        let vertical_center = height / 3;
        let top = self.scroll_offset.col;
        for r in 0..height {
            if let Some(line) = self.buffer.lines.get(r.saturating_add(top)) {
                let _ = Self::render_line(
                    r,
                    &line.get_visible_graphemes(
                        self.scroll_offset.row..self.scroll_offset.row.saturating_add(width),
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
        let Size { height, width: _ } = Terminal::get_size().unwrap_or_default();
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
        line_index = min(line_index, self.buffer.lines.len());
        self.text_location = Location {
            grapheme_index,
            line_index,
        };
        self.scroll_location_into_view();
    }

    //updates scroll_offset when scrolling
    fn scroll_location_into_view(&mut self) {
        let Size { height, width } = Terminal::get_size().unwrap_or_default();
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

        //TODO: Horizontal move needs accomodate graphemes
        //calculate width and move scroll_offset to that position
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
            return " ".to_string();
        }
        let welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_msg.len();
        if width <= len {
            return "~".to_string();
        }
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;
        let mut full_message = format!("~{}{}", " ".repeat(padding), welcome_msg);
        full_message.truncate(width);
        full_message
    }

    fn resize(&mut self, to: Size) {
        self.size = to;
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
        self.needs_redraw = true;
    }

    fn delete_grapheme(&mut self, delete_option: DeleteOption) {
        let Location {
            mut grapheme_index,
            line_index,
        } = self.text_location;
        if let DeleteOption::Backspace = delete_option {
            grapheme_index = grapheme_index.saturating_sub(1);
        }
        self.buffer.delete(line_index, grapheme_index);
        self.text_location = Location {
            grapheme_index,
            line_index,
        };
        self.needs_redraw = true;
    }
}
