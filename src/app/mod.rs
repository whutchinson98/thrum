use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::widgets::TableState;

use crate::imap::EmailSummary;
use crate::ui;

#[cfg(test)]
mod test;

pub struct App {
    pub should_quit: bool,
    pub emails: Vec<EmailSummary>,
    pub table_state: TableState,
}

impl App {
    pub fn new(mut emails: Vec<EmailSummary>) -> Self {
        emails.reverse();
        let mut table_state = TableState::default();
        if !emails.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            should_quit: false,
            emails,
            table_state,
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
        match key {
            KeyCode::Char('q') => {
                #[cfg(feature = "tracing")]
                tracing::trace!("quit requested");
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            _ => {}
        }
    }

    fn select_next(&mut self) {
        if self.emails.is_empty() {
            return;
        }
        let current = self.table_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.emails.len() - 1);
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.emails.is_empty() {
            return;
        }
        let current = self.table_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.table_state.select(Some(prev));
    }

    fn select_first(&mut self) {
        if !self.emails.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.emails.is_empty() {
            self.table_state.select(Some(self.emails.len() - 1));
        }
    }
}
