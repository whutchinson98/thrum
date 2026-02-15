use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

#[cfg(test)]
mod test;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub imap: ImapConfig,
    pub smtp: SmtpConfig,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub folder: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct SmtpConfig {
    pub url: String,
    pub authentication: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to execute password command: {0}")]
    Command(String),
    #[error("could not determine config directory")]
    NoConfigDir,
}

fn default_config_path() -> Result<PathBuf, ConfigError> {
    let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
    Ok(config_dir.join("thrum.toml"))
}

/// If the value is wrapped in backticks, execute it as a shell command and
/// return stdout. Otherwise return the value as-is.
fn expand_command(value: &str) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() >= 2 {
        let cmd = &trimmed[1..trimmed.len() - 1];
        #[cfg(feature = "tracing")]
        tracing::trace!(cmd, "executing password command");
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| ConfigError::Command(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ConfigError::Command(format!(
                "command failed ({}): {}",
                output.status, stderr
            )));
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("password command succeeded");
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        #[cfg(feature = "tracing")]
        tracing::trace!("using plain password value");
        Ok(value.to_string())
    }
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(level = tracing::Level::TRACE)
)]
pub fn load(path: Option<PathBuf>) -> Result<Config, ConfigError> {
    let config_path = match path {
        Some(p) => p,
        None => default_config_path()?,
    };

    #[cfg(feature = "tracing")]
    tracing::trace!(path = %config_path.display(), "reading config file");
    let contents = std::fs::read_to_string(&config_path)?;
    #[cfg(feature = "tracing")]
    tracing::trace!(bytes = contents.len(), "config file read");

    let mut config: Config = toml::from_str(&contents)?;
    #[cfg(feature = "tracing")]
    tracing::trace!("config parsed");

    #[cfg(feature = "tracing")]
    tracing::trace!("expanding password");
    config.imap.pass = expand_command(&config.imap.pass)?;
    #[cfg(feature = "tracing")]
    tracing::trace!("password expanded");

    Ok(config)
}
