# Codebase Index: rvw

> Generated: 2026-04-09 18:11:26 UTC | Files: 14 | Lines: 2455
> Languages: Markdown (3), Rust (10), TOML (1)

## Directory Structure

```
rvw/
  CLAUDE.md
  Cargo.toml
  INDEX.md
  README.md
  src/
    app.rs
    editor.rs
    git.rs
    input.rs
    languages.rs
    lsp/
      diff.rs
      mod.rs
    main.rs
    review.rs
    ui.rs
```

---

## Public API Surface

**CLAUDE.md**
- `# CLAUDE.md`

**Cargo.toml**
- `[package]`
- `[dependencies]`

**INDEX.md**
- `# Codebase Index: rvw`

**README.md**
- `# rvw`
- `# Review current branch against main (auto-detected)`
- `# Review against a specific base branch`
- `# Use a different repository path`

**src/app.rs**
- `pub enum FilterMode`
- `pub struct App`
- `pub async fn run(repo_path: PathBuf, base: Option<&str>, editor: Option<&str>) -> Result<()>`

**src/editor.rs**
- `pub fn open_editor(editor_cmd: &str, repo_path: &Path, file_path: &str, line: u32) -> Result<()>`
- `pub struct HelixConfig`

**src/git.rs**
- `pub struct ChangedFile`
- `pub enum FileStatus`
- `pub struct Hunk`
- `pub struct RepoInfo`
- `pub fn detect_base_branch(repo: &Repository) -> Result<String>`
- `pub fn current_branch(repo: &Repository) -> Result<String>`
- `pub fn analyze_repo(repo_path: &Path, base_override: Option<&str>) -> Result<RepoInfo>`
- `pub fn get_base_file_content( repo_path: &Path, base_branch: &str, file_path: &str, ) -> Result<String>`
- `pub fn diff_hunks_for_file( repo_path: &Path, base_branch: &str, file_path: &str, ) -> Result<Vec<DiffHunk>>`
- `pub enum DiffLineKind`
- `pub struct DiffLine`
- `pub struct DiffHunk`

**src/input.rs**
- `pub enum Action`
- `pub fn handle_key(key: KeyEvent, app: &mut App) -> Action`

**src/languages.rs**
- `pub struct LanguageInfo`
- `pub fn language_for_path(path: &Path) -> Option<LanguageInfo>`
- `pub fn language_for_extension(ext: &str) -> Option<LanguageInfo>`

**src/lsp/diff.rs**
- `pub fn hunks_to_diagnostics(hunks: &[DiffHunk]) -> Vec<Diagnostic>`

**src/lsp/mod.rs**
- `pub mod diff`
- `pub async fn run_lsp_server(repo_path: PathBuf, base_branch: String) -> Result<()>`

**src/review.rs**
- `pub struct ReviewState`
- `pub struct FileReviewState`
- `pub enum ReviewStatus`

**src/ui.rs**
- `pub async fn run_tui(app: &mut App) -> Result<()>`

---

## CLAUDE.md

**Language:** Markdown | **Size:** 7.0 KB | **Lines:** 106

**Declarations:**

---

## Cargo.toml

**Language:** TOML | **Size:** 854 B | **Lines:** 29

**Imports:**
- `ratatui`
- `crossterm`
- `tower-lsp`
- `tokio`
- `git2`
- `clap`
- `serde`
- `serde_json`
- `toml`
- `dirs`
- *... and 4 more imports*

**Declarations:**

---

## INDEX.md

**Language:** Markdown | **Size:** 8.1 KB | **Lines:** 405

**Declarations:**

---

## README.md

**Language:** Markdown | **Size:** 2.3 KB | **Lines:** 69

**Declarations:**

---

## src/app.rs

**Language:** Rust | **Size:** 7.0 KB | **Lines:** 242

