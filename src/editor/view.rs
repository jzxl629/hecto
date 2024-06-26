mod buffer;
mod line;
mod location;
use super::editorcommand::{Direction, EditorCommand};
use super::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use location::Location;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    location: Location,
    scroll_offset: Location,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap_or_default(),
            location: Location::default(),
            scroll_offset: Location::default(),
        }
    }
}

impl View {
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
        let top = self.scroll_offset.y;
        for r in 0..height {
            if let Some(line) = self.buffer.lines.get(r.saturating_add(top)) {
                let _ = Self::render_line(
                    r,
                    &line.get(self.scroll_offset.x..self.scroll_offset.x.saturating_add(width)),
                );
            } else if r == vertical_center && self.buffer.is_empty() {
                let _ = Self::render_line(r, &Self::build_welcome_msg(width));
            } else {
                let _ = Self::render_line(r, &"~".to_string());
            }
        }
        self.needs_redraw = false;
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    pub fn get_position(&self) -> Position {
        self.location.subtract(&self.scroll_offset).into()
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Move(direction) => self.move_text_location(direction),
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Quit => {}
        }
    }

    pub fn move_text_location(&mut self, direction: Direction) {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::get_size().unwrap_or_default();
        match direction {
            Direction::Up => {
                y = y.saturating_sub(1);
            }
            Direction::Down => {
                y = y.saturating_add(1);
            }
            Direction::Left => {
                x = x.saturating_sub(1);
            }
            Direction::Right => {
                x = x.saturating_add(1);
            }
            Direction::PageUp => {
                y = 0;
            }
            Direction::PageDown => {
                y = height.saturating_sub(1);
            }
            Direction::Home => {
                x = 0;
            }
            Direction::End => {
                x = width.saturating_sub(1);
            }
        }
        self.location = Location { x, y };
        self.scroll_location_into_view();
    }

    //updates scroll_offset when scrolling
    fn scroll_location_into_view(&mut self) {
        let Size { height, width } = Terminal::get_size().unwrap_or_default();
        let Location { x, y } = self.location;
        let mut offset_changed = false;
        let Location {
            x: mut scroll_offset_x,
            y: mut scroll_offset_y,
        } = self.scroll_offset;

        if y < scroll_offset_y {
            scroll_offset_y = y;
            offset_changed = true;
        } else if y >= scroll_offset_y.saturating_add(height) {
            scroll_offset_y = scroll_offset_y.saturating_sub(height).saturating_add(1);
            offset_changed = true;
        }

        if x < scroll_offset_x {
            scroll_offset_x = x;
            offset_changed = true;
        } else if x >= scroll_offset_x.saturating_add(width) {
            scroll_offset_x = x.saturating_sub(width).saturating_add(1);
            offset_changed = true;
        }
        self.needs_redraw = offset_changed;
        self.scroll_offset = Location {
            x: scroll_offset_x,
            y: scroll_offset_y,
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
}
