mod buffer;
use super::terminal::{Size, Terminal};
use buffer::Buffer;
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View {
    pub buffer: Buffer,
}

impl View {
    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
        }
    }

    pub fn render_buffer(&self) -> Result<(), Error> {
        match Terminal::get_size() {
            Ok(Size { width: _, height }) => {
                for r in 0..height {
                    Terminal::clear_line()?;
                    if let Some(line) = self.buffer.lines.get(r) {
                        Terminal::print(line)?;
                        Terminal::print("\r\n")?;
                    } else {
                        Self::draw_empty_row()?;
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn render_welcome_msg(&self) -> Result<(), Error> {
        match Terminal::get_size() {
            Ok(Size { width: _, height }) => {
                for r in 0..height {
                    Terminal::clear_line()?;
                    #[allow(clippy::integer_division)]
                    if self.buffer.is_empty() && r == height / 3 {
                        Self::draw_welcome_msg()?;
                    } else {
                        Self::draw_empty_row()?;
                    }
                    if r.saturating_add(1) < height {
                        Terminal::print("\r\n")?;
                    }
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn render(&self) -> Result<(), Error> {
        if self.buffer.is_empty() {
            self.render_welcome_msg()?;
        } else {
            self.render_buffer()?;
        }
        Ok(())
    }

    fn draw_welcome_msg() -> Result<(), Error> {
        let mut welcome_msg = format!("{NAME} editor -- version {VERSION}");
        let width = Terminal::get_size()?.width;
        let len = welcome_msg.len();
        #[allow(clippy::integer_division)]
        let padding = (width.saturating_sub(len)) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_msg = format!("~{spaces}{welcome_msg}");
        //at most as wide as the screen
        welcome_msg.truncate(width);
        Terminal::print(&welcome_msg)?;
        Ok(())
    }

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
        Ok(())
    }
}
