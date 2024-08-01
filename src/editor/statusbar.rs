use super::documentstatus::DocumentStatus;
use super::terminal::{Size, Terminal};
pub struct StatusBar {
    pub current_status: DocumentStatus,
    needs_redraw: bool,
    width: usize,
    position_y: usize,
    margin_bottom: usize,
    is_visible: bool,
}

impl StatusBar {
    pub fn new(margin_bottom: usize) -> Self {
        let size = Terminal::get_size().unwrap_or_default();
        let mut status_bar = Self {
            current_status: DocumentStatus::default(),
            width: size.width,
            needs_redraw: true,
            position_y: 0,
            margin_bottom,
            is_visible: false,
        };
        status_bar.resize(size);
        status_bar
    }

    pub fn resize(&mut self, to: Size) {
        let mut position_y = 0;
        let mut is_visible = false;
        if let Some(result) = to
            .height
            .checked_sub(self.margin_bottom)
            .and_then(|result| result.checked_sub(1))
        {
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
            let num_lines = self.current_status.num_lines_to_string();
            let modified_status = self.current_status.is_modified_to_string();
            let beginning = format!(
                "{} - {num_lines} {modified_status}",
                self.current_status.file_name
            );
            let position = self.current_status.caret_position_to_string();
            let remainder_len = size.width.saturating_sub(beginning.len());
            let status = format!("{beginning}{position:>remainder_len$}");
            let to_print = if status.len() <= size.width {
                status
            } else {
                String::new()
            };
            let result = Terminal::invert_print(&to_print, self.position_y);
            debug_assert!(result.is_ok(), "Failed to render status bar");
            self.needs_redraw = false;
        }
    }

    pub fn update_document_status(&mut self, new_document_status: DocumentStatus) {
        if new_document_status != self.current_status {
            self.current_status = new_document_status;
            self.needs_redraw = true;
        }
    }
}
