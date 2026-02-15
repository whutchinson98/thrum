use std::collections::HashMap;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::widgets::TableState;

use crate::imap::{EmailBody, EmailSummary, ImapClient};
use crate::smtp::{self, SmtpClient};
use crate::ui;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComposeStep {
    Body,
    Subject,
    To,
    Cc,
    Bcc,
}

pub struct ComposeState {
    pub step: ComposeStep,
    pub is_reply: bool,
    pub body_lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub to: String,
    pub to_cursor: usize,
    pub cc: String,
    pub cc_cursor: usize,
    pub bcc: String,
    pub bcc_cursor: usize,
    pub subject: String,
    pub subject_cursor: usize,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub quoted_text: String,
    pub status_message: Option<String>,
}

pub enum View {
    Inbox,
    Detail(DetailState),
    Compose(Box<ComposeState>),
}

pub struct DetailState {
    pub thread: Vec<ThreadMessage>,
    pub active_index: usize,
    pub scroll_offset: u16,
    pub status_message: Option<String>,
}

pub struct ThreadMessage {
    pub email_index: usize,
    pub body: Option<EmailBody>,
}

pub struct App<I: ImapClient, S: SmtpClient> {
    pub should_quit: bool,
    pub emails: Vec<EmailSummary>,
    pub table_state: TableState,
    pub view: View,
    pub threads: Vec<Vec<usize>>,
    pub pending_prefix: bool,
    pub status_message: Option<String>,
    pub imap_client: I,
    pub smtp_client: S,
    pub sender_from: String,
}

