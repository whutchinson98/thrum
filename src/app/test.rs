use super::*;
use crossterm::event::KeyCode;

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
fn default_state() {
    let app = App::default();
    assert!(!app.should_quit);
    assert!(app.table_state.selected().is_none());
}

#[test]
fn q_quits() {
    let mut app = App::default();
    app.handle_key(KeyCode::Char('q'));
    assert!(app.should_quit);
}

#[test]
fn other_keys_ignored() {
    let mut app = App::default();
    app.handle_key(KeyCode::Char('a'));
    assert!(!app.should_quit);
    app.handle_key(KeyCode::Enter);
    assert!(!app.should_quit);
}

#[test]
fn j_moves_down() {
    let mut app = App::new(sample_emails());
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(1));
    app.handle_key(KeyCode::Down);
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_moves_up() {
    let mut app = App::new(sample_emails());
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
    let mut app = App::new(sample_emails());
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(2));
}

#[test]
fn k_at_top_stays() {
    let mut app = App::new(sample_emails());
    app.handle_key(KeyCode::Char('k'));
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn navigation_on_empty_list() {
    let mut app = App::default();
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
    let app = App::new(sample_emails());
    assert_eq!(app.emails[0].uid, 3);
    assert_eq!(app.emails[1].uid, 2);
    assert_eq!(app.emails[2].uid, 1);
}

#[test]
fn g_selects_first() {
    let mut app = App::new(sample_emails());
    app.handle_key(KeyCode::Char('j'));
    app.handle_key(KeyCode::Char('j'));
    assert_eq!(app.table_state.selected(), Some(2));
    app.handle_key(KeyCode::Char('g'));
    assert_eq!(app.table_state.selected(), Some(0));
}

#[test]
fn shift_g_selects_last() {
    let mut app = App::new(sample_emails());
    assert_eq!(app.table_state.selected(), Some(0));
    app.handle_key(KeyCode::Char('G'));
    assert_eq!(app.table_state.selected(), Some(2));
}
