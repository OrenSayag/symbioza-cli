# Repository Guidelines

## Project Structure & Module Organization

Codex CLI lives in this monorepo. The Rust workspace in `codex-rs/` supplies the runtime; each crate sits in its own folder (for example `codex-rs/core` builds `codex-core`) with integration fixtures under `tests/`. The JavaScript wrapper that publishes the npm binary is in `codex-cli/`. Reference docs and FAQs live in `docs/`, while automation and release helpers reside in `scripts/`.

## Build, Test, and Development Commands

- `cd codex-rs && cargo run --bin codex`: run the CLI in debug mode.
- `cd codex-rs && just fmt`: apply the shared rustfmt profile.
- `cd codex-rs && just fix -p codex-core`: scoped Clippy autofix for the crate you touched.
- `cd codex-rs && cargo test -p codex-tui`: run crate tests; add `-- --nocapture` for extra logs.
- `cd codex-rs && just test`: full suite via `cargo nextest --no-fail-fast`.
- `pnpm run format:fix`: Prettier for Markdown, JSON, and workflow files.

## Coding Style & Naming Conventions

Rust targets edition 2024; follow the repo `rustfmt.toml` and keep imports tidy. Crates and binaries always use the `codex-` prefix to match workspace metadata. Prefer inline `format!` interpolation (`format!("{value}")`) and leverage ratatui Stylize helpers outlined in `codex-rs/tui/styles.md`. JavaScript and Markdown code should pass Prettier, and shell scripts stay POSIX unless an existing shebang requires bash.

## Testing Guidelines

Run crate-specific suites with `cargo test -p <crate>` before invoking broader runs. TUI rendering relies on Insta snapshots: regenerate with `cargo test -p codex-tui`, inspect `cargo insta pending-snapshots -p codex-tui`, and accept after review. Tests should use `pretty_assertions::assert_eq` for clearer diffs. Place new integration fixtures under `codex-rs/cli/tests` and document required environment variables in `docs/`.

## Commit & Pull Request Guidelines

Work from topic branches such as `feat/short-label` or `fix/issue-123`. Write imperative, signed-off commits (`git commit -s`) so the DCO check succeeds, and keep every commit buildable. Before opening a PR, run formatting, Clippy, and the relevant tests, then explain what changed, why, and how to verify it. Link tracking issues and include CLI output or screenshots whenever behavior or UI shifts. Update documentation alongside any user-facing change.

## Security & Configuration Tips

Do not check in credentials; configuration lives in `~/.codex/config.toml`. Report security concerns to `security@openai.com`. When examples require tokens, prefer placeholder environment variables rather than real values.

# Repository Purpose

This repository is a fork of the original Codex CLI project, and we are renaming
it to "Symbioza". We will build upon this product to create a custom tool that
implements our features.
