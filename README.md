# rvw

A terminal code review tool for reviewing agent-produced code on git branches.

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

Requires Rust and OpenSSL development headers (`openssl-devel` on Fedora, `libssl-dev` on Ubuntu/Debian).

```bash
cargo install --path .
```

## Disclaimer

This tool was fully coded by an LLM (Claude). I have not reviewed the source code — which is somewhat ironic given what the tool does.

## License

MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
