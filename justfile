export RUSTFLAGS := "-Dwarnings"
export RUSTDOCFLAGS := "-Dwarnings"
export CARGO_TERM_COLOR := "always"

clippy *ARGS:
  cargo clippy --all-features {{ ARGS }}

fmt *ARGS:
  cargo fmt {{ ARGS }}

check:
  cargo check --all-features

test:
  cargo test --all-features

build *ARGS:
  cargo build {{ ARGS }}
