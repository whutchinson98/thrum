use super::*;

#[test]
fn format_address_with_name() {
    let addr = imap_proto::Address {
        name: Some(b"Alice Smith"),
        adl: None,
        mailbox: Some(b"alice"),
        host: Some(b"example.com"),
    };
    assert_eq!(format_address(&addr), "Alice Smith <alice@example.com>");
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
            folder: "INBOX".to_string(),
            subject: "Hello".to_string(),
            from: "alice@example.com".to_string(),
            to: "me@example.com".to_string(),
            date: "2025-01-01".to_string(),
            seen: false,
            snippet: "Hey there".to_string(),
            message_id: None,
            in_reply_to: None,
            references: vec![],
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

#[test]
fn parse_references_single() {
    let input = b"References: <abc@example.com>\r\n";
    let refs = parse_references(input);
    assert_eq!(refs, vec!["abc@example.com"]);
}

#[test]
fn parse_references_multiple() {
    let input = b"References: <abc@example.com> <def@example.com> <ghi@example.com>\r\n";
    let refs = parse_references(input);
    assert_eq!(
        refs,
        vec!["abc@example.com", "def@example.com", "ghi@example.com"]
    );
}

#[test]
fn parse_references_empty() {
    let refs = parse_references(b"");
    assert!(refs.is_empty());
}

#[test]
fn parse_references_no_angle_brackets() {
    let refs = parse_references(b"References: no-brackets\r\n");
    assert!(refs.is_empty());
}
