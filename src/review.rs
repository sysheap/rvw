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
    /// Snapshot of the file's diff signature at the moment it was marked
    /// reviewed. `None` for legacy state files written before this field
    /// existed; such entries are left untouched by `invalidate_stale`.
    #[serde(default)]
    pub reviewed_signature: Option<String>,
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
            let content =
                std::fs::read_to_string(&state_path).context("Failed to read review state")?;
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

    pub fn mark_reviewed(&mut self, path: &str, signature: String) {
        self.files.insert(
            path.to_string(),
            FileReviewState {
                status: ReviewStatus::Reviewed,
                reviewed_at: Some(Utc::now()),
                reviewed_signature: Some(signature),
            },
        );
    }

    pub fn mark_pending(&mut self, path: &str) {
        self.files.insert(
            path.to_string(),
            FileReviewState {
                status: ReviewStatus::Pending,
                reviewed_at: None,
                reviewed_signature: None,
            },
        );
    }

    pub fn toggle_reviewed(&mut self, path: &str, signature: String) {
        if self.is_reviewed(path) {
            self.mark_pending(path);
        } else {
            self.mark_reviewed(path, signature);
        }
    }

    /// Demote any "Reviewed" entries whose stored diff signature no longer
    /// matches the current signature for that path.
    ///
    /// Note: this intentionally fires when a merge-base shift changes the
    /// base-side blob OID, even if the file in HEAD is untouched. The diff
    /// the reviewer saw is no longer what's currently displayed, so a fresh
    /// review is warranted. Don't "fix" this.
    ///
    /// Legacy entries with `reviewed_signature == None` are left alone — we
    /// have no baseline to compare against, so we trust them. The next
    /// `mark_reviewed` will populate the field.
    pub fn invalidate_stale(&mut self, current_signatures: &HashMap<String, String>) {
        for (path, file_state) in self.files.iter_mut() {
            if file_state.status != ReviewStatus::Reviewed {
                continue;
            }
            let Some(stored) = file_state.reviewed_signature.as_ref() else {
                continue;
            };
            if current_signatures.get(path) != Some(stored) {
                file_state.status = ReviewStatus::Pending;
                file_state.reviewed_at = None;
                file_state.reviewed_signature = None;
            }
        }
    }
}

fn state_file_path(repo_path: &Path, branch: &str) -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine XDG data directory")?
        .join("rvw");

    let mut hasher = Sha256::new();
    hasher.update(repo_path.to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    let hash: String = digest
        .iter()
        .take(6)
        .map(|b| format!("{:02x}", b))
        .collect();

    // Sanitize branch name for filename
    let safe_branch: String = branch
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let filename = format!("{}-{}.json", hash, safe_branch);
    Ok(data_dir.join(filename))
}
