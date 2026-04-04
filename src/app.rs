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
    pub diff_cache: HashMap<String, Vec<DiffHunk>>,
    pub diff_scroll: u16,
    pub diff_total_lines: u16,
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
            diff_total_lines: 0,
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

    pub fn toggle_filter(&mut self) {
        self.filter = self.filter.cycle();
        let len = self.filtered_files().len();
        if self.selected >= len {
            self.selected = len.saturating_sub(1);
        }
    }

    pub fn toggle_reviewed(&mut self) {
        if let Some(file) = self.selected_file() {
            let path = file.path.clone();
            self.review_state.toggle_reviewed(&path);
        }
    }

    pub fn mark_reviewed(&mut self, path: &str) {
        self.review_state.mark_reviewed(path);
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

            // +1 per hunk for the header line, +1 between hunks for spacing
            let total: usize = hunks.iter().map(|h| h.lines.len() + 1).sum::<usize>()
                + hunks.len().saturating_sub(1);
            self.diff_total_lines = total as u16;

            self.diff_cache.insert(path.clone(), hunks);
        }
    }

    pub fn scroll_diff(&mut self, delta: i16) {
        if delta > 0 {
            self.diff_scroll = self
                .diff_scroll
                .saturating_add(delta as u16)
                .min(self.diff_total_lines.saturating_sub(1));
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

    let review_state = ReviewState::load(&repo_path, &repo_info.branch)?;

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
