use native_tls::TlsConnector;

use crate::config::{ImapConfig, SslConfig};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq)]
pub struct EmailSummary {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: String,
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
        tracing::instrument(level = tracing::Level::TRACE, skip(config, ssl))
    )]
    pub fn connect(config: &ImapConfig, ssl: &SslConfig) -> Result<Self, ImapError> {
        #[cfg(feature = "tracing")]
        tracing::trace!("building TLS connector");
        let tls = TlsConnector::builder().build()?;
        #[cfg(feature = "tracing")]
        tracing::trace!("TLS connector built");

        let addr = (config.host.as_str(), config.port);

        #[cfg(feature = "tracing")]
        tracing::trace!(host = config.host, port = config.port, starttls = %ssl.starttls, "connecting to IMAP server");
        let client = if ssl.starttls == "yes" {
            imap::connect_starttls(addr, &config.host, &tls)?
        } else {
            imap::connect(addr, &config.host, &tls)?
        };
        #[cfg(feature = "tracing")]
        tracing::trace!("TCP/TLS connection established");

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
        let messages = self.session.fetch("1:*", "(UID ENVELOPE)")?;
        #[cfg(feature = "tracing")]
        tracing::trace!(raw_count = messages.len(), "messages fetched from server");

        let mut emails = Vec::new();

        for fetch in messages.iter() {
            let uid = fetch.uid.unwrap_or(0);
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
