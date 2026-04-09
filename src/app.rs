use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::git::{self, ChangedFile, DiffHunk, RepoInfo};
use crate::review::ReviewState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    Pending,
}

impl FilterMode {
    pub fn cycle(self) -> Self {
        match self {
            FilterMode::All => FilterMode::Pending,
            FilterMode::Pending => FilterMode::All,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FilterMode::All => "all",
            FilterMode::Pending => "pending",
        }
    }
}

pub struct App {
    pub repo_info: RepoInfo,
    pub review_state: ReviewState,
    pub selected: usize,
    pub filter: FilterMode,
    pub editor_cmd: String,
    pub should_quit: bool,
    diff_cache: HashMap<String, Vec<DiffHunk>>,
    pub diff_scroll: u16,
    last_diff_file: Option<String>,
}

impl App {
    pub fn new(repo_info: RepoInfo, review_state: ReviewState, editor_cmd: String) -> Self {
        Self {
            repo_info,
            review_state,
            selected: 0,
            filter: FilterMode::All,
            editor_cmd,
            should_quit: false,
            diff_cache: HashMap::new(),
            diff_scroll: 0,
            last_diff_file: None,
        }
    }

    pub fn filtered_files(&self) -> Vec<&ChangedFile> {
        self.repo_info
            .files
            .iter()
            .filter(|f| match self.filter {
                FilterMode::All => true,
                FilterMode::Pending => !self.review_state.is_reviewed(&f.path),
            })
            .collect()
    }

    pub fn selected_file(&self) -> Option<&ChangedFile> {
        let files = self.filtered_files();
        files.get(self.selected).copied()
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = self.filtered_files().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        if delta > 0 {
            self.selected = (self.selected + delta as usize).min(len - 1);
        } else {
            let abs = (-delta) as usize;
            self.selected = self.selected.saturating_sub(abs);
        }
    }

    /// Advance the selection after the current file has been marked reviewed.
    ///
    /// In `FilterMode::All`, step forward by one (bounded to the last file) —
    /// the filtered list didn't shrink, so we need to actively move.
    ///
    /// In `FilterMode::Pending`, the just-reviewed file has already dropped
    /// out of the filtered list, so the selection index naturally points at
    /// what was the next pending file. We only need to clamp back into range
    /// if we were on the last entry.
    ///
    /// Must also handle the empty-list case (e.g., Pending mode after the
    /// final file was reviewed) — mirror `move_selection`'s behavior of
    /// resetting `selected` to `0`.
    pub fn advance_after_review(&mut self) {
        let len = self.filtered_files().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        match self.filter {
            FilterMode::All => {
                self.selected = (self.selected + 1).min(len - 1);
            }
            FilterMode::Pending => {
                if self.selected >= len {
                    self.selected = len - 1;
                }
            }
        }
    }

    pub fn toggle_filter(&mut self) {
        self.filter = self.filter.cycle();
        let len = self.filtered_files().len();
        if self.selected >= len {
            self.selected = len.saturating_sub(1);
        }
    }

    /// Toggle the reviewed state of the currently-selected file.
    ///
    /// Returns `true` if the file is now marked reviewed, `false` otherwise
    /// (including when no file is selected). Call sites use this to decide
    /// whether to auto-advance the selection — un-reviewing should stay put,
    /// marking-as-reviewed should advance.
    pub fn toggle_reviewed(&mut self) -> bool {
        if let Some(file) = self.selected_file() {
            let path = file.path.clone();
            let signature = file.signature.clone();
            self.review_state.toggle_reviewed(&path, signature);
            self.review_state.is_reviewed(&path)
        } else {
            false
        }
    }

    pub fn mark_reviewed(&mut self, path: &str) {
        let Some(signature) = self
            .repo_info
            .files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.signature.clone())
        else {
            return;
        };
        self.review_state.mark_reviewed(path, signature);
    }

    pub fn reviewed_count(&self) -> usize {
        self.repo_info
            .files
            .iter()
            .filter(|f| self.review_state.is_reviewed(&f.path))
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.repo_info.files.len()
    }

    pub fn ensure_diff_loaded(&mut self) {
        let current_path = self.selected_file().map(|f| f.path.clone());

        if current_path != self.last_diff_file {
            self.diff_scroll = 0;
            self.last_diff_file = current_path.clone();
        }

        if let Some(ref path) = current_path
            && !self.diff_cache.contains_key(path)
        {
            let hunks = git::diff_hunks_for_file(
                &self.repo_info.repo_path,
                &self.repo_info.base_branch,
                path,
            )
            .unwrap_or_default();

            self.diff_cache.insert(path.clone(), hunks);
        }
    }

    pub fn current_diff_hunks(&self) -> Option<&[DiffHunk]> {
        let path = &self.selected_file()?.path;
        self.diff_cache.get(path).map(|v| v.as_slice())
    }

    fn diff_display_lines(&self) -> u16 {
        match self.current_diff_hunks() {
            Some(hunks) => {
                let total: usize = hunks.iter().map(|h| h.lines.len() + 1).sum::<usize>()
                    + hunks.len().saturating_sub(1);
                total as u16
            }
            None => 0,
        }
    }

    pub fn scroll_diff(&mut self, delta: i16) {
        let total = self.diff_display_lines();
        if delta > 0 {
            self.diff_scroll = self
                .diff_scroll
                .saturating_add(delta as u16)
                .min(total.saturating_sub(1));
        } else {
            self.diff_scroll = self.diff_scroll.saturating_sub((-delta) as u16);
        }
    }
}

pub async fn run(repo_path: PathBuf, base: Option<&str>, editor: Option<&str>) -> Result<()> {
    let repo_info = git::analyze_repo(&repo_path, base)?;

    if repo_info.files.is_empty() {
        println!(
            "No changes between '{}' and '{}'.",
            repo_info.base_branch, repo_info.branch
        );
        return Ok(());
    }

    let mut review_state = ReviewState::load(&repo_path, &repo_info.branch)?;

    let current_signatures: HashMap<String, String> = repo_info
        .files
        .iter()
        .map(|f| (f.path.clone(), f.signature.clone()))
        .collect();
    review_state.invalidate_stale(&current_signatures);

    let editor_cmd = editor
        .map(String::from)
        .or_else(|| std::env::var("EDITOR").ok())
        .unwrap_or_else(|| "hx".to_string());

    let mut app = App::new(repo_info, review_state, editor_cmd);

    // Set up helix config
    let helix_config = crate::editor::HelixConfig::new(&app.repo_info)?;
    helix_config.install()?;

    let result = crate::ui::run_tui(&mut app).await;

    // Cleanup helix config
    helix_config.uninstall()?;

    // Save review state
    app.review_state.save()?;

    result
}
