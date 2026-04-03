# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build              # debug build
cargo build --release    # release build
cargo check              # fast type-check without codegen
cargo clippy             # linting
cargo test               # run tests (none yet)
```

Requires `openssl-devel` (Fedora) or `libssl-dev` (Ubuntu) for the `git2` crate.

## What This Is

`rvw` is a terminal code review tool for reviewing agent-produced code on git branches. Single binary, two modes:

- **`rvw`** (default) — TUI showing changed files relative to a base branch. Opens your editor at changed lines, tracks review progress across sessions.
- **`rvw lsp --base <branch> --repo <path>`** — LSP server mode, spawned by helix (not run directly). Publishes Information-level diagnostics at changed hunks (enabling `]d`/`[d` navigation) and hover showing old code from the base branch.

## Architecture

The binary dispatches in `main.rs` via clap: no subcommand runs the TUI (`app::run`), the `lsp` subcommand runs the LSP server (`lsp::run_lsp_server`). Both modes share `git.rs` for diff computation and `languages.rs` for file type detection.

### TUI mode flow
`app::run` → `git::analyze_repo` (computes diff via git2) → `editor::HelixConfig::install` (generates `.helix/languages.toml` with the review LSP merged in) → `ui::run_tui` (ratatui event loop) → on file select, suspends TUI and spawns editor → on exit, restores helix config and saves review state.

### LSP mode flow
`lsp::run_lsp_server` → tower-lsp server on stdin/stdout → on `didOpen`/`didChange`, calls `git::diff_hunks_for_file` and publishes diagnostics via `lsp::diff::hunks_to_diagnostics` → on hover, shows old code from the base branch.

### Key shared modules
- **`git.rs`** — All git operations via `git2`: merge-base detection, tree-to-tree diff, per-file hunk extraction. Both TUI (for file list) and LSP (for diagnostics/hover) use this.
- **`languages.rs`** — Maps file extensions to language name and default LSP server names (for helix config generation).
- **`review.rs`** — Persists review state to `~/.local/share/rvw/<repo-hash>-<branch>.json`. Keyed by SHA256 of repo path + branch name.

### Helix config integration (`editor.rs`)
`HelixConfig` manages `.helix/languages.toml` lifecycle: backs up existing config, parses it as TOML, appends the `rvw` language server to each language's server list (preserving existing servers), restores backup on exit. A `ctrlc` handler ensures cleanup on Ctrl+C. Stale backups from crashed sessions are auto-restored on next startup.

