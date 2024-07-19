pub struct StatusBar {
    pub file_name: String,
    pub num_line: usize,
    pub caret_line: usize,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            num_line: 0,
            caret_line: 0,
        }
    }
}

impl StatusBar {
    pub fn render(&mut self) {}
}
