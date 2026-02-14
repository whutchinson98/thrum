use super::*;
use crossterm::event::KeyCode;

#[test]
fn default_state() {
    let app = App::default();
    assert!(!app.should_quit);
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
