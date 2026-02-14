use super::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn render_does_not_panic() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::default();
    terminal.draw(|frame| render(frame, &app)).unwrap();
}
