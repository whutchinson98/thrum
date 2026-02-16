mod app;
mod config;
mod imap;
mod smtp;
mod ui;

use std::path::PathBuf;

use app::App;
use clap::Parser;
use imap::ImapClient;

#[derive(Parser)]
#[command(name = "thrum", version, about = "A terminal email client")]
struct Cli {
    /// Path to config file (default: ~/.config/thrum.toml)
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "tracing")]
    {
        let log_file = std::fs::File::create("thrum.log").expect("failed to create thrum.log");
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(log_file)
            .init();
        tracing::trace!("tracing initialized");
    }

    let cli = Cli::parse();
    #[cfg(feature = "tracing")]
    tracing::trace!("CLI args parsed");

    #[cfg(feature = "tracing")]
    tracing::trace!("loading config");
    let config = config::load(cli.config).map_err(|e| std::io::Error::other(e.to_string()))?;
    #[cfg(feature = "tracing")]
    tracing::trace!("config loaded");

    #[cfg(feature = "tracing")]
    tracing::trace!(host = %config.imap.host, port = config.imap.port, "connecting to IMAP server");
    let mut client = imap::NativeImapClient::connect(&config.imap)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    #[cfg(feature = "tracing")]
    tracing::trace!("IMAP connected");

    #[cfg(feature = "tracing")]
    tracing::trace!(host = %config.smtp.host, port = config.smtp.port, "connecting to SMTP server");
    let smtp_client = smtp::NativeSmtpClient::connect(&config.smtp)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    #[cfg(feature = "tracing")]
    tracing::trace!("SMTP connected");

    #[cfg(feature = "tracing")]
    tracing::trace!("fetching inbox");
    let emails = client
        .fetch_inbox()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    #[cfg(feature = "tracing")]
    tracing::trace!(count = emails.len(), "inbox fetched");

    #[cfg(feature = "tracing")]
    tracing::trace!("initializing terminal");
    let mut terminal = ratatui::init();
    #[cfg(feature = "tracing")]
    tracing::trace!("terminal initialized, starting app");

    let sender_from = config.sender.formatted_from();
    let result = App::new(emails, client, smtp_client, sender_from).run(&mut terminal);

    #[cfg(feature = "tracing")]
    tracing::trace!("app exited, restoring terminal");
    ratatui::restore();
    #[cfg(feature = "tracing")]
    tracing::trace!("terminal restored");

    result
}
