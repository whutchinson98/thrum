use chrono::{DateTime, Datelike, Local};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};

use crate::app::App;
use crate::imap::ImapClient;
use crate::smtp::SmtpClient;

#[cfg(test)]
mod test;

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip(frame, app))
)]
pub fn render<I: ImapClient, S: SmtpClient>(frame: &mut Frame, app: &mut App<I, S>) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_top_bar(frame, top);
    render_main(frame, main, app);
    render_status_bar(frame, status, app);
}

fn render_top_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let bar = Paragraph::new(Line::from(" q=Quit  j/k=Navigate").style(Style::new().bold()));
    frame.render_widget(bar, area);
}

fn render_main<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &mut App<I, S>,
) {
    let block = Block::bordered().title(" Inbox ");

    if app.emails.is_empty() {
        let content = Paragraph::new("No messages").block(block);
        frame.render_widget(content, area);
    } else {
        let rows: Vec<Row> = app
            .emails
            .iter()
            .map(|e| {
                let row_style = if e.seen {
                    Style::new().fg(Color::Gray)
                } else {
                    Style::new().bold()
                };

                let unread_cell = Cell::from(if e.seen {
                    Span::raw("  ")
                } else {
                    Span::styled("● ", Style::new().fg(Color::Blue))
                });

                let from_cell = Cell::from(e.from.as_str());

                let subject_snippet = Line::from(vec![
                    Span::raw(&e.subject),
                    Span::raw(" "),
                    Span::styled(&e.snippet, Style::new().fg(Color::DarkGray)),
                ]);
                let subject_cell = Cell::from(subject_snippet);

                let date_cell = Cell::from(format_date(&e.date));

                Row::new(vec![unread_cell, from_cell, subject_cell, date_cell]).style(row_style)
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(20),
            Constraint::Fill(1),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .row_highlight_style(Style::new().bg(Color::DarkGray).fg(Color::White));

        frame.render_stateful_widget(table, area, &mut app.table_state);
    }
}

fn render_status_bar<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App<I, S>,
) {
    let count = app.emails.len();
    let text = if count == 0 {
        String::new()
    } else {
        let selected = app.table_state.selected().map(|s| s + 1).unwrap_or(0);
        format!(" {selected}/{count} messages")
    };
    let bar = Paragraph::new(text);
    frame.render_widget(bar, area);
}

pub fn format_date(raw: &str) -> String {
    let parsed = DateTime::parse_from_rfc2822(raw);

    let Ok(parsed) = parsed else {
        return raw.to_string();
    };

    let local = parsed.with_timezone(&Local);
    let now = Local::now();
    let today = now.date_naive();
    let msg_date = local.date_naive();

    if msg_date == today {
        local.format("%-I:%M %p").to_string()
    } else if msg_date.year() == today.year() {
        local.format("%b %-d").to_string()
    } else {
        local.format("%b %-d, %Y").to_string()
    }
}
