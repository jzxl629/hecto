mod documentstatus;
mod editorcommand;
mod fileinfo;
mod messagebar;
mod statusbar;
mod terminal;
mod view;
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use editorcommand::EditorCommand;
use messagebar::MessageBar;
use statusbar::StatusBar;
use std::env;
use std::io::Error;
use terminal::Terminal;
use view::View;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    should_quit: bool,
    view: View,
    status_bar: StatusBar,
    message_bar: MessageBar,
    title: String,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut editor = Editor {
            should_quit: false,
            view: View::new(2),
            status_bar: StatusBar::new(1),
            message_bar: MessageBar::new(),
            title: String::new(),
        };
        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            editor.view.load(file_name);
        }
        editor
            .message_bar
            .update_msg("HELP: Ctrl-S = save | Ctrl-Q = quit".to_string());
        editor.refresh_status();
        Ok(editor)
    }

    pub fn refresh_status(&mut self) {
        let status = self.view.get_current_document_status();
        let title = format!("{} - {NAME}", status.file_name);
        self.status_bar.update_document_status(status);
        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    pub fn run(&mut self) {
        loop {
            let new_document_status = self.view.get_current_document_status();
            self.status_bar.update_document_status(new_document_status);
            self.refresh_screen();
            if self.should_quit {
                break;
            };
            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                }
            }
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };
        if should_process {
            if let Ok(command) = EditorCommand::try_from(event) {
                if matches!(command, EditorCommand::Quit) {
                    self.should_quit = true;
                } else {
                    self.view.handle_command(command);
                    self.status_bar.handle_command(command);
                    self.message_bar.handle_command(command);
                }
            }
        }
    }

    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = self.view.render();
        let _ = self.status_bar.render();
        let _ = self.message_bar.render();
        //update the position of the caret
        let _ = Terminal::move_caret_to(self.view.get_caret_position());
        let _ = Terminal::show_caret();
        let _ = Terminal::buffer_flush();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye.\r\n");
        }
    }
}
