use super::*;

#[test]
fn parse_config() {
    let toml = r#"
[imap]
host = "imap.example.com"
port = 993
user = "me@example.com"
pass = "hunter2"
folders = ["INBOX"]

[smtp]
host = "smtp.example.com"
port = 587
user = "me@example.com"
pass = "hunter2"

[sender]
from = "me@example.com"
name = "Me"
"#;

    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.imap.host, "imap.example.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.user, "me@example.com");
    assert_eq!(config.imap.pass, "hunter2");
    assert_eq!(config.imap.folders, vec!["INBOX".to_string()]);
    assert_eq!(config.imap.sent_folder, None);
    assert_eq!(config.smtp.host, "smtp.example.com");
    assert_eq!(config.smtp.port, 587);
    assert_eq!(config.smtp.user, "me@example.com");
    assert_eq!(config.smtp.pass, "hunter2");
    assert_eq!(config.sender.from, "me@example.com");
    assert_eq!(config.sender.name.as_deref(), Some("Me"));
    assert_eq!(config.sender.formatted_from(), "Me <me@example.com>");
}

#[test]
fn parse_config_with_sent_folder() {
    let toml = r#"
[imap]
host = "imap.example.com"
port = 993
user = "me@example.com"
pass = "hunter2"
folders = ["INBOX"]
sent_folder = "Sent"

[smtp]
host = "smtp.example.com"
port = 587
user = "me@example.com"
pass = "hunter2"

[sender]
from = "me@example.com"
"#;

    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.imap.sent_folder.as_deref(), Some("Sent"));
}

#[test]
fn expand_plain_value() {
    let result = expand_command("plaintext").unwrap();
    assert_eq!(result, "plaintext");
}

#[test]
fn expand_backtick_command() {
    let result = expand_command("`echo secret123`").unwrap();
    assert_eq!(result, "secret123");
}

#[test]
fn expand_backtick_preserves_inner_whitespace() {
    let result = expand_command("`echo hello`").unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn expand_command_failure() {
    let result = expand_command("`false`");
    assert!(result.is_err());
}

#[test]
fn default_config_path_contains_thrum() {
    let path = default_config_path().unwrap();
    assert!(path.ends_with("thrum.toml"));
}

#[test]
fn load_from_file() {
    let dir = std::env::temp_dir().join("thrum_test_load");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("thrum.toml");
    std::fs::write(
        &path,
        r#"
[imap]
host = "imap.localhost"
port = 993
user = "u"
pass = "`echo s3cret`"
folders = ["INBOX"]

[smtp]
host = "smtp.localhost"
port = 587
user = "u"
pass = "`echo sm7p`"

[sender]
from = "u@localhost"
"#,
    )
    .unwrap();

    let config = load(Some(path)).unwrap();
    assert_eq!(config.imap.pass, "s3cret");
    assert_eq!(config.smtp.pass, "sm7p");
    assert_eq!(config.sender.from, "u@localhost");
    assert_eq!(config.sender.name, None);
    assert_eq!(config.sender.formatted_from(), "u@localhost");

    std::fs::remove_dir_all(&dir).ok();
}
