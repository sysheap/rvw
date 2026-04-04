# rvw

A terminal code review tool for reviewing agent-produced code on git branches.

![rvw screenshot](https://raw.githubusercontent.com/sysheap/rvw/main/assets/screenshot.png)

## The Problem

Coding agents produce code on feature branches, often across multiple commits. Reviewing this with `git diff` is limiting — you can't see full file context and you can't use LSP to navigate the codebase.

## What rvw Does

`rvw` gives you a TUI that shows all files changed between your current branch and main. Select a file and it opens in your editor at the first changed line. An integrated LSP server marks changed regions so you can jump between them with `]d`/`[d` and hover to see what the code looked like before.

Review state is tracked across sessions. You see which files you've already looked at and which still need attention.

## Editor Support

**rvw only works with [Helix](https://helix-editor.com/).** This is the editor I use and I built it specifically for that workflow. It automatically generates a `.helix/languages.toml` that registers the review LSP alongside your existing language servers, and cleans it up when you exit.

Other editors could theoretically work for basic file opening (the `--editor` flag and `$EDITOR` are supported), but the LSP integration — which is the core value — is helix-specific.

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
