use super::documentstatus::DocumentStatus;
use super::editorcommand::EditorCommand;
use super::terminal::{Size, Terminal};
pub struct MessageBar {
    pub current_status: DocumentStatus,
    needs_redraw: bool,
    width: usize,
    position_y: usize,
    is_visible: bool,
}

impl MessageBar {
    pub fn new() -> Self {
        let size = Terminal::get_size().unwrap_or_default();
        let mut message_bar = Self {
            current_status: DocumentStatus::default(),
            width: size.width,
            needs_redraw: true,
            position_y: 0,
            is_visible: false,
        };
        message_bar.resize(size);
        message_bar
    }

    pub fn resize(&mut self, to: Size) {
        let mut position_y = 0;
        let mut is_visible = false;
        if let Some(result) = to.height.checked_sub(1) {
            position_y = result;
            is_visible = true;
        }
        self.width = to.width;
        self.position_y = position_y;
        self.needs_redraw = true;
        self.is_visible = is_visible;
    }

    pub fn render(&mut self) {
        if !self.needs_redraw || !self.is_visible {
            return;
        }
        if let Ok(size) = Terminal::get_size() {
            let msg = String::from("HELP: Ctrl-S = save | Ctrl-Q = quit");
            let to_print = if msg.len() <= size.width {
                msg
            } else {
                String::new()
            };
            let result = Terminal::invert_print(&to_print, self.position_y);
            debug_assert!(result.is_ok(), "Failed to render message bar");
            self.needs_redraw = false;
        }
    }

    pub fn handle_command(&mut self, command: EditorCommand) {
        if let EditorCommand::Resize(size) = command {
            self.resize(size);
        }
    }
}