impl<I: ImapClient, S: SmtpClient> App<I, S> {
    pub fn new(
        mut emails: Vec<EmailSummary>,
        imap_client: I,
        smtp_client: S,
        sender_from: String,
    ) -> Self {
        emails.reverse();
        let threads = build_threads(&emails);
        let mut table_state = TableState::default();
        if !threads.is_empty() {
            table_state.select(Some(0));
        }
        Self {
            should_quit: false,
            emails,
            table_state,
            view: View::Inbox,
            threads,
            pending_prefix: false,
            status_message: None,
            imap_client,
            smtp_client,
            sender_from,
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self, terminal), err)
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
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
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
            self.handle_key(key.code, key.modifiers);
        }
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    pub fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        // Handle prefix chord (m was pressed previously)
        if self.pending_prefix {
            self.pending_prefix = false;
            self.handle_prefix_chord(key);
            return;
        }

        match &self.view {
            View::Inbox => self.handle_inbox_key(key),
            View::Detail(_) => self.handle_detail_key(key, modifiers),
            View::Compose(_) => self.handle_compose_key(key, modifiers),
        }
    }

    fn handle_prefix_chord(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('d') => self.delete_selected_email(),
            KeyCode::Char('a') => self.archive_selected_email(),
            KeyCode::Char('r') => self.mark_selected_read(),
            KeyCode::Char('l') => {
                #[cfg(feature = "tracing")]
                tracing::trace!("label menu stub");
                self.status_message = Some("Labels not yet implemented".to_string());
                if let View::Detail(ref mut state) = self.view {
                    state.status_message = Some("Labels not yet implemented".to_string());
                }
            }
            _ => {} // Unknown chord — ignore
        }
    }

    fn handle_inbox_key(&mut self, key: KeyCode) {
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
            KeyCode::Enter => self.open_email(),
            KeyCode::Char('r') => self.start_reply(),
            KeyCode::Char('c') => self.start_new_email(),
            KeyCode::Char('m') => {
                #[cfg(feature = "tracing")]
                tracing::trace!("prefix key pressed");
                self.pending_prefix = true;
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                #[cfg(feature = "tracing")]
                tracing::trace!("returning to inbox");
                self.view = View::Inbox;
            }
            KeyCode::Char('q') => {
                #[cfg(feature = "tracing")]
                tracing::trace!("quit requested from detail");
                self.should_quit = true;
            }
            KeyCode::Char('r') => self.start_reply(),
            KeyCode::Char('c') => self.start_new_email(),
            KeyCode::Char('m') => {
                #[cfg(feature = "tracing")]
                tracing::trace!("prefix key pressed");
                self.pending_prefix = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let View::Detail(ref mut state) = self.view {
                    if state.active_index + 1 < state.thread.len() {
                        state.active_index += 1;
                    } else {
                        state.scroll_offset = state.scroll_offset.saturating_add(1);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let View::Detail(ref mut state) = self.view {
                    if state.active_index > 0 {
                        state.active_index -= 1;
                    } else {
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                }
            }
            KeyCode::Enter => {
                if let View::Detail(ref mut state) = self.view {
                    let idx = state.active_index;
                    if state.thread[idx].body.is_some() {
                        state.thread[idx].body = None;
                    } else {
                        let email_index = state.thread[idx].email_index;
                        let uid = self.emails[email_index].uid;
                        if let Ok(body) = self.imap_client.fetch_email(uid) {
                            state.thread[idx].body = Some(body);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn open_email(&mut self) {
        let thread_indices = self.selected_email_indices();
        if thread_indices.is_empty() {
            return;
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(?thread_indices, "opening email thread");

        // The most recent message is the last one in the thread (oldest-first order)
        let most_recent_idx = thread_indices.len() - 1;
        let most_recent_email_idx = thread_indices[most_recent_idx];
        let uid = self.emails[most_recent_email_idx].uid;

        // Mark as seen
        let _ = self.imap_client.mark_seen(uid);
        self.emails[most_recent_email_idx].seen = true;

        // Fetch full body for the most recent message
        let body = self.imap_client.fetch_email(uid).ok();

        let thread: Vec<ThreadMessage> = thread_indices
            .iter()
            .enumerate()
            .map(|(i, &email_index)| ThreadMessage {
                email_index,
                body: if i == most_recent_idx {
                    body.clone()
                } else {
                    None
                },
            })
            .collect();

        self.view = View::Detail(DetailState {
            thread,
            active_index: most_recent_idx,
            scroll_offset: 0,
            status_message: None,
        });
    }

    /// Get the email indices for the current selection.
    /// In inbox view, returns all indices in the selected thread.
    /// In detail view, returns just the active message's index.
    fn selected_email_indices(&self) -> Vec<usize> {
        match &self.view {
            View::Detail(state) => {
                vec![state.thread[state.active_index].email_index]
            }
            View::Inbox => {
                let Some(selected) = self.table_state.selected() else {
                    return vec![];
                };
                self.threads.get(selected).cloned().unwrap_or_default()
            }
            View::Compose(_) => vec![],
        }
    }

    /// Get all UIDs to act on. In inbox view, returns all UIDs in the thread.
    /// In detail view, returns just the active message's UID.
    fn selected_uids(&self) -> Vec<u32> {
        self.selected_email_indices()
            .iter()
            .filter_map(|&idx| self.emails.get(idx).map(|e| e.uid))
            .collect()
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn delete_selected_email(&mut self) {
        let uids = self.selected_uids();
        if uids.is_empty() {
            return;
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(?uids, "deleting emails");

        for &uid in &uids {
            let _ = self.imap_client.delete_email(uid);
        }
        self.emails.retain(|e| !uids.contains(&e.uid));
        self.threads = build_threads(&self.emails);
        self.fix_selection();
        self.view = View::Inbox;
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn archive_selected_email(&mut self) {
        let uids = self.selected_uids();
        if uids.is_empty() {
            return;
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(?uids, "archiving emails");

        for &uid in &uids {
            let _ = self.imap_client.archive_email(uid);
        }
        self.emails.retain(|e| !uids.contains(&e.uid));
        self.threads = build_threads(&self.emails);
        self.fix_selection();
        self.view = View::Inbox;
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn mark_selected_read(&mut self) {
        let uids = self.selected_uids();
        if uids.is_empty() {
            return;
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(?uids, "marking as read");

        for &uid in &uids {
            let _ = self.imap_client.mark_seen(uid);
        }
        for email in self.emails.iter_mut() {
            if uids.contains(&email.uid) {
                email.seen = true;
            }
        }
    }

    fn fix_selection(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.threads.len() && !self.threads.is_empty() {
                self.table_state.select(Some(self.threads.len() - 1));
            } else if self.threads.is_empty() {
                self.table_state.select(None);
            }
        }
    }

    fn select_next(&mut self) {
        if self.threads.is_empty() {
            return;
        }
        let current = self.table_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.threads.len() - 1);
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        if self.threads.is_empty() {
            return;
        }
        let current = self.table_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.table_state.select(Some(prev));
    }

    fn select_first(&mut self) {
        if !self.threads.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn select_last(&mut self) {
        if !self.threads.is_empty() {
            self.table_state.select(Some(self.threads.len() - 1));
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn start_reply(&mut self) {
        // Get the thread indices for the current selection
        let thread_indices = match &self.view {
            View::Inbox => {
                let Some(selected) = self.table_state.selected() else {
                    return;
                };
                self.threads.get(selected).cloned().unwrap_or_default()
            }
            View::Detail(state) => state.thread.iter().map(|m| m.email_index).collect(),
            View::Compose(_) => return,
        };

        if thread_indices.is_empty() {
            return;
        }

        // Most recent message is last in thread (oldest-first)
        let reply_to_idx = *thread_indices.last().unwrap();
        let reply_to = &self.emails[reply_to_idx];

        // Build subject with Re: prefix (avoid double-prefixing)
        let subject = if reply_to.subject.trim().to_lowercase().starts_with("re:") {
            reply_to.subject.clone()
        } else {
            format!("Re: {}", reply_to.subject)
        };

        // Set in_reply_to from the reply-to email's message_id
        let in_reply_to = reply_to.message_id.clone();

        // Build references chain
        let mut references = reply_to.references.clone();
        if let Some(ref mid) = reply_to.message_id
            && !references.contains(mid)
        {
            references.push(mid.clone());
        }

        // Extract email address from "Name <email>" or plain "email" format
        let to = extract_email_address(&reply_to.from);

        // Build quoted text by fetching bodies
        let mut quoted_parts = Vec::new();
        for &idx in &thread_indices {
            let email = &self.emails[idx];
            if let Ok(body) = self.imap_client.fetch_email(email.uid) {
                quoted_parts.push(format!(
                    "On {}, {} wrote:\n{}",
                    email.date,
                    email.from,
                    body.body_text
                        .lines()
                        .map(|l| format!("> {l}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
        let quoted_text = quoted_parts.join("\n\n");

        self.view = View::Compose(Box::new(ComposeState {
            step: ComposeStep::Body,
            is_reply: true,
            body_lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            to: to.clone(),
            to_cursor: to.len(),
            cc: String::new(),
            cc_cursor: 0,
            bcc: String::new(),
            bcc_cursor: 0,
            subject,
            subject_cursor: 0,
            in_reply_to,
            references,
            quoted_text,
            status_message: None,
        }));
    }

    fn start_new_email(&mut self) {
        if matches!(self.view, View::Compose(_)) {
            return;
        }

        self.view = View::Compose(Box::new(ComposeState {
            step: ComposeStep::Body,
            is_reply: false,
            body_lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            to: String::new(),
            to_cursor: 0,
            cc: String::new(),
            cc_cursor: 0,
            bcc: String::new(),
            bcc_cursor: 0,
            subject: String::new(),
            subject_cursor: 0,
            in_reply_to: None,
            references: vec![],
            quoted_text: String::new(),
            status_message: None,
        }));
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn handle_compose_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                #[cfg(feature = "tracing")]
                tracing::trace!("compose cancelled");
                self.view = View::Inbox;
            }
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::ALT) => {
                self.advance_compose_step();
            }
            _ => {
                let View::Compose(ref mut state) = self.view else {
                    return;
                };
                match state.step {
                    ComposeStep::Body => handle_body_input(state, key),
                    ComposeStep::Subject => {
                        handle_line_input(&mut state.subject, &mut state.subject_cursor, key);
                    }
                    ComposeStep::To => handle_line_input(&mut state.to, &mut state.to_cursor, key),
                    ComposeStep::Cc => handle_line_input(&mut state.cc, &mut state.cc_cursor, key),
                    ComposeStep::Bcc => {
                        handle_line_input(&mut state.bcc, &mut state.bcc_cursor, key);
                    }
                }
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn advance_compose_step(&mut self) {
        let View::Compose(ref mut state) = self.view else {
            return;
        };

        match state.step {
            ComposeStep::Body => {
                if state.is_reply {
                    state.step = ComposeStep::To;
                } else {
                    state.step = ComposeStep::Subject;
                }
                state.status_message = None;
            }
            ComposeStep::Subject => {
                if state.subject.trim().is_empty() {
                    state.status_message = Some("Subject cannot be empty".to_string());
                    return;
                }
                state.step = ComposeStep::To;
                state.status_message = None;
            }
            ComposeStep::To => {
                if state.to.trim().is_empty() {
                    state.status_message = Some("To field cannot be empty".to_string());
                    return;
                }
                state.step = ComposeStep::Cc;
                state.status_message = None;
            }
            ComposeStep::Cc => {
                state.step = ComposeStep::Bcc;
                state.status_message = None;
            }
            ComposeStep::Bcc => {
                self.send_email();
            }
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn send_email(&mut self) {
        let View::Compose(ref state) = self.view else {
            return;
        };

        let is_reply = state.is_reply;

        let mut body = state.body_lines.join("\n");
        if !state.quoted_text.is_empty() {
            body.push_str("\n\n");
            body.push_str(&state.quoted_text);
        }

        let to: Vec<String> = state
            .to
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let cc: Vec<String> = state
            .cc
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let bcc: Vec<String> = state
            .bcc
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let email = smtp::Email {
            from: self.sender_from.clone(),
            to,
            cc,
            bcc,
            subject: state.subject.clone(),
            body,
            in_reply_to: state.in_reply_to.clone(),
            references: state.references.clone(),
        };

        #[cfg(feature = "tracing")]
        tracing::trace!(
            from = %email.from,
            to = ?email.to,
            cc = ?email.cc,
            bcc = ?email.bcc,
            subject = %email.subject,
            in_reply_to = ?email.in_reply_to,
            "sending email"
        );

        match self.smtp_client.send(&email) {
            Ok(()) => {
                #[cfg(feature = "tracing")]
                tracing::trace!("email sent successfully");
                let msg = if is_reply {
                    "Reply sent!"
                } else {
                    "Email sent!"
                };
                self.status_message = Some(msg.to_string());
                self.view = View::Inbox;
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::trace!(%e, "email send failed");
                if let View::Compose(ref mut state) = self.view {
                    state.status_message = Some(format!("Send failed: {e}"));
                }
            }
        }
    }
}

fn handle_body_input(state: &mut ComposeState, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            state.body_lines[state.cursor_row].insert(state.cursor_col, c);
            state.cursor_col += c.len_utf8();
        }
        KeyCode::Enter => {
            let rest = state.body_lines[state.cursor_row].split_off(state.cursor_col);
            state.cursor_row += 1;
            state.body_lines.insert(state.cursor_row, rest);
            state.cursor_col = 0;
        }
        KeyCode::Backspace => {
            if state.cursor_col > 0 {
                state.cursor_col -= 1;
                state.body_lines[state.cursor_row].remove(state.cursor_col);
            } else if state.cursor_row > 0 {
                let line = state.body_lines.remove(state.cursor_row);
                state.cursor_row -= 1;
                state.cursor_col = state.body_lines[state.cursor_row].len();
                state.body_lines[state.cursor_row].push_str(&line);
            }
        }
        KeyCode::Left => {
            if state.cursor_col > 0 {
                state.cursor_col -= 1;
            }
        }
        KeyCode::Right => {
            if state.cursor_col < state.body_lines[state.cursor_row].len() {
                state.cursor_col += 1;
            }
        }
        KeyCode::Up => {
            if state.cursor_row > 0 {
                state.cursor_row -= 1;
                state.cursor_col = state
                    .cursor_col
                    .min(state.body_lines[state.cursor_row].len());
            }
        }
        KeyCode::Down => {
            if state.cursor_row + 1 < state.body_lines.len() {
                state.cursor_row += 1;
                state.cursor_col = state
                    .cursor_col
                    .min(state.body_lines[state.cursor_row].len());
            }
        }
        _ => {}
    }
}

fn handle_line_input(field: &mut String, cursor: &mut usize, key: KeyCode) {
    match key {
        KeyCode::Char(c) => {
            field.insert(*cursor, c);
            *cursor += c.len_utf8();
        }
        KeyCode::Backspace => {
            if *cursor > 0 {
                *cursor -= 1;
                field.remove(*cursor);
            }
        }
        KeyCode::Left => {
            if *cursor > 0 {
                *cursor -= 1;
            }
        }
        KeyCode::Right => {
            if *cursor < field.len() {
                *cursor += 1;
            }
        }
        _ => {}
    }
}

fn extract_email_address(from: &str) -> String {
    if let Some(start) = from.find('<')
        && let Some(end) = from.find('>')
    {
        return from[start + 1..end].to_string();
    }
    from.to_string()
}

/// Strip leading "Re:" / "RE:" / "re:" prefixes (possibly repeated) to get the base subject.
fn normalize_subject(subject: &str) -> String {
    let mut s = subject.trim();
    loop {
        let lower = s.to_lowercase();
        if let Some(rest) = lower.strip_prefix("re:") {
            s = &s[3..];
            s = s.trim_start();
            // Handle "re:" stripping on the actual string by length
            let _ = rest; // just used to detect prefix
        } else {
            break;
        }
    }
    s.to_lowercase()
}

fn build_threads(emails: &[EmailSummary]) -> Vec<Vec<usize>> {
    // Map message_id -> email index
    let mut id_to_index: HashMap<String, usize> = HashMap::new();
    for (i, email) in emails.iter().enumerate() {
        if let Some(ref mid) = email.message_id {
            id_to_index.insert(mid.clone(), i);
        }
    }

    // Map normalized subject -> first email index with that base subject
    let mut subject_to_index: HashMap<String, usize> = HashMap::new();
    // Iterate in reverse order so that oldest emails (highest index) get registered first
    for i in (0..emails.len()).rev() {
        let base = normalize_subject(&emails[i].subject);
        if !base.is_empty() {
            subject_to_index.entry(base).or_insert(i);
        }
    }

    // parent[i] = index of parent email (if found)
    let mut parent: Vec<Option<usize>> = vec![None; emails.len()];

    for (i, email) in emails.iter().enumerate() {
        // Try in_reply_to first
        if let Some(ref reply_to) = email.in_reply_to
            && let Some(&parent_idx) = id_to_index.get(reply_to)
        {
            parent[i] = Some(parent_idx);
            continue;
        }
        // Fall back to last reference
        let mut found = false;
        for r in email.references.iter().rev() {
            if let Some(&parent_idx) = id_to_index.get(r) {
                parent[i] = Some(parent_idx);
                found = true;
                break;
            }
        }
        if found {
            continue;
        }
        // Fall back to subject matching
        let base = normalize_subject(&email.subject);
        if !base.is_empty()
            && let Some(&first_idx) = subject_to_index.get(&base)
            && first_idx != i
        {
            parent[i] = Some(first_idx);
        }
    }

    // Find root of each email
    let mut root_of: Vec<usize> = (0..emails.len()).collect();
    for (i, root) in root_of.iter_mut().enumerate() {
        let mut current = i;
        while let Some(p) = parent[current] {
            current = p;
        }
        *root = current;
    }

    // Group by root
    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, root) in root_of.iter().enumerate() {
        groups.entry(*root).or_default().push(i);
    }

    // Sort each group oldest-first (higher index = older since emails are reversed)
    // Actually, emails are stored newest-first (reversed from IMAP order).
    // So higher index = older. We want oldest first in thread, so reverse each group.
    let mut threads: Vec<Vec<usize>> = groups.into_values().collect();
    for thread in &mut threads {
        thread.sort();
        thread.reverse(); // now oldest first (highest index = oldest)
    }

    // Sort threads by first appearance in email list
    threads.sort_by_key(|t| t.iter().copied().min().unwrap_or(0));

    threads
}
