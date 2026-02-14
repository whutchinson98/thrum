mod app;
mod config;
mod imap;
mod ui;

use std::path::PathBuf;

use app::App;
use clap::Parser;
use imap::ImapClient;

#[derive(Parser)]
#[command(name = "thrum", about = "A terminal email client")]
struct Cli {
    /// Path to config file (default: ~/.config/thrum.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    let config = config::load(cli.config).map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut client = imap::NativeImapClient::connect(&config.imap, &config.ssl)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let emails = client
        .fetch_inbox()
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    let mut terminal = ratatui::init();
    let result = App::new(emails).run(&mut terminal);
    ratatui::restore();
    result
}
