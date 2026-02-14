mod app;
mod config;
mod ui;

use std::path::PathBuf;

use app::App;
use clap::Parser;

#[derive(Parser)]
#[command(name = "thrum", about = "A terminal email client")]
struct Cli {
    /// Path to config file (default: ~/.config/thrum.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let _config = config::load(cli.config).map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}
