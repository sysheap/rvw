# rvw

Review git branches in your editor, with full LSP support. A terminal code review tool for people who hate reviewing code inside GitHub or GitLab.

![rvw screenshot](https://raw.githubusercontent.com/sysheap/rvw/main/assets/screenshot.png)

## The Problem

Reviewing a pull request inside GitHub or GitLab means staring at a static diff. You can't jump to a definition, find references, hover for a type, or use any of the LSP features you rely on every day to actually *understand* code. The diff view is a read-only island, cut off from the tools that make code legible.

`rvw` brings code review back into your editor.

## What rvw Does

`rvw` gives you a TUI that lists all files changed between your current branch and a base branch (auto-detected, or pass `--base`). Pick a file and it opens in your editor at the first changed line — your editor, your LSP, your keybindings. Jump to definitions, find references, hover for types, do everything you'd normally do while writing code.

An integrated LSP server marks the changed regions inside the file so you can step between them with `]d`/`[d`, and hover any changed line to see what it looked like on the base branch.

Review state is persisted across sessions: you see which files you've already looked at and which still need attention.

### Inline comments as review comments

Because you're reviewing inside your editor, leaving an inline comment on a change is just… writing a comment in the code. No separate review UI, no context switching — the same shortcuts and snippets you already use for writing code work for writing review notes. Commit them on a review branch, push, and the author sees them exactly where they live.

## Editor Support

**rvw only works with [Helix](https://helix-editor.com/).** This is the editor I use and I built it specifically for that workflow. It automatically generates a `.helix/languages.toml` that registers the review LSP alongside your existing language servers, and cleans it up when you exit.

Other editors could theoretically work for basic file opening (the `--editor` flag and `$EDITOR` are supported), but the LSP integration that powers change navigation and base-branch hover is helix-specific.

## Usage

```bash
# Review current branch against main (auto-detected)
rvw

# Review against a specific base branch
rvw --base develop

# Use a different repository path
rvw --repo /path/to/repo
```

### TUI Keybindings

| Key | Action |
|-----|--------|
| `j`/`k` | Navigate file list |
| `Enter` | Open file at first change |
| `1`-`9` | Open file at specific hunk |
| `r` | Toggle reviewed status |
| `f` | Filter (all / pending) |
| `g`/`G` | Jump to top / bottom |
| `q` | Quit |

### In Helix

| Key | Action |
|-----|--------|
| `]d`/`[d` | Jump to next/previous change |
| Hover | See old code from base branch |

## Installation

Requires Rust 1.85+ and OpenSSL development headers (`openssl-devel` on Fedora, `libssl-dev` on Ubuntu/Debian).

```bash
cargo install rvw
```

## Disclaimer

This tool was fully coded by an LLM (Claude). I have not reviewed the source code — which is somewhat ironic given what the tool does.

## License

MIT — see [LICENSE](LICENSE).
