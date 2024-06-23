use super::terminal::{Size, Terminal};
use std::io::Error;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct View;

impl View {
    pub fn render() -> Result<(), Error> {
        match Terminal::get_size() {
            Ok(Size { width: _, height }) => {
                Terminal::clear_line()?;
                Terminal::print("Hello, world!")?;
                for r in 1..height {
                    Terminal::clear_line()?;
                    #[allow(clippy::integer_division)]
                    if r == height / 3 {
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
