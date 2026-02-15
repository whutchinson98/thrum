use super::*;

#[test]
fn format_address_with_name() {
    let addr = imap_proto::Address {
        name: Some(b"Alice Smith"),
        adl: None,
        mailbox: Some(b"alice"),
        host: Some(b"example.com"),
    };
    assert_eq!(format_address(&addr), "Alice Smith");
}

#[test]
fn format_address_without_name() {
    let addr = imap_proto::Address {
        name: None,
        adl: None,
        mailbox: Some(b"bob"),
        host: Some(b"example.com"),
    };
    assert_eq!(format_address(&addr), "bob@example.com");
}

#[test]
fn format_address_empty_name_falls_back() {
    let addr = imap_proto::Address {
        name: Some(b""),
        adl: None,
        mailbox: Some(b"carol"),
        host: Some(b"example.com"),
    };
    assert_eq!(format_address(&addr), "carol@example.com");
}

#[test]
fn format_address_empty_fields() {
    let addr = imap_proto::Address {
        name: None,
        adl: None,
        mailbox: None,
        host: None,
    };
    assert_eq!(format_address(&addr), "");
}

#[test]
fn mock_client_returns_emails() {
    let mut mock = MockImapClient::new();
    mock.expect_fetch_inbox().returning(|| {
        Ok(vec![EmailSummary {
            uid: 1,
            subject: "Hello".to_string(),
            from: "alice@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: false,
            snippet: "Hey there".to_string(),
        }])
    });

    let emails = mock.fetch_inbox().unwrap();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].subject, "Hello");
}

#[test]
fn mock_client_returns_empty() {
    let mut mock = MockImapClient::new();
    mock.expect_fetch_inbox().returning(|| Ok(vec![]));

    let emails = mock.fetch_inbox().unwrap();
    assert!(emails.is_empty());
}

#[test]
fn extract_snippet_plain_text() {
    let input = b"Hello, this is a plain text email body.";
    let result = extract_snippet(input);
    assert_eq!(result, "Hello, this is a plain text email body.");
}

#[test]
fn extract_snippet_truncates() {
    let long = "word ".repeat(50);
    let result = extract_snippet(long.as_bytes());
    assert!(result.len() <= 110);
    assert!(result.ends_with("..."));
}

#[test]
fn extract_snippet_strips_html() {
    let input = b"<p>Hello <b>world</b></p>";
    let result = extract_snippet(input);
    assert_eq!(result, "Hello world");
}

#[test]
fn extract_snippet_empty() {
    let result = extract_snippet(b"");
    assert_eq!(result, "");
}
