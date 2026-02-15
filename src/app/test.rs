use super::*;
use crate::imap::MockImapClient;
use crate::smtp::MockSmtpClient;
use crossterm::event::KeyCode;

fn mock_clients() -> (MockImapClient, MockSmtpClient) {
    (MockImapClient::new(), MockSmtpClient::new())
}

fn sample_emails() -> Vec<EmailSummary> {
    vec![
        EmailSummary {
            uid: 1,
            subject: "First".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: true,
            snippet: "Hello".to_string(),
        },
        EmailSummary {
            uid: 2,
            subject: "Second".to_string(),
            from: "bob@example.com".to_string(),
            date: "2025-01-02".to_string(),
            seen: false,
            snippet: "World".to_string(),
        },
        EmailSummary {
            uid: 3,
            subject: "Third".to_string(),
            from: "carol@example.com".to_string(),
            date: "2025-01-03".to_string(),
            seen: false,
            snippet: "Test".to_string(),
        },
    ]
}

#[test]
fn empty_app() {
    let (imap, smtp) = mock_clients();
    let app = App::new(Vec::new(), imap, smtp);
    assert!(!app.should_quit);
    assert!(app.table_state.selected().is_none());
}

#[test]
fn q_quits() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    app.handle_key(KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn other_keys_ignored() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    app.handle_key(KeyCode::Char('a'));
    assert!(!app.should_quit);
    app.handle_key(KeyCode::Enter);
    assert!(!app.should_quit);
}

#[test]
fn j_moves_down() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Down);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_moves_up() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('k'));
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Up);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn j_at_bottom_stays() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_at_top_stays() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('k'));
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn navigation_on_empty_list() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    app.handle_key(KeyCode::Char('j'));
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('k'));
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('g'));
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('G'));
    assert!(app.table_state.selected().is_none());
}

#[test]
fn emails_reversed_for_newest_first() {
    let (imap, smtp) = mock_clients();
    let app = App::new(sample_emails(), imap, smtp);
    assert_eq!(app.emails[0].uid, 3);
    assert_eq!(app.emails[1].uid, 2);
    assert_eq!(app.emails[2].uid, 1);
}

#[test]
fn g_selects_first() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('g'));
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn shift_g_selects_last() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('G'));
    assert_eq!(app.table_state.selected(), Some(2));
}
