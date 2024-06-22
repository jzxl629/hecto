use core::fmt::Display;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType};
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

pub struct Terminal {}

pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Terminal {
    pub fn terminate() -> Result<(), Error> {
        Self::buffer_flush()?;
        disable_raw_mode()
    }

    pub fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::buffer_flush()?;
        Ok(())
    }

    pub fn print<T: Display>(string: T) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }

    pub fn get_size() -> Result<Size, Error> {
        let (width_u16, height_u16) = size()?;
        #[allow(clippy::as_conversions)]
        let height = height_u16 as usize;
        #[allow(clippy::as_conversions)]
        let width = width_u16 as usize;
        Ok(Size { width, height })
    }

    //Clears everything
    pub fn clear_screen() -> Result<(), std::io::Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    pub fn buffer_flush() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
