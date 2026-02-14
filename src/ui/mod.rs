use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use crate::app::App;

#[cfg(test)]
mod test;

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE, skip(frame, _app))
)]
pub fn render(frame: &mut Frame, _app: &App) {
    let [top, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    render_top_bar(frame, top);
    render_main(frame, main);
    render_status_bar(frame, status);
}

fn render_top_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let bar = Paragraph::new(Line::from(" q=Quit").style(Style::new().bold()));
    frame.render_widget(bar, area);
}

fn render_main(frame: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::bordered().title(" Inbox ");
    let content = Paragraph::new("No messages").block(block);
    frame.render_widget(content, area);
}

fn render_status_bar(frame: &mut Frame, area: ratatui::layout::Rect) {
    let bar = Paragraph::new("");
    frame.render_widget(bar, area);
}
