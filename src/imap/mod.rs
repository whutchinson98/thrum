use imap::types::Flag;
use native_tls::TlsConnector;

use crate::config::ImapConfig;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq)]
pub struct EmailSummary {
    pub uid: u32,
    pub folder: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub date: String,
    pub seen: bool,
    pub snippet: String,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmailBody {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub date: String,
    pub body_text: String,
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
    fn fetch_email(&mut self, uid: u32, folder: &str) -> Result<EmailBody, ImapError>;
    fn mark_seen(&mut self, uid: u32, folder: &str) -> Result<(), ImapError>;
    fn delete_email(&mut self, uid: u32, folder: &str) -> Result<(), ImapError>;
    fn archive_email(&mut self, uid: u32, folder: &str) -> Result<(), ImapError>;
    fn append(&mut self, folder: &str, content: &[u8]) -> Result<(), ImapError>;
}

pub struct NativeImapClient {
    session: imap::Session<native_tls::TlsStream<std::net::TcpStream>>,
    folders: Vec<String>,
}

impl NativeImapClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(config), err)
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
            folders: config.folders.clone(),
        })
    }
}

impl ImapClient for NativeImapClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
    )]
    fn fetch_inbox(&mut self) -> Result<Vec<EmailSummary>, ImapError> {
        let mut emails = Vec::new();
        let mut seen_message_ids: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for folder in &self.folders.clone() {
            #[cfg(feature = "tracing")]
            tracing::trace!(folder = %folder, "selecting folder");
            self.session.select(folder)?;
            #[cfg(feature = "tracing")]
            tracing::trace!("folder selected");

            #[cfg(feature = "tracing")]
            tracing::trace!("fetching messages");
            let messages = self.session.fetch(
                "1:*",
                "(UID ENVELOPE FLAGS BODY.PEEK[TEXT]<0.200> BODY.PEEK[HEADER.FIELDS (References)])",
            )?;
            #[cfg(feature = "tracing")]
            tracing::trace!(raw_count = messages.len(), "messages fetched from server");

            for fetch in messages.iter() {
                let uid = fetch.uid.unwrap_or(0);

                let seen = fetch.flags().iter().any(|f| matches!(f, Flag::Seen));

                let snippet = fetch.text().map(extract_snippet).unwrap_or_default();

                let references = fetch.header().map(parse_references).unwrap_or_default();

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

                    let to = envelope
                        .to
                        .as_ref()
                        .and_then(|addrs| addrs.first())
                        .map(format_address)
                        .unwrap_or_default();

                    let date = envelope
                        .date
                        .map(|d| String::from_utf8_lossy(d).into_owned())
                        .unwrap_or_default();

                    let message_id = envelope
                        .message_id
                        .map(|m| String::from_utf8_lossy(m).into_owned());

                    let in_reply_to = envelope
                        .in_reply_to
                        .map(|r| String::from_utf8_lossy(r).into_owned());

                    // Deduplicate by message_id (keep first seen)
                    if let Some(ref mid) = message_id
                        && !seen_message_ids.insert(mid.clone())
                    {
                        continue;
                    }

                    emails.push(EmailSummary {
                        uid,
                        folder: folder.clone(),
                        subject,
                        from,
                        to,
                        date,
                        seen,
                        snippet,
                        message_id,
                        in_reply_to,
                        references,
                    });
                }
            }
        }

        #[cfg(feature = "tracing")]
        tracing::trace!(count = emails.len(), "emails parsed");

        Ok(emails)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
    )]
    fn fetch_email(&mut self, uid: u32, folder: &str) -> Result<EmailBody, ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(uid, folder, "fetching email body");

        self.session.select(folder)?;
        let messages = self
            .session
            .uid_fetch(uid.to_string(), "(UID ENVELOPE BODY.PEEK[TEXT])")?;

        let fetch = messages
            .iter()
            .next()
            .ok_or_else(|| imap::Error::Bad("message not found".to_string()))?;

        let body_text = fetch.text().map(extract_body_text).unwrap_or_default();

        let (subject, from, to, date) = if let Some(envelope) = fetch.envelope() {
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
            let to = envelope
                .to
                .as_ref()
                .map(|addrs| addrs.iter().map(format_address).collect())
                .unwrap_or_default();
            let date = envelope
                .date
                .map(|d| String::from_utf8_lossy(d).into_owned())
                .unwrap_or_default();
            (subject, from, to, date)
        } else {
            (String::new(), String::new(), Vec::new(), String::new())
        };

        Ok(EmailBody {
            uid,
            subject,
            from,
            to,
            date,
            body_text,
        })
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
    )]
    fn mark_seen(&mut self, uid: u32, folder: &str) -> Result<(), ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(uid, folder, "marking as seen");

        self.session.select(folder)?;
        self.session.uid_store(uid.to_string(), "+FLAGS (\\Seen)")?;
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
    )]
    fn delete_email(&mut self, uid: u32, folder: &str) -> Result<(), ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(uid, folder, "moving to Trash");

        self.session.select(folder)?;
        self.session.uid_mv(uid.to_string(), "Trash")?;
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self), err)
    )]
    fn archive_email(&mut self, uid: u32, folder: &str) -> Result<(), ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(uid, folder, "moving to Archive");

        self.session.select(folder)?;
        self.session.uid_mv(uid.to_string(), "Archive")?;
        Ok(())
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self, content), err)
    )]
    fn append(&mut self, folder: &str, content: &[u8]) -> Result<(), ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(folder, bytes = content.len(), "appending to folder");

        self.session.append(folder, content)?;

        #[cfg(feature = "tracing")]
        tracing::trace!("append successful");

        Ok(())
    }
}

