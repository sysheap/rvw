use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewState {
    pub repo: String,
    pub branch: String,
    pub started_at: DateTime<Utc>,
    pub files: HashMap<String, FileReviewState>,
    #[serde(skip)]
    state_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileReviewState {
    pub status: ReviewStatus,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReviewStatus {
    Pending,
    Reviewed,
}

impl ReviewState {
    pub fn load(repo_path: &Path, branch: &str) -> Result<Self> {
        let state_path = state_file_path(repo_path, branch)?;

        if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)
                .context("Failed to read review state")?;
            let mut state: ReviewState =
                serde_json::from_str(&content).context("Failed to parse review state")?;
            state.state_path = state_path;
            Ok(state)
        } else {
            Ok(ReviewState {
                repo: repo_path.to_string_lossy().to_string(),
                branch: branch.to_string(),
                started_at: Utc::now(),
                files: HashMap::new(),
                state_path,
            })
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&self.state_path, json)?;
        Ok(())
    }

    pub fn is_reviewed(&self, path: &str) -> bool {
        self.files
            .get(path)
            .is_some_and(|f| f.status == ReviewStatus::Reviewed)
    }

    pub fn mark_reviewed(&mut self, path: &str) {
        self.files.insert(
            path.to_string(),
            FileReviewState {
                status: ReviewStatus::Reviewed,
                reviewed_at: Some(Utc::now()),
            },
        );
    }

    pub fn mark_pending(&mut self, path: &str) {
        self.files.insert(
            path.to_string(),
            FileReviewState {
                status: ReviewStatus::Pending,
                reviewed_at: None,
            },
        );
    }

    pub fn toggle_reviewed(&mut self, path: &str) {
        if self.is_reviewed(path) {
            self.mark_pending(path);
        } else {
            self.mark_reviewed(path);
        }
    }
}

fn state_file_path(repo_path: &Path, branch: &str) -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine XDG data directory")?
        .join("rvw");

    let mut hasher = Sha256::new();
    hasher.update(repo_path.to_string_lossy().as_bytes());
    let hash = format!("{:.12x}", hasher.finalize());
    let hash = &hash[..12];

    // Sanitize branch name for filename
    let safe_branch: String = branch
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();

    let filename = format!("{}-{}.json", hash, safe_branch);
    Ok(data_dir.join(filename))
}
