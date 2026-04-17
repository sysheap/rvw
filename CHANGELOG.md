# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Drop the OpenSSL / libssh2 system dependency by disabling `git2`'s default features and vendoring `libgit2`. Installing no longer requires `libssl-dev` / `openssl-devel`.

## [0.3.0] - 2026-04-09

### Added
- Auto-demote previously reviewed files back to Pending when their diff has changed since you last reviewed them, so iterative agent edits can't slip past unnoticed (d5708b9)

## [0.2.0] - 2026-04-09

### Changed
- Reframed README around the "LSP-in-code-review" pain point (5be3cae)
- Keep next files visible when scrolling near the bottom of the file list (96b0eb6)
- Auto-advance selection to the next file after reviewing one (50e09bf)

### Fixed
- Quiet spurious LSP diagnostics (50e09bf)

### Added
- `rust-toolchain.toml` for reproducible builds (ff40dbe)

[Unreleased]: https://github.com/sysheap/rvw/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/sysheap/rvw/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/sysheap/rvw/compare/v0.1.0...v0.2.0