**Imports:**
- `anyhow::Result`
- `std::collections::HashMap`
- `std::path::PathBuf`
- `crate::git::{self, ChangedFile, DiffHunk, RepoInfo}`
- `crate::review::ReviewState`

**Declarations:**

**`impl FilterMode`**
  `pub fn cycle(self) -> Self`

  `pub fn label(&self) -> &'static str`


**`impl App`**
  `pub fn new(repo_info: RepoInfo, review_state: ReviewState, editor_cmd: String) -> Self`

  `pub fn filtered_files(&self) -> Vec<&ChangedFile>`

  `pub fn selected_file(&self) -> Option<&ChangedFile>`

  `pub fn move_selection(&mut self, delta: i32)`

  `pub fn advance_after_review(&mut self)`

  `pub fn toggle_filter(&mut self)`

  `pub fn toggle_reviewed(&mut self) -> bool`

  `pub fn mark_reviewed(&mut self, path: &str)`

  `pub fn reviewed_count(&self) -> usize`

  `pub fn total_count(&self) -> usize`

  `pub fn ensure_diff_loaded(&mut self)`

  `pub fn current_diff_hunks(&self) -> Option<&[DiffHunk]>`

  `fn diff_display_lines(&self) -> u16`

  `pub fn scroll_diff(&mut self, delta: i16)`


---

## src/editor.rs

**Language:** Rust | **Size:** 11.1 KB | **Lines:** 316

**Imports:**
- `anyhow::{Context, Result}`
- `std::path::{Path, PathBuf}`
- `std::process::Command`
- `std::sync::Mutex`
- `crate::git::RepoInfo`
- `crate::languages`

**Declarations:**

`static CLEANUP_STATE: Mutex<Option<CleanupInfo>> = Mutex::new(None)`

`struct CleanupInfo`
> Fields: `config_path: PathBuf`, `backup_path: PathBuf`, `had_existing: bool`, `helix_dir: PathBuf`

**`impl HelixConfig`**
  `pub fn new(repo_info: &RepoInfo) -> Result<Self>`

  `pub fn install(&self) -> Result<()>`

  `pub fn uninstall(&self) -> Result<()>`

  `fn generate_config(&self) -> Result<String>`

  `fn find_lang_info(&self, name: &str) -> Option<languages::LanguageInfo>`


`fn do_cleanup(info: &CleanupInfo)`

---

## src/git.rs

**Language:** Rust | **Size:** 10.5 KB | **Lines:** 369

**Imports:**
- `anyhow::{Context, Result, bail}`
- `git2::{Delta, DiffOptions, Repository}`
- `std::path::{Path, PathBuf}`

**Declarations:**

**`impl FileStatus`**
  `pub fn label(&self) -> &'static str`


`fn find_merge_base<'a>(repo: &'a Repository, base_branch: &str) -> Result<git2::Commit<'a>>`

`fn collect_changed_files(diff: &git2::Diff<'_>) -> Result<Vec<ChangedFile>>`

`fn collect_stats(diff: &git2::Diff<'_>) -> Result<Vec<(usize, usize)>>`

`fn collect_hunks_for_file(diff: &git2::Diff<'_>, file_index: usize) -> Result<Vec<Hunk>>`

**`impl DiffHunk`**
  `pub fn removed_lines(&self) -> impl Iterator<Item = (u32, &str)>`

  `pub fn added_lines(&self) -> impl Iterator<Item = (u32, &str)>`


---

## src/input.rs

**Language:** Rust | **Size:** 3.0 KB | **Lines:** 94

**Imports:**
- `crossterm::event::{KeyCode, KeyEvent, KeyModifiers}`
- `crate::app::App`

**Declarations:**

`const DIFF_PAGE_LINES: i16 = 15`

---

## src/languages.rs

**Language:** Rust | **Size:** 4.0 KB | **Lines:** 134

**Imports:**
- `std::path::Path`

**Declarations:**

---

## src/lsp/diff.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 43

