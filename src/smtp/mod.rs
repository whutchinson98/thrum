use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::config::SmtpConfig;

#[cfg(test)]
mod test;

pub struct Email {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error("message build error: {0}")]
    Message(#[from] lettre::error::Error),
    #[error("address parse error: {0}")]
    Address(#[from] lettre::address::AddressError),
}

#[cfg_attr(test, mockall::automock)]
pub trait SmtpClient {
    fn send(&self, email: &Email) -> Result<Vec<u8>, SmtpError>;
}

pub struct NativeSmtpClient {
    #[allow(dead_code)]
    transport: SmtpTransport,
}

impl NativeSmtpClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(config), err)
    )]
    pub fn connect(config: &SmtpConfig) -> Result<Self, SmtpError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(host = %config.host, port = config.port, "connecting to SMTP server");

        let creds = Credentials::new(config.user.clone(), config.pass.clone());
        #[cfg(feature = "tracing")]
        tracing::trace!(user = %config.user, "credentials created");

        #[cfg(feature = "tracing")]
        tracing::trace!("building SMTP transport");
        let transport = SmtpTransport::relay(&config.host)?
            .port(config.port)
            .credentials(creds)
            .build();

        #[cfg(feature = "tracing")]
        tracing::trace!("SMTP transport built");

        Ok(Self { transport })
    }
}

impl SmtpClient for NativeSmtpClient {
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = tracing::Level::TRACE, skip(self, email), err)
    )]
    fn send(&self, email: &Email) -> Result<Vec<u8>, SmtpError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(from = %email.from, to = ?email.to, subject = %email.subject, "building message");

        let from: Mailbox = email.from.parse()?;

        let mut builder = Message::builder().from(from).subject(&email.subject);

        for to_addr in &email.to {
            let mailbox: Mailbox = to_addr.parse()?;
            builder = builder.to(mailbox);
        }

        for cc_addr in &email.cc {
            let mailbox: Mailbox = cc_addr.parse()?;
            builder = builder.cc(mailbox);
        }

        for bcc_addr in &email.bcc {
            let mailbox: Mailbox = bcc_addr.parse()?;
            builder = builder.bcc(mailbox);
        }

        if let Some(ref reply_to) = email.in_reply_to {
            builder = builder.in_reply_to(reply_to.clone());
        }

        if !email.references.is_empty() {
            let refs_str = email
                .references
                .iter()
                .map(|r| format!("<{r}>"))
                .collect::<Vec<_>>()
                .join(" ");
            builder = builder.references(refs_str);
        }

        let message = builder.body(email.body.clone())?;
        let formatted = message.formatted();
        #[cfg(feature = "tracing")]
        tracing::trace!("message built");

        #[cfg(feature = "tracing")]
        tracing::trace!("sending email");

        self.transport.send(&message)?;

        #[cfg(feature = "tracing")]
        tracing::trace!("email sent");

        Ok(formatted)
    }
}
