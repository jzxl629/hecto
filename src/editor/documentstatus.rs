#[derive(Default, PartialEq, Eq, Debug)]
pub struct DocumentStatus {
    pub num_lines: usize,
    pub current_caret_line: usize,
    pub file_name: String,
    pub is_modified: bool,
}

impl DocumentStatus {
    pub fn num_lines_to_string(&self) -> String {
        let result = format!("{}", self.num_lines,);
        return result;
    }

    pub fn is_modified_to_string(&self) -> String {
        let result = format!("{}", if self.is_modified { "(modified)" } else { "" },);
        return result;
    }

    pub fn caret_position_to_string(&self) -> String {
        let result = format!("{}/{}", self.current_caret_line, self.num_lines);
        return result;
    }
}
