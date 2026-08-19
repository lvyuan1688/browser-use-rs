# Contributing to browser-use-rs

Thanks for your interest! This is a community-driven, open-source browser-agent
project. Contributions of all sizes are welcome.

## Quick start

```bash
git clone https://github.com/lvyuan1688/browser-use-rs
cd browser-use-rs
cargo build
cargo test
```

The skeleton ships a `ReplayBackend` that returns canned data — you don't need
a running browser to exercise the agent loop.

## Ways to contribute

- **Bugs**: open an issue with OS, Rust version, command, and stack trace.
- **Backends**: add a new `Browser` implementation in
  `crates/browser-core/src/<backend>.rs`. Wire it into the CLI in
  `src/main.rs`.
- **Compression**: improve `crates/compress` so the DOM fed to the LLM is
  smaller but still actionable.
- **Replay UI**: extend `replay-ui` with scrubbing, search, and export.
- **Docs**: typos, clarifications, and new guides are all welcome.

## Pull request checklist

- [ ] `cargo fmt` is clean
- [ ] `cargo clippy -- -D warnings` is clean
- [ ] `cargo test` passes
- [ ] `CHANGELOG.md` updated (if user-visible)

## Code of conduct

Be kind. Personal attacks, harassment, or discriminatory behavior will not be
tolerated.

## License

By contributing, you agree your contributions are licensed under the MIT
license (see `LICENSE`).
