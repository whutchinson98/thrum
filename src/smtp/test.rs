use super::*;

#[test]
fn mock_client_sends_email() {
    let mut mock = MockSmtpClient::new();
    mock.expect_send().returning(|_| Ok(()));

    let email = Email {
        from: "sender@example.com".to_string(),
        to: vec!["recipient@example.com".to_string()],
        subject: "Test".to_string(),
        body: "Hello".to_string(),
    };

    assert!(mock.send(&email).is_ok());
}

#[test]
fn mock_client_send_fails() {
    let mut mock = MockSmtpClient::new();
    mock.expect_send()
        .returning(|_| Err(SmtpError::Message(lettre::error::Error::MissingTo)));

    let email = Email {
        from: "sender@example.com".to_string(),
        to: vec![],
        subject: "Test".to_string(),
        body: "Hello".to_string(),
    };

    let result = mock.send(&email);
    assert!(result.is_err());
}

#[test]
fn email_struct_fields() {
    let email = Email {
        from: "alice@example.com".to_string(),
        to: vec![
            "bob@example.com".to_string(),
            "carol@example.com".to_string(),
        ],
        subject: "Greetings".to_string(),
        body: "Hi there".to_string(),
    };

    assert_eq!(email.from, "alice@example.com");
    assert_eq!(email.to.len(), 2);
    assert_eq!(email.subject, "Greetings");
    assert_eq!(email.body, "Hi there");
}
