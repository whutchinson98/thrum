use super::*;

#[test]
fn parse_config() {
    let toml = r#"
[imap]
host = "imap.example.com"
port = 993
user = "me@example.com"
pass = "hunter2"
folder = "INBOX"

[smtp]
url = "smtp://smtp.example.com:587"
authentication = "plain"
"#;

    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.imap.host, "imap.example.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.user, "me@example.com");
    assert_eq!(config.imap.pass, "hunter2");
    assert_eq!(config.imap.folder, "INBOX");
    assert_eq!(config.smtp.url, "smtp://smtp.example.com:587");
    assert_eq!(config.smtp.authentication, "plain");
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
folder = "INBOX"

[smtp]
url = "smtp://localhost"
authentication = "login"
"#,
    )
    .unwrap();

    let config = load(Some(path)).unwrap();
    assert_eq!(config.imap.pass, "s3cret");

    std::fs::remove_dir_all(&dir).ok();
}
