use chrono::{DateTime, Datelike, Local};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};

use crate::app::{App, ComposeStep, View};
use crate::imap::ImapClient;
use crate::smtp::SmtpClient;

#[cfg(test)]
mod test;

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip(frame, app))
)]
pub fn render<I: ImapClient, S: SmtpClient>(frame: &mut Frame, app: &mut App<I, S>) {
    match &app.view {
        View::Inbox => render_inbox(frame, app),
        View::Detail(_) => render_detail(frame, app),
        View::Compose(_) => render_compose(frame, app),
    }
}

fn render_inbox<I: ImapClient, S: SmtpClient>(frame: &mut Frame, app: &mut App<I, S>) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_inbox_top_bar(frame, top);
    render_inbox_main(frame, main, app);
    render_inbox_status_bar(frame, status, app);
}

fn render_inbox_top_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let bar = Paragraph::new(
        Line::from(" q=Quit  j/k=Navigate  r=Reply  c=Compose  m-a=Archive  m-r=Read  m-d=Delete  m-l=Labels")
            .style(Style::new().bold()),
    );
    frame.render_widget(bar, area);
}

fn render_inbox_main<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &mut App<I, S>,
) {
    let block = Block::bordered().title(" Inbox ");

    if app.threads.is_empty() {
        let content = Paragraph::new("No messages").block(block);
        frame.render_widget(content, area);
    } else {
        let rows: Vec<Row> = app
            .threads
            .iter()
            .filter_map(|thread| {
                // Show the newest message (last in thread, oldest-first order)
                let email_idx = *thread.last()?;
                let e = app.emails.get(email_idx)?;
                let thread_count = thread.len();

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

                let mut subject_parts = vec![Span::raw(&e.subject)];
                if thread_count > 1 {
                    subject_parts.push(Span::styled(
                        format!(" ({thread_count})"),
                        Style::new().fg(Color::Cyan),
                    ));
                }
                subject_parts.push(Span::raw(" "));
                subject_parts.push(Span::styled(&e.snippet, Style::new().fg(Color::DarkGray)));
                let subject_cell = Cell::from(Line::from(subject_parts));

                let date_cell = Cell::from(format_date(&e.date));

                Some(
                    Row::new(vec![unread_cell, from_cell, subject_cell, date_cell])
                        .style(row_style),
                )
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

fn render_inbox_status_bar<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App<I, S>,
) {
    let text = if let Some(ref msg) = app.status_message {
        format!(" {msg}")
    } else {
        let count = app.threads.len();
        if count == 0 {
            String::new()
        } else {
            let selected = app.table_state.selected().map(|s| s + 1).unwrap_or(0);
            format!(" {selected}/{count} conversations")
        }
    };
    let bar = Paragraph::new(text);
    frame.render_widget(bar, area);
}

fn render_detail<I: ImapClient, S: SmtpClient>(frame: &mut Frame, app: &mut App<I, S>) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_detail_top_bar(frame, top);
    render_detail_main(frame, main, app);
    render_detail_status_bar(frame, status, app);
}

fn render_detail_top_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let bar = Paragraph::new(
        Line::from(
            " Esc=Back  r=Reply  c=Compose  m-d=Delete  m-a=Archive  m-r=Read  m-l=Labels  j/k=Navigate",
        )
        .style(Style::new().bold()),
    );
    frame.render_widget(bar, area);
}

fn render_detail_main<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App<I, S>,
) {
    let View::Detail(ref state) = app.view else {
        return;
    };

    let mut lines: Vec<Line> = Vec::new();

    for (i, msg) in state.thread.iter().enumerate() {
        let email = &app.emails[msg.email_index];
        let is_active = i == state.active_index;

        if let Some(ref body) = msg.body {
            // Expanded message
            let header_style = if is_active {
                Style::new().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::new()
            };

            lines.push(Line::from(vec![
                Span::styled("▼ From: ", header_style.bold()),
                Span::styled(&body.from, header_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  To:   ", header_style),
                Span::styled(body.to.join(", "), header_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Date: ", header_style),
                Span::styled(format_date(&body.date), header_style),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Subj: ", header_style),
                Span::styled(&body.subject, header_style),
            ]));
            lines.push(Line::from(""));

            for text_line in body.body_text.lines() {
                lines.push(Line::from(format!("  {text_line}")));
            }

            lines.push(Line::from(""));
        } else {
            // Collapsed message
            let style = if is_active {
                Style::new().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::new().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::styled("▶ ", style),
                Span::styled(&email.from, style.bold()),
                Span::styled(" — ", style),
                Span::styled(format_date(&email.date), style),
                Span::styled(" — ", style),
                Span::styled(&email.subject, style),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::bordered().title(" Email "))
        .wrap(Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    frame.render_widget(paragraph, area);
}

fn render_detail_status_bar<I: ImapClient, S: SmtpClient>(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    app: &App<I, S>,
) {
    let text = if let View::Detail(ref state) = app.view {
        state
            .status_message
            .as_deref()
            .map(|s| format!(" {s}"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let bar = Paragraph::new(text);
    frame.render_widget(bar, area);
}

fn render_compose<I: ImapClient, S: SmtpClient>(frame: &mut Frame, app: &mut App<I, S>) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let View::Compose(ref state) = app.view else {
        return;
    };

    // Top bar with keybind hints
    let hint = match state.step {
        ComposeStep::Bcc => " Esc=Cancel  Alt+S=Send",
        _ => " Esc=Cancel  Alt+S=Next",
    };
    let bar = Paragraph::new(Line::from(hint).style(Style::new().bold()));
    frame.render_widget(bar, top);

    // Main compose area
    let title = if state.is_reply {
        format!(" Reply: {} ", state.subject)
    } else {
        " New Email ".to_string()
    };
    let block = Block::bordered().title(title);
    let inner = block.inner(main);
    frame.render_widget(block, main);

    let mut lines: Vec<Line> = Vec::new();

    // Header fields
    let label_style = Style::new().bold();
    let active_style = Style::new().fg(Color::Yellow);

    // Subject field (editable for new emails, static for replies)
    if !state.is_reply {
        let subj_style = if state.step == ComposeStep::Subject {
            active_style
        } else {
            Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled("  Sub: ", label_style),
            Span::styled(&state.subject, subj_style),
            if state.step == ComposeStep::Subject {
                Span::styled("_", active_style)
            } else {
                Span::raw("")
            },
        ]));
    }

    let to_style = if state.step == ComposeStep::To {
        active_style
    } else {
        Style::new()
    };
    lines.push(Line::from(vec![
        Span::styled("  To:  ", label_style),
        Span::styled(&state.to, to_style),
        if state.step == ComposeStep::To {
            Span::styled("_", active_style)
        } else {
            Span::raw("")
        },
    ]));

    let cc_style = if state.step == ComposeStep::Cc {
        active_style
    } else {
        Style::new()
    };
    lines.push(Line::from(vec![
        Span::styled("  CC:  ", label_style),
        Span::styled(&state.cc, cc_style),
        if state.step == ComposeStep::Cc {
            Span::styled("_", active_style)
        } else {
            Span::raw("")
        },
    ]));

    let bcc_style = if state.step == ComposeStep::Bcc {
        active_style
    } else {
        Style::new()
    };
    lines.push(Line::from(vec![
        Span::styled("  BCC: ", label_style),
        Span::styled(&state.bcc, bcc_style),
        if state.step == ComposeStep::Bcc {
            Span::styled("_", active_style)
        } else {
            Span::raw("")
        },
    ]));

    // Separator
    lines.push(Line::from("  ─────────────────────────────────────────"));

    // Body text
    for (i, line) in state.body_lines.iter().enumerate() {
        if state.step == ComposeStep::Body && i == state.cursor_row {
            // Show cursor in the body line
            let (before, after) = if state.cursor_col <= line.len() {
                (&line[..state.cursor_col], &line[state.cursor_col..])
            } else {
                (line.as_str(), "")
            };
            lines.push(Line::from(vec![
                Span::raw(format!("  {before}")),
                Span::styled(
                    if after.is_empty() {
                        " ".to_string()
                    } else {
                        after.chars().next().unwrap().to_string()
                    },
                    Style::new().bg(Color::White).fg(Color::Black),
                ),
                Span::raw(if after.len() > 1 {
                    after[after.chars().next().unwrap().len_utf8()..].to_string()
                } else {
                    String::new()
                }),
            ]));
        } else {
            lines.push(Line::from(format!("  {line}")));
        }
    }

    // Quoted text
    if !state.quoted_text.is_empty() {
        lines.push(Line::from(""));
        for quoted_line in state.quoted_text.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {quoted_line}"),
                Style::new().fg(Color::DarkGray),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);

    // Status bar
    let status_text = state
        .status_message
        .as_deref()
        .map(|s| format!(" {s}"))
        .unwrap_or_else(|| {
            format!(
                " Step: {}",
                match state.step {
                    ComposeStep::Body => "Body",
                    ComposeStep::Subject => "Subject",
                    ComposeStep::To => "To",
                    ComposeStep::Cc => "CC",
                    ComposeStep::Bcc => "BCC",
                }
            )
        });
    let status_bar = Paragraph::new(status_text);
    frame.render_widget(status_bar, status);
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
