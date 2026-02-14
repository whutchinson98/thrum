mod app;
mod ui;

use app::App;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}
