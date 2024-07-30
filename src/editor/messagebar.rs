use super::editorcommand::EditorCommand;
use super::terminal::{Size, Terminal};
use std::time::Duration;
use std::time::Instant;
const DEFAULT_DURATION: Duration = Duration::new(5, 0);

#[derive(Default)]
pub struct MessageBar {
    needs_redraw: bool,
    is_cleared: bool,
    message: Message,
}

struct Message {
    text: String,
    time: Instant,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            text: String::new(),
            time: Instant::now(),
        }
    }
}

impl Message {
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > DEFAULT_DURATION
    }
}

impl MessageBar {
    pub fn new() -> Self {
        Self {
            needs_redraw: true,
            is_cleared: false,
            message: Message::default(),
        }
    }

    pub fn update_msg(&mut self, msg: String) {
        self.message = Message {
            text: msg,
            time: Instant::now(),
        };
        self.needs_redraw = true;
    }

    pub fn resize(&mut self, _to: Size) {
        self.needs_redraw = true;
    }

    pub fn render(&mut self) {
        self.needs_redraw = self.needs_redraw || (!self.is_cleared && self.message.is_expired());
        if !self.needs_redraw {
            return;
        }
        let size = Terminal::get_size().unwrap_or_default();
        if self.message.is_expired() {
            self.is_cleared = true;
        }
        let msg = if self.message.is_expired() {
            ""
        } else {
            &self.message.text
        };
        let result = Terminal::print_row(size.height.saturating_sub(1), msg);
        debug_assert!(result.is_ok(), "Failed to render message bar");
        self.needs_redraw = false;
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        if let EditorCommand::Resize(size) = command {
            self.resize(size);
        }
    }
}