**Imports:**
- `tower_lsp::lsp_types::*`
- `crate::git::DiffHunk`

**Declarations:**

---

## src/lsp/mod.rs

**Language:** Rust | **Size:** 4.8 KB | **Lines:** 147

**Imports:**
- `anyhow::Result`
- `std::path::PathBuf`
- `tower_lsp::jsonrpc`
- `tower_lsp::lsp_types::*`
- `tower_lsp::{Client, LanguageServer, LspService, Server}`
- `crate::git`

**Declarations:**

`struct Backend`
> Fields: `client: Client`, `repo_path: PathBuf`, `base_branch: String`

**`impl LanguageServer for Backend`**
  `async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult>`

  `async fn initialized(&self, _: InitializedParams)`

  `async fn shutdown(&self) -> jsonrpc::Result<()>`

  `async fn did_open(&self, params: DidOpenTextDocumentParams)`

  `async fn did_change(&self, params: DidChangeTextDocumentParams)`

  `async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>>`


**`impl Backend`**
  `fn uri_to_relative_path(&self, uri: &Url) -> Option<String>`

  `fn compute_diagnostics(&self, uri: &Url) -> Option<Vec<Diagnostic>>`


---

## src/main.rs

**Language:** Rust | **Size:** 1.4 KB | **Lines:** 66

**Imports:**
- `anyhow::Result`
- `clap::{Parser, Subcommand}`
- `std::path::PathBuf`

**Declarations:**

`mod app`

`mod editor`

`mod git`

`mod input`

`mod languages`

`mod lsp`

`mod review`

`mod ui`

`struct Cli`
> Fields: `base: Option<String>`, `editor: Option<String>`, `repo: Option<PathBuf>`, `command: Option<Commands>`

`enum Commands`
> Variants: `Lsp`

`async fn main() -> Result<()>`

---

## src/review.rs

**Language:** Rust | **Size:** 3.4 KB | **Lines:** 125

**Imports:**
- `anyhow::{Context, Result}`
- `chrono::{DateTime, Utc}`
- `serde::{Deserialize, Serialize}`
- `sha2::{Digest, Sha256}`
- `std::collections::HashMap`
- `std::path::{Path, PathBuf}`

**Declarations:**

**`impl ReviewState`**
  `pub fn load(repo_path: &Path, branch: &str) -> Result<Self>`

  `pub fn save(&self) -> Result<()>`

  `pub fn is_reviewed(&self, path: &str) -> bool`

  `pub fn mark_reviewed(&mut self, path: &str)`

  `pub fn mark_pending(&mut self, path: &str)`

  `pub fn toggle_reviewed(&mut self, path: &str)`


`fn state_file_path(repo_path: &Path, branch: &str) -> Result<PathBuf>`

---

## src/ui.rs

**Language:** Rust | **Size:** 9.8 KB | **Lines:** 310

**Imports:**
- `anyhow::Result`
- `crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
}`
- `ratatui::{
    Frame,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
}`
- `std::io`
- `crate::app::App`
- `crate::git::DiffLineKind`
- `crate::input`

**Declarations:**

`const COLOR_HEADER: Color = Color::Cyan`

`const COLOR_FILTER: Color = Color::Yellow`

`const COLOR_HUNK_HEADER: Color = Color::Cyan`

`const COLOR_CONTEXT: Color = Color::Reset`

`const COLOR_ADDED: Color = Color::Green`

`const COLOR_REMOVED: Color = Color::Red`

`const COLOR_DIFF_TITLE: Color = Color::Reset`

`const COLOR_HELP_KEY: Color = Color::Green`

`fn render(f: &mut Frame, app: &App)`

`fn render_header(f: &mut Frame, app: &App, area: Rect)`

`fn render_file_list(f: &mut Frame, app: &App, area: Rect)`

`fn render_diff_preview(f: &mut Frame, app: &App, area: Rect)`

`fn render_help(f: &mut Frame, area: Rect)`

