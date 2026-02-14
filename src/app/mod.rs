use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::imap::EmailSummary;
use crate::ui;

#[cfg(test)]
mod test;

pub struct App {
    pub should_quit: bool,
    pub emails: Vec<EmailSummary>,
}

impl App {
    pub fn new(emails: Vec<EmailSummary>) -> Self {
        Self {
            should_quit: false,
            emails,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl App {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self, terminal))
    )]
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| ui::render(frame, self))?;
            self.handle_event()?;
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn handle_event(&mut self) -> std::io::Result<()> {
        let event = event::read()?;
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            self.handle_key(key.code);
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    pub fn handle_key(&mut self, key: KeyCode) {
        if let KeyCode::Char('q') = key {
            self.should_quit = true;
        }
    }
}
