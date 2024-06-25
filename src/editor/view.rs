mod buffer;
use super::terminal::{Position, Size, Terminal};
use buffer::Buffer;
use std::{io::Error, result};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct View {
    pub buffer: Buffer,
    pub needs_redraw: bool,
    pub size: Size,
}

impl Default for View {
    fn default() -> Self {
        Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::get_size().unwrap_or_default(),
        }
    }
}

impl View {
    pub fn resize(&mut self, to: Size) {
        self.size = to;
        self.needs_redraw = true;
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line.");
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
        for r in 0..height {
            if let Some(line) = self.buffer.lines.get(r) {
                let truncated_line = if line.len() >= width {
                    &line[0..width]
                } else {
                    line
                };
                let _ = Self::render_line(r, truncated_line);
            } else if r == vertical_center && self.buffer.is_empty() {
                let _ = Self::render_line(r, &Self::build_welcome_msg(width));
            } else {
                let _ = Self::render_line(r, &"~".to_string());
            }
        }
        self.needs_redraw = false;
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
}
