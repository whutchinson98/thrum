# CLAUDE.md

## Project Overview

Thrum is a terminal email client (TUI) written in Rust (edition 2024). It uses [ratatui](https://ratatui.rs/) for terminal UI components.

## Version Control

This project uses **jj** (Jujutsu) as the version control CLI with a git backend. Use `jj` commands instead of `git` for all VCS operations:

- `jj status` instead of `git status`
- `jj diff` instead of `git diff`
- `jj new` instead of `git commit` (jj auto-commits working copy changes)
- `jj describe -m "message"` to set the change description
- `jj log` instead of `git log`
- `jj bookmark` for branch-like workflows

## Build & Run

```sh
just build           # debug build
just check           # type-check without building
just test            # run all tests
just clippy          # lint
just fmt             # format
```

## Development Environment

The project uses a Nix flake (`flake.nix`) with direnv (`.envrc`) to provide the development toolchain (rustc, cargo, clippy, rustfmt, rust-analyzer, cargo-info, cargo-udeps, just).

## Project Structure

```
src/
  main.rs            # entrypoint
  <module>/
    mod.rs           # module implementation
    test.rs          # tests for that module
```

## Testing

Tests live in separate `test.rs` files alongside the module they test, not inline with `#[cfg(test)]` blocks in the implementation file. Each module should include its test file conditionally:

```rust
// in mod.rs
#[cfg(test)]
mod test;
```

And the corresponding `test.rs`:

```rust
use super::*;

#[test]
fn it_works() {
    // ...
}
```

## Tracing

Use the `tracing` crate with `tracing::instrument` for instrumentation. Default instrument level is `tracing::Level::TRACE`.

Tracing is gated behind a cargo feature flag called `tracing` so that logs are only included when explicitly enabled at build time:

```toml
[features]
tracing = ["dep:tracing"]

[dependencies]
tracing = { version = "0.1", optional = true }
```

To build with tracing enabled:

```sh
cargo run --features tracing
```

When annotating functions:

```rust
#[cfg_attr(feature = "tracing", tracing::instrument(level = tracing::Level::TRACE))]
fn my_function() {
    // ...
}
```

## UI Framework

The TUI is built with [ratatui](https://ratatui.rs/). Use ratatui idioms and patterns for rendering, layout, and event handling.

## Style Guidelines

- Keep code simple and focused; avoid over-engineering.
- Follow standard `cargo fmt` formatting.
- Use `cargo clippy` to catch common mistakes.
