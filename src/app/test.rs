use super::*;
use crate::imap::{EmailBody, MockImapClient};
use crate::smtp::MockSmtpClient;
use crossterm::event::{KeyCode, KeyModifiers};

fn mock_clients() -> (MockImapClient, MockSmtpClient) {
    let mut imap = MockImapClient::new();
    // Set up default expectations for methods that may be called
    imap.expect_fetch_email().returning(|uid| {
        Ok(EmailBody {
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

fn sample_emails() -> Vec<EmailSummary> {
    vec![
        EmailSummary {
            uid: 1,
            subject: "First".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: true,
            snippet: "Hello".to_string(),
            message_id: Some("msg1@example.com".to_string()),
            in_reply_to: None,
            references: vec![],
        },
        EmailSummary {
            uid: 2,
            subject: "Second".to_string(),
            from: "bob@example.com".to_string(),
            date: "2025-01-02".to_string(),
            seen: false,
            snippet: "World".to_string(),
            message_id: Some("msg2@example.com".to_string()),
            in_reply_to: None,
            references: vec![],
        },
        EmailSummary {
            uid: 3,
            subject: "Third".to_string(),
            from: "carol@example.com".to_string(),
            date: "2025-01-03".to_string(),
            seen: false,
            snippet: "Test".to_string(),
            message_id: Some("msg3@example.com".to_string()),
            in_reply_to: None,
            references: vec![],
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
    app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(app.should_quit);
}

#[test]
fn other_keys_ignored() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(!app.should_quit);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!app.should_quit);
}

#[test]
fn j_moves_down() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_moves_up() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Up, KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn j_at_bottom_stays() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_at_top_stays() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn navigation_on_empty_list() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
    assert!(app.table_state.selected().is_none());
    app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE);
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
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn shift_g_selects_last() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn enter_opens_detail_view() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
}

#[test]
fn esc_returns_to_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Inbox));
}

#[test]
fn r_shows_reply_stub() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    if let View::Detail(ref state) = app.view {
        assert_eq!(
            state.status_message.as_deref(),
            Some("Reply not yet implemented")
        );
    } else {
        panic!("expected detail view");
    }
}

#[test]
fn m_d_deletes_email_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Inbox));
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_a_archives_email_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Inbox));
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_d_deletes_email_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_a_archives_email_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_r_marks_read_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    // First email (index 0 after reverse) is uid=3, seen=false
    assert!(!app.emails[0].seen);
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(app.emails[0].seen);
}

#[test]
fn r_reply_stub_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert_eq!(
        app.status_message.as_deref(),
        Some("Reply not yet implemented")
    );
}

#[test]
fn mark_seen_on_open() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    // First email (index 0 after reverse) is uid=3, seen=false
    assert!(!app.emails[0].seen);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    // After opening, the email should be marked as seen
    assert!(app.emails[0].seen);
}

#[test]
fn q_quits_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
    app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(app.should_quit);
}

#[test]
fn thread_grouping_with_reply() {
    let (imap, smtp) = mock_clients();
    let emails = vec![
        EmailSummary {
            uid: 1,
            subject: "Original".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: true,
            snippet: "Hello".to_string(),
            message_id: Some("orig@example.com".to_string()),
            in_reply_to: None,
            references: vec![],
        },
        EmailSummary {
            uid: 2,
            subject: "Re: Original".to_string(),
            from: "bob@example.com".to_string(),
            date: "2025-01-02".to_string(),
            seen: false,
            snippet: "Reply".to_string(),
            message_id: Some("reply@example.com".to_string()),
            in_reply_to: Some("orig@example.com".to_string()),
            references: vec!["orig@example.com".to_string()],
        },
    ];
    let app = App::new(emails, imap, smtp);
    // Both emails should be in the same thread — only 1 thread in inbox
    assert_eq!(app.threads.len(), 1);
    assert_eq!(app.threads[0].len(), 2);
}

#[test]
fn thread_grouping_by_subject() {
    let (imap, smtp) = mock_clients();
    let emails = vec![
        EmailSummary {
            uid: 1,
            subject: "Hello World".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: true,
            snippet: "Hi".to_string(),
            message_id: None,
            in_reply_to: None,
            references: vec![],
        },
        EmailSummary {
            uid: 2,
            subject: "RE: Hello World".to_string(),
            from: "bob@example.com".to_string(),
            date: "2025-01-02".to_string(),
            seen: false,
            snippet: "Reply".to_string(),
            message_id: None,
            in_reply_to: None,
            references: vec![],
        },
    ];
    let app = App::new(emails, imap, smtp);
    // Should be grouped by subject matching — only 1 thread
    assert_eq!(app.threads.len(), 1);
    assert_eq!(app.threads[0].len(), 2);
}
