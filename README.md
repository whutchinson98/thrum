# Thrum

A terminal email client (TUI) written in Rust.

## Configuration

Thrum reads its config from `~/.config/thrum.toml`. Create the file with your IMAP and SMTP credentials:

```toml
[imap]
host = "imap.example.com"
port = 993
user = "you@example.com"
pass = "your-password"
folders = ["INBOX"]
sent_folder = "Sent"       # optional

[smtp]
host = "smtp.example.com"
port = 587
user = "you@example.com"
pass = "your-password"

[sender]
from = "you@example.com"
name = "Your Name"         # optional
```

### Password commands

Instead of storing passwords in plain text, you can use a shell command wrapped in backticks. Thrum will execute the command and use its stdout as the password:

```toml
[imap]
host = "imap.gmail.com"
port = 993
user = "you@gmail.com"
pass = "`pass email/gmail`"
folders = ["INBOX"]
sent_folder = "Sent"

[smtp]
host = "smtp.gmail.com"
port = 587
user = "you@gmail.com"
pass = "`pass email/gmail`"

[sender]
from = "you@gmail.com"
name = "Your Name"
```

This works with any secret manager (`pass`, `op`, `gpg`, `security find-generic-password`, etc.).
