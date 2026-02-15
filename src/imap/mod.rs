use imap::types::Flag;
use native_tls::TlsConnector;

use crate::config::ImapConfig;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq)]
pub struct EmailSummary {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: String,
    pub seen: bool,
    pub snippet: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ImapError {
    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),
    #[error("IMAP error: {0}")]
    Imap(#[from] imap::Error),
}

#[cfg_attr(test, mockall::automock)]
pub trait ImapClient {
    fn fetch_inbox(&mut self) -> Result<Vec<EmailSummary>, ImapError>;
}

pub struct NativeImapClient {
    session: imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    folder: String,
}

impl NativeImapClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(config))
    )]
    pub fn connect(config: &ImapConfig) -> Result<Self, ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!("building TLS connector");
        let tls = TlsConnector::builder().build()?;
        #[cfg(feature = "tracing")]
        tracing::trace!("TLS connector built");

        let addr = (config.host.as_str(), config.port);

        #[cfg(feature = "tracing")]
        tracing::trace!(
            host = config.host,
            port = config.port,
            "connecting to IMAP server"
        );
        let client = imap::connect(addr, &config.host, &tls)?;
        #[cfg(feature = "tracing")]
        tracing::trace!("TLS connection established");

        #[cfg(feature = "tracing")]
        tracing::trace!(user = %config.user, "logging in");
        let session = client
            .login(&config.user, &config.pass)
            .map_err(|(e, _)| e)?;
        #[cfg(feature = "tracing")]
        tracing::trace!("login successful");

        Ok(Self {
            session,
            folder: config.folder.clone(),
        })
    }
}

impl ImapClient for NativeImapClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self))
    )]
    fn fetch_inbox(&mut self) -> Result<Vec<EmailSummary>, ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(folder = %self.folder, "selecting folder");
        self.session.select(&self.folder)?;
        #[cfg(feature = "tracing")]
        tracing::trace!("folder selected");

        #[cfg(feature = "tracing")]
        tracing::trace!("fetching messages");
        let messages = self
            .session
            .fetch("1:*", "(UID ENVELOPE FLAGS BODY.PEEK[TEXT]<0.200>)")?;
        #[cfg(feature = "tracing")]
        tracing::trace!(raw_count = messages.len(), "messages fetched from server");

        let mut emails = Vec::new();

        for fetch in messages.iter() {
            let uid = fetch.uid.unwrap_or(0);

            let seen = fetch.flags().iter().any(|f| matches!(f, Flag::Seen));

            let snippet = fetch.text().map(extract_snippet).unwrap_or_default();

            if let Some(envelope) = fetch.envelope() {
                let subject = envelope
                    .subject
                    .map(|s| String::from_utf8_lossy(s).into_owned())
                    .unwrap_or_default();

                let from = envelope
                    .from
                    .as_ref()
                    .and_then(|addrs| addrs.first())
                    .map(format_address)
                    .unwrap_or_default();

                let date = envelope
                    .date
                    .map(|d| String::from_utf8_lossy(d).into_owned())
                    .unwrap_or_default();

                emails.push(EmailSummary {
                    uid,
                    subject,
                    from,
                    date,
                    seen,
                    snippet,
                });
            }
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(count = emails.len(), "emails parsed");

        #[cfg(feature = "tracing")]
        tracing::trace!("logging out");
        self.session.logout()?;
        #[cfg(feature = "tracing")]
        tracing::trace!("logged out");

        Ok(emails)
    }
}

fn format_address(addr: &imap_proto::Address) -> String {
    if let Some(name) = addr.name {
        let name = String::from_utf8_lossy(name);
        if !name.is_empty() {
            return name.into_owned();
        }
    }

    let mailbox = addr
        .mailbox
        .map(|m| String::from_utf8_lossy(m).into_owned())
        .unwrap_or_default();
    let host = addr
        .host
        .map(|h| String::from_utf8_lossy(h).into_owned())
        .unwrap_or_default();

    if mailbox.is_empty() && host.is_empty() {
        String::new()
    } else {
        format!("{mailbox}@{host}")
    }
}

pub fn extract_snippet(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);

    let body = if let Some(plain) = extract_mime_plain_text(&text) {
        plain.to_string()
    } else if text.contains("Content-Type:") {
        // Single-part with MIME headers — skip to body after blank line
        text.split_once("\r\n\r\n")
            .or_else(|| text.split_once("\n\n"))
            .map(|(_, body)| body.to_string())
            .unwrap_or_else(|| text.into_owned())
    } else {
        text.into_owned()
    };

    let stripped = strip_html_tags(&body);

    let collapsed: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");

    truncate_at_word_boundary(&collapsed, 100)
}

fn extract_mime_plain_text(text: &str) -> Option<&str> {
    // Look for a multipart boundary
    let boundary = text
        .lines()
        .find(|line| line.contains("boundary="))
        .and_then(|line| {
            let start = line.find("boundary=")?;
            let rest = &line[start + 9..];
            let boundary = rest.trim_matches('"').trim_matches(';').trim();
            Some(boundary.to_string())
        });

    let boundary = boundary?;

    let parts: Vec<&str> = text.split(&format!("--{boundary}")).collect();

    for part in &parts {
        let lower = part.to_lowercase();
        if lower.contains("content-type: text/plain") || lower.contains("content-type:text/plain") {
            // Skip headers to get body
            if let Some(body_start) = part.find("\r\n\r\n") {
                return Some(&part[body_start + 4..]);
            }
            if let Some(body_start) = part.find("\n\n") {
                return Some(&part[body_start + 2..]);
            }
        }
    }

    None
}

fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
}

fn truncate_at_word_boundary(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }

    let truncated = &s[..max];
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &s[..last_space])
    } else {
        format!("{truncated}...")
    }
}