impl Drop for NativeImapClient {
    fn drop(&mut self) {
        let _ = self.session.logout();
    }
}

fn format_address(addr: &imap_proto::Address) -> String {
    let mailbox = addr
        .mailbox
        .map(|m| String::from_utf8_lossy(m).into_owned())
        .unwrap_or_default();
    let host = addr
        .host
        .map(|h| String::from_utf8_lossy(h).into_owned())
        .unwrap_or_default();

    let email = if mailbox.is_empty() && host.is_empty() {
        String::new()
    } else {
        format!("{mailbox}@{host}")
    };

    if let Some(name) = addr.name {
        let name = String::from_utf8_lossy(name);
        if !name.is_empty() && !email.is_empty() {
            return format!("{name} <{email}>");
        }
    }

    email
}

pub fn parse_references(raw: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let mut refs = Vec::new();
    let mut start = None;

    for (i, ch) in text.char_indices() {
        match ch {
            '<' => start = Some(i + 1),
            '>' => {
                if let Some(s) = start.take() {
                    let msg_id = text[s..i].trim().to_string();
                    if !msg_id.is_empty() {
                        refs.push(msg_id);
                    }
                }
            }
            _ => {}
        }
    }

    refs
}

pub fn extract_body_text(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);

    if let Some(mime_text) = extract_mime_text(&text) {
        return mime_text;
    }

    let trimmed = text.trim_start();

    // Raw HTML email (no MIME wrapper)
    if trimmed.starts_with("<!") || trimmed.starts_with("<html") || trimmed.starts_with("<HTML") {
        return strip_html_tags(&text);
    }

    let body = if text.contains("Content-Type:") {
        text.split_once("\r\n\r\n")
            .or_else(|| text.split_once("\n\n"))
            .map(|(_, body)| body.to_string())
            .unwrap_or_else(|| text.into_owned())
    } else {
        text.into_owned()
    };

    // If it looks like HTML, strip tags
    if body.contains('<') && body.contains('>') {
        strip_html_tags(&body)
    } else {
        body
    }
}

pub fn extract_snippet(raw: &[u8]) -> String {
    let text = String::from_utf8_lossy(raw);

    let body = if let Some(mime_text) = extract_mime_text(&text) {
        mime_text
    } else {
        let trimmed = text.trim_start();
        if trimmed.starts_with("<!") || trimmed.starts_with("<html") || trimmed.starts_with("<HTML")
        {
            text.into_owned()
        } else if text.contains("Content-Type:") {
            // Single-part with MIME headers — skip to body after blank line
            text.split_once("\r\n\r\n")
                .or_else(|| text.split_once("\n\n"))
                .map(|(_, body)| body.to_string())
                .unwrap_or_else(|| text.into_owned())
        } else {
            text.into_owned()
        }
    };

    let stripped = strip_html_tags(&body);

    let collapsed: String = stripped.split_whitespace().collect::<Vec<_>>().join(" ");

    truncate_at_word_boundary(&collapsed, 100)
}

fn extract_mime_text(text: &str) -> Option<String> {
    // Look for a multipart boundary from a Content-Type header in the text
    let boundary = text
        .lines()
        .find(|line| line.contains("boundary="))
        .and_then(|line| {
            let start = line.find("boundary=")?;
            let rest = &line[start + 9..];
            let boundary = rest.trim_matches('"').trim_matches(';').trim();
            Some(boundary.to_string())
        })
        // For top-level multipart, BODY[TEXT] starts with --boundary
        .or_else(|| {
            let first_line = text.lines().next()?.trim_end();
            if first_line.starts_with("--") && first_line.len() > 2 {
                Some(first_line[2..].to_string())
            } else {
                None
            }
        });

    let boundary = boundary?;

    let parts: Vec<&str> = text.split(&format!("--{boundary}")).collect();

    let mut plain_text = None;
    let mut html_text = None;

    for part in &parts {
        let lower = part.to_lowercase();
        if (lower.contains("content-type: text/plain") || lower.contains("content-type:text/plain"))
            && let Some(body) = extract_part_body(part)
        {
            plain_text = Some(body.to_string());
        } else if (lower.contains("content-type: text/html")
            || lower.contains("content-type:text/html"))
            && let Some(body) = extract_part_body(part)
        {
            html_text = Some(strip_html_tags(body));
        }
    }

    // Prefer plaintext, fall back to stripped HTML
    plain_text.or(html_text)
}

fn extract_part_body(part: &str) -> Option<&str> {
    if let Some(start) = part.find("\r\n\r\n") {
        Some(&part[start + 4..])
    } else if let Some(start) = part.find("\n\n") {
        Some(&part[start + 2..])
    } else {
        None
    }
}

fn strip_html_tags(input: &str) -> String {
    // First pass: remove <style>...</style> and <script>...</script> blocks
    let mut text = input.to_string();
    for tag in &["style", "script"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        loop {
            let lower = text.to_lowercase();
            let Some(start) = lower.find(&open) else {
                break;
            };
            if let Some(end_offset) = lower[start..].find(&close) {
                let end = start + end_offset + close.len();
                text = format!("{}{}", &text[..start], &text[end..]);
            } else {
                text.truncate(start);
                break;
            }
        }
    }

    // Second pass: strip remaining HTML tags
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Decode common HTML entities
    result = result.replace("&nbsp;", " ");
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");
    result = result.replace("&apos;", "'");

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
