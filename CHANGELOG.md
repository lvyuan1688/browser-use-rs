# Changelog

All notable changes to browser-use-rs are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] — 2026-08-20

### Added
- `crates/browser-core` `Browser` trait + 3 backends (cdp/playwright/replay).
- `crates/agent-loop` Think/Act/Observe state machine.
- `crates/compress` DOM compression with depth pruning.
- `replay-ui` ratatui stepper for recorded sessions.
- `src/main.rs` CLI with `go` and `info` subcommands.
- `examples/replay.rs` end-to-end recorded session.
- `CONTRIBUTING.md`, Issue/PR templates.

## [0.1.3] — 2026-08-15

### Added
- `docs/v0.1.3-patch-notes.md`.

## [0.1.2] — 2026-08-13

### Added
- Stub `ReplayBackend` returning canned `ActionResult::Ok`.

## [0.1.1] — 2026-08-12

### Added
- Initial `Browser` trait draft.

## [0.1.0] — 2026-08-10

Initial public skeleton.
