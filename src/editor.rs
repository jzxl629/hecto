mod terminal;
use core::cmp::min;
use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{self},
    KeyEvent, KeyEventKind, KeyModifiers,
};
use std::io::Error;
use terminal::{Position, Size, Terminal};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Default)]
struct Location {
    x: usize,
    y: usize,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    location: Location,
}

impl Editor {
    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> Result<(), Error> {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            };
            let event = read()?;
            self.evaluate_event(&event)?;
        }
        Ok(())
    }

    fn move_cursor_to(&mut self, key_code: KeyCode) -> Result<(), Error> {
        let Location { mut x, mut y } = self.location;
        let Size { height, width } = Terminal::get_size()?;
        match key_code {
            KeyCode::Up => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down => {
                y = min(height.saturating_sub(1), y.saturating_add(1));
            }
            KeyCode::Left => {
                x = x.saturating_sub(1);
            }
            KeyCode::Right => {
                x = min(width.saturating_sub(1), x.saturating_add(1));
            }
            KeyCode::PageUp => {
                y = 0;
            }
            KeyCode::PageDown => {
                y = height.saturating_sub(1);
            }
            KeyCode::Home => {
                x = 0;
            }
            KeyCode::End => {
                x = width.saturating_sub(1);
            }
            _ => (),
        }
        self.location = Location { x, y };
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) -> Result<(), Error> {
        if let Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('q') if *modifiers == KeyModifiers::CONTROL => {
                    self.should_quit = true;
                }
                KeyCode::Up
                | KeyCode::Down
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageUp
                | KeyCode::PageDown => {
                    self.move_cursor_to(*code)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        //initialize the position
        Terminal::move_caret_to(Position::default())?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::print("Goodbye. \r\n")?;
        } else {
            Self::draw_rows()?;
            Terminal::move_caret_to(Position {
                col: self.location.x,
                row: self.location.y,
            })?;
        }
        Terminal::show_caret()?;
        Terminal::buffer_flush()?;
        Ok(())
    }

    fn draw_rows() -> Result<(), Error> {
        match Terminal::get_size() {
            Ok(Size { width: _, height }) => {
                for r in 0..height {
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

    fn draw_empty_row() -> Result<(), Error> {
        Terminal::print("~")?;
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
        Terminal::print(welcome_msg)?;
        Ok(())
    }
}
