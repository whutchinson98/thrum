use super::*;
use crate::imap::EmailSummary;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn render_does_not_panic() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::default();
    terminal.draw(|frame| render(frame, &app)).unwrap();
}

#[test]
fn render_with_emails() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new(vec![
        EmailSummary {
            uid: 1,
            subject: "Hello".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
        },
        EmailSummary {
            uid: 2,
            subject: "Meeting".to_string(),
            from: "Bob Jones".to_string(),
            date: "2025-01-02".to_string(),
        },
    ]);
    terminal.draw(|frame| render(frame, &app)).unwrap();
}
