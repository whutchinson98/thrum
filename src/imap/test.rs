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

#[test]
fn extract_body_text_prefers_plain_in_multipart() {
    // Simulates BODY[TEXT] for a top-level multipart/alternative message
    let input = b"--boundary123\r\n\
        Content-Type: text/plain; charset=\"UTF-8\"\r\n\
        \r\n\
        Hello plain world\r\n\
        --boundary123\r\n\
        Content-Type: text/html; charset=\"UTF-8\"\r\n\
        \r\n\
        <html><body><p>Hello HTML world</p></body></html>\r\n\
        --boundary123--";
    let result = extract_body_text(input);
    assert!(
        result.contains("Hello plain world"),
        "Should prefer plain text, got: {result}"
    );
    assert!(
        !result.contains("HTML"),
        "Should not contain HTML content, got: {result}"
    );
}

#[test]
fn extract_body_text_falls_back_to_html_when_no_plain() {
    let input = b"--boundary456\r\n\
        Content-Type: text/html; charset=\"UTF-8\"\r\n\
        \r\n\
        <html><body><p>Only HTML here</p></body></html>\r\n\
        --boundary456--";
    let result = extract_body_text(input);
    assert!(
        result.contains("Only HTML here"),
        "Should strip HTML and return text, got: {result}"
    );
    assert!(
        !result.contains("<html>"),
        "Should have stripped HTML tags, got: {result}"
    );
}

#[test]
fn extract_body_text_plain_text_passthrough() {
    let input = b"Just a simple plain text email.";
    let result = extract_body_text(input);
    assert_eq!(result, "Just a simple plain text email.");
}

#[test]
fn extract_body_text_with_boundary_in_content_type_header() {
    // When BODY[TEXT] includes a nested multipart with boundary= in a Content-Type header
    let input = b"Content-Type: multipart/alternative; boundary=\"inner\"\r\n\
        \r\n\
        --inner\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        Plain text body\r\n\
        --inner\r\n\
        Content-Type: text/html\r\n\
        \r\n\
        <b>HTML body</b>\r\n\
        --inner--";
    let result = extract_body_text(input);
    assert!(
        result.contains("Plain text body"),
        "Should extract plain text, got: {result}"
    );
}

#[test]
fn extract_snippet_prefers_plain_in_multipart() {
    let input = b"--snipbound\r\n\
        Content-Type: text/plain\r\n\
        \r\n\
        Snippet plain text\r\n\
        --snipbound\r\n\
        Content-Type: text/html\r\n\
        \r\n\
        <p>Snippet HTML</p>\r\n\
        --snipbound--";
    let result = extract_snippet(input);
    assert!(
        result.contains("Snippet plain text"),
        "Snippet should prefer plain text, got: {result}"
    );
}
