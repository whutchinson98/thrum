use super::*;
use crate::imap::{EmailBody, MockImapClient};
use crate::smtp::MockSmtpClient;
use crossterm::event::{KeyCode, KeyModifiers};

const SENDER: &str = "me@example.com";

fn mock_clients() -> (MockImapClient, MockSmtpClient) {
    let mut imap = MockImapClient::new();
    // Set up default expectations for methods that may be called
    imap.expect_fetch_email().returning(|uid, _folder| {
        Ok(EmailBody {
            uid,
            subject: "Test".to_string(),
            from: "test@example.com".to_string(),
            to: vec!["me@example.com".to_string()],
            date: "2025-01-01".to_string(),
            body_text: "Test body".to_string(),
        })
    });
    imap.expect_mark_seen().returning(|_, _| Ok(()));
    imap.expect_delete_email().returning(|_, _| Ok(()));
    imap.expect_archive_email().returning(|_, _| Ok(()));
    imap.expect_append().returning(|_, _| Ok(()));
    (imap, MockSmtpClient::new())
}

fn sample_emails() -> Vec<EmailSummary> {
    vec![
        EmailSummary {
            uid: 1,
            folder: "INBOX".to_string(),
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
            folder: "INBOX".to_string(),
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
            folder: "INBOX".to_string(),
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
    let app = App::new(Vec::new(), imap, smtp, SENDER.to_string(), None);
    assert!(!app.should_quit);
    assert!(app.table_state.selected().is_none());
}

#[test]
fn q_quits() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE);
    assert!(app.should_quit);
}

#[test]
fn other_keys_ignored() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(!app.should_quit);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!app.should_quit);
}

#[test]
fn j_moves_down() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_moves_up() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
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
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_at_top_stays() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn navigation_on_empty_list() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(Vec::new(), imap, smtp, SENDER.to_string(), None);
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
    let app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    assert_eq!(app.emails[0].uid, 3);
    assert_eq!(app.emails[1].uid, 2);
    assert_eq!(app.emails[2].uid, 1);
}

#[test]
fn g_selects_first() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn shift_g_selects_last() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn enter_opens_detail_view() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
}

#[test]
fn esc_returns_to_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Inbox));
}

#[test]
fn r_opens_compose_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Compose(_)));
}

#[test]
fn m_d_deletes_email_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
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
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
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
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('d'), KeyModifiers::NONE);
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_a_archives_email_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    let initial_count = app.emails.len();
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert_eq!(app.emails.len(), initial_count - 1);
}

#[test]
fn m_r_marks_read_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    // First email (index 0 after reverse) is uid=3, seen=false
    assert!(!app.emails[0].seen);
    app.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(app.emails[0].seen);
}

#[test]
fn r_opens_compose_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Compose(_)));
}

#[test]
fn mark_seen_on_open() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    // First email (index 0 after reverse) is uid=3, seen=false
    assert!(!app.emails[0].seen);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    // After opening, the email should be marked as seen
    assert!(app.emails[0].seen);
}

#[test]
fn q_quits_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
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
            folder: "INBOX".to_string(),
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
            folder: "INBOX".to_string(),
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
    let app = App::new(emails, imap, smtp, SENDER.to_string(), None);
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
            folder: "INBOX".to_string(),
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
            folder: "INBOX".to_string(),
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
    let app = App::new(emails, imap, smtp, SENDER.to_string(), None);
    // Should be grouped by subject matching — only 1 thread
    assert_eq!(app.threads.len(), 1);
    assert_eq!(app.threads[0].len(), 2);
}

#[test]
fn compose_esc_cancels() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Compose(_)));
    app.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Inbox));
}

#[test]
fn compose_body_text_input() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Compose(_)));
    app.handle_key(KeyCode::Char('H'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.body_lines[0], "Hi");
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn compose_ctrl_s_advances_steps() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    // Body -> To
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::To);
    } else {
        panic!("expected compose view");
    }

    // To -> Cc (To is pre-filled from reply)
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::Cc);
    } else {
        panic!("expected compose view");
    }

    // Cc -> Bcc
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::Bcc);
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn compose_to_validation() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    // Advance to To step
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::To);
    }

    // Clear the To field
    if let View::Compose(ref mut state) = app.view {
        state.to.clear();
        state.to_cursor = 0;
    }

    // Try to advance — should stay on To step with error
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::To);
        assert!(state.status_message.is_some());
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn compose_subject_re_prefix() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    if let View::Compose(ref state) = app.view {
        // sample_emails first thread has subject "Third" (after reverse, uid=3 is first)
        assert!(state.subject.starts_with("Re: "));
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn compose_in_reply_to_set() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);
    if let View::Compose(ref state) = app.view {
        // First thread after reverse is uid=3 with message_id "msg3@example.com"
        assert_eq!(state.in_reply_to.as_deref(), Some("msg3@example.com"));
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn compose_send_calls_smtp() {
    let (imap, mut smtp) = mock_clients();
    smtp.expect_send().returning(|_| Ok(vec![]));

    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    // Advance through all steps: Body -> To -> Cc -> Bcc -> send
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT); // -> To
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT); // -> Cc
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT); // -> Bcc
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT); // -> send

    assert!(matches!(app.view, View::Inbox));
    assert_eq!(app.status_message.as_deref(), Some("Reply sent!"));
}

#[test]
fn c_opens_new_email_from_inbox() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
    if let View::Compose(ref state) = app.view {
        assert!(!state.is_reply);
        assert!(state.subject.is_empty());
        assert!(state.to.is_empty());
        assert!(state.in_reply_to.is_none());
        assert!(state.references.is_empty());
        assert!(state.quoted_text.is_empty());
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn c_opens_new_email_from_detail() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(matches!(app.view, View::Detail(_)));
    app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);
    assert!(matches!(app.view, View::Compose(_)));
    if let View::Compose(ref state) = app.view {
        assert!(!state.is_reply);
    }
}

#[test]
fn new_email_step_flow_includes_subject() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);

    // Body -> Subject
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::Subject);
    } else {
        panic!("expected compose view");
    }

    // Subject empty — should stay on Subject with error
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::Subject);
        assert!(state.status_message.is_some());
    }

    // Type a subject
    app.handle_key(KeyCode::Char('H'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('i'), KeyModifiers::NONE);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.subject, "Hi");
    }

    // Subject -> To
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::To);
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn reply_step_flow_skips_subject() {
    let (imap, smtp) = mock_clients();
    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    // Body -> To (skips Subject for replies)
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    if let View::Compose(ref state) = app.view {
        assert_eq!(state.step, ComposeStep::To);
    } else {
        panic!("expected compose view");
    }
}

#[test]
fn new_email_send_calls_smtp() {
    let (imap, mut smtp) = mock_clients();
    smtp.expect_send().returning(|_| Ok(vec![]));

    let mut app = App::new(sample_emails(), imap, smtp, SENDER.to_string(), None);
    app.handle_key(KeyCode::Char('c'), KeyModifiers::NONE);

    // Body -> Subject
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    // Type subject
    app.handle_key(KeyCode::Char('T'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('e'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('s'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('t'), KeyModifiers::NONE);
    // Subject -> To
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    // Type recipient
    app.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('@'), KeyModifiers::NONE);
    app.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);
    // To -> Cc
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    // Cc -> Bcc
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);
    // Bcc -> send
    app.handle_key(KeyCode::Char('s'), KeyModifiers::ALT);

    assert!(matches!(app.view, View::Inbox));
    assert_eq!(app.status_message.as_deref(), Some("Email sent!"));
}
