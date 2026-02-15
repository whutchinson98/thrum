use super::*;
use crate::imap::{EmailSummary, MockImapClient};
use crate::smtp::MockSmtpClient;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn mock_clients() -> (MockImapClient, MockSmtpClient) {
    let mut imap = MockImapClient::new();
    imap.expect_fetch_email().returning(|uid| {
        Ok(crate::imap::EmailBody {
            uid,
            subject: "Test".to_string(),
            from: "test@example.com".to_string(),
            to: vec!["me@example.com".to_string()],
            date: "2025-01-01".to_string(),
            body_text: "Test body".to_string(),
        })
    });
    imap.expect_mark_seen().returning(|_| Ok(()));
    imap.expect_delete_email().returning(|_| Ok(()));
    imap.expect_archive_email().returning(|_| Ok(()));
    (imap, MockSmtpClient::new())
}

#[test]
fn render_does_not_panic() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
}

#[test]
fn render_with_emails() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let (imap, smtp) = mock_clients();
    let mut app = App::new(
        vec![
            EmailSummary {
                uid: 1,
                subject: "Hello".to_string(),
                from: "alice@example.com".to_string(),
                date: "2025-01-01".to_string(),
                seen: false,
                snippet: "Hey there".to_string(),
                message_id: None,
                in_reply_to: None,
                references: vec![],
            },
            EmailSummary {
                uid: 2,
                subject: "Meeting".to_string(),
                from: "Bob Jones".to_string(),
                date: "2025-01-02".to_string(),
                seen: true,
                snippet: "Let's meet".to_string(),
                message_id: None,
                in_reply_to: None,
                references: vec![],
            },
        ],
        imap,
        smtp,
    );
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
}

#[test]
fn render_detail_view() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let (imap, smtp) = mock_clients();
    let mut app = App::new(
        vec![EmailSummary {
            uid: 1,
            subject: "Hello".to_string(),
            from: "alice@example.com".to_string(),
            date: "Mon, 01 Jan 2025 10:00:00 +0000".to_string(),
            seen: false,
            snippet: "Hey there".to_string(),
            message_id: None,
            in_reply_to: None,
            references: vec![],
        }],
        imap,
        smtp,
    );
    // Open detail view
    app.handle_key(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    );
    assert!(matches!(app.view, View::Detail(_)));
    terminal.draw(|frame| render(frame, &mut app)).unwrap();
}

#[test]
fn format_date_today() {
    let now = Local::now();
    let rfc2822 = now.to_rfc2822();
    let result = format_date(&rfc2822);
    // Should be time-only format like "3:33 AM"
    assert!(result.contains(':'));
    assert!(result.contains("AM") || result.contains("PM"));
}

#[test]
fn format_date_this_year() {
    let now = Local::now();
    // Use Jan 1 of current year (unless today IS Jan 1)
    let year = now.format("%Y").to_string();
    let date_str = format!("Mon, 01 Jan {year} 10:00:00 +0000");
    let result = format_date(&date_str);
    let today = now.date_naive();
    if today.month() == 1 && today.day() == 1 {
        // If today is Jan 1, it'll show time format
        assert!(result.contains(':'));
    } else {
        assert!(result.contains("Jan"));
    }
}

#[test]
fn format_date_old_year() {
    let result = format_date("Mon, 14 Feb 2022 10:00:00 +0000");
    assert!(result.contains("2022"));
    assert!(result.contains("Feb"));
}

#[test]
fn format_date_invalid() {
    let result = format_date("not a date");
    assert_eq!(result, "not a date");
}
