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
        #[cfg(feature = "tracing")]
        tracing::trace!("entering main loop");
        while !self.should_quit {
            #[cfg(feature = "tracing")]
            tracing::trace!("drawing frame");
            terminal.draw(|frame| ui::render(frame, self))?;
            #[cfg(feature = "tracing")]
            tracing::trace!("frame drawn, waiting for event");
            self.handle_event()?;
        }
        #[cfg(feature = "tracing")]
        tracing::trace!("main loop exited");
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn handle_event(&mut self) -> std::io::Result<()> {
        #[cfg(feature = "tracing")]
        tracing::trace!("waiting for crossterm event");
        let event = event::read()?;
        #[cfg(feature = "tracing")]
        tracing::trace!(?event, "event received");
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            #[cfg(feature = "tracing")]
            tracing::trace!(?key.code, "key press");
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
            #[cfg(feature = "tracing")]
            tracing::trace!("quit requested");
            self.should_quit = true;
        }
    }
}
