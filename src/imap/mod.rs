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
        let tls = TlsConnector::builder().build()?;
        let addr = (config.host.as_str(), config.port);

        let client = if ssl.starttls == "yes" {
            imap::connect_starttls(addr, &config.host, &tls)?
        } else {
            imap::connect(addr, &config.host, &tls)?
        };

        let session = client
            .login(&config.user, &config.pass)
            .map_err(|(e, _)| e)?;

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
        self.session.select(&self.folder)?;

        let messages = self.session.fetch("1:*", "(UID ENVELOPE)")?;
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

        self.session.logout()?;
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
