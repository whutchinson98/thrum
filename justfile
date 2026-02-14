export RUSTFLAGS := "-Dwarnings"
export RUSTDOCFLAGS := "-Dwarnings"
export CARGO_TERM_COLOR := "always"

clippy *ARGS:
  cargo clippy {{ ARGS }}

fmt *ARGS:
  cargo fmt {{ ARGS }}

check:
  cargo check

test:
  cargo test

build *ARGS:
  cargo build {{ ARGS }}
