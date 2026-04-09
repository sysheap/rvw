use anyhow::{Context, Result, bail};
use git2::{Delta, DiffOptions, Repository};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<Hunk>,
    pub is_binary: bool,
    /// Fingerprint of the file's content on both sides of the diff
    /// (`<old_blob_oid>:<new_blob_oid>`). Used to detect when a previously
    /// reviewed file has changed and should be re-reviewed.
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl FileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub new_start: u32,
}

#[derive(Debug)]
pub struct RepoInfo {
    pub repo_path: PathBuf,
    pub branch: String,
    pub base_branch: String,
    pub files: Vec<ChangedFile>,
}

pub fn detect_base_branch(repo: &Repository) -> Result<String> {
    for name in &["main", "master"] {
        if repo.find_branch(name, git2::BranchType::Local).is_ok() {
            return Ok(name.to_string());
        }
    }
    bail!("Could not find 'main' or 'master' branch. Use --base to specify the base branch.")
}

pub fn current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head().context("Failed to get HEAD")?;
    if head.is_branch() {
        let name = head.shorthand().unwrap_or("HEAD").to_string();
        Ok(name)
    } else {
        Ok("HEAD (detached)".to_string())
    }
}

pub fn analyze_repo(repo_path: &Path, base_override: Option<&str>) -> Result<RepoInfo> {
    let repo = Repository::discover(repo_path).context("Not a git repository")?;
    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?
        .to_path_buf();

    let branch = current_branch(&repo)?;
    let base_branch = match base_override {
        Some(b) => b.to_string(),
        None => detect_base_branch(&repo)?,
    };

    let merge_base_commit = find_merge_base(&repo, &base_branch)?;
    let head_commit = repo
        .head()?
        .peel_to_commit()
        .context("HEAD is not a commit")?;

    let merge_base_tree = merge_base_commit.tree()?;
    let head_tree = head_commit.tree()?;

    let mut diff_opts = DiffOptions::new();
    diff_opts.patience(true);
    diff_opts.context_lines(0);

    let diff = repo
        .diff_tree_to_tree(
            Some(&merge_base_tree),
            Some(&head_tree),
            Some(&mut diff_opts),
        )
        .context("Failed to compute diff")?;

    let files = collect_changed_files(&diff)?;

    Ok(RepoInfo {
        repo_path: workdir,
        branch,
        base_branch,
        files,
    })
}

fn find_merge_base<'a>(repo: &'a Repository, base_branch: &str) -> Result<git2::Commit<'a>> {
    let base_ref = repo
        .find_branch(base_branch, git2::BranchType::Local)
        .with_context(|| format!("Branch '{}' not found", base_branch))?;
    let base_commit = base_ref
        .get()
        .peel_to_commit()
        .context("Base branch does not point to a commit")?;

    let head_commit = repo
        .head()?
        .peel_to_commit()
        .context("HEAD is not a commit")?;

    let merge_base_oid = repo
        .merge_base(base_commit.id(), head_commit.id())
        .context("Could not find merge base between HEAD and base branch")?;

    let merge_base = repo
        .find_commit(merge_base_oid)
        .context("Merge base commit not found")?;

    Ok(merge_base)
}

fn collect_changed_files(diff: &git2::Diff<'_>) -> Result<Vec<ChangedFile>> {
    let mut files: Vec<ChangedFile> = Vec::new();

    let stats_per_file = collect_stats(diff)?;

    for (i, delta) in diff.deltas().enumerate() {
        let status = match delta.status() {
            Delta::Added => FileStatus::Added,
            Delta::Modified => FileStatus::Modified,
            Delta::Deleted => FileStatus::Deleted,
            Delta::Renamed => FileStatus::Renamed,
            _ => continue,
        };

        let new_file = delta.new_file();
        let old_file = delta.old_file();
        let path = new_file
            .path()
            .or_else(|| old_file.path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let old_path = if status == FileStatus::Renamed {
            old_file.path().map(|p| p.to_string_lossy().to_string())
        } else {
            None
        };

        let is_binary = new_file.is_binary() || old_file.is_binary();

        let (additions, deletions) = stats_per_file.get(i).copied().unwrap_or((0, 0));

        let hunks = collect_hunks_for_file(diff, i)?;

        let signature = format!("{}:{}", old_file.id(), new_file.id());

        files.push(ChangedFile {
            path,
            old_path,
            status,
            additions,
            deletions,
            hunks,
            is_binary,
            signature,
        });
    }

    // Sort: new files first, then modified, then deleted. Within same status, alphabetical.
    files.sort_by(|a, b| {
        let status_order = |s: &FileStatus| -> u8 {
            match s {
                FileStatus::Added => 0,
                FileStatus::Modified => 1,
                FileStatus::Renamed => 2,
                FileStatus::Deleted => 3,
            }
        };
        status_order(&a.status)
            .cmp(&status_order(&b.status))
            .then_with(|| a.path.cmp(&b.path))
    });

    Ok(files)
}

fn collect_stats(diff: &git2::Diff<'_>) -> Result<Vec<(usize, usize)>> {
    let mut stats: Vec<(usize, usize)> = vec![(0, 0); diff.deltas().len()];

    diff.foreach(
        &mut |_, _| true,
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            // Find which delta index this belongs to
            // The callback doesn't directly give us the index, so we match by file path
            let file_path = delta.new_file().path().or_else(|| delta.old_file().path());

            if let Some(fp) = file_path {
                for (i, d) in diff.deltas().enumerate() {
                    let dp = d.new_file().path().or_else(|| d.old_file().path());
                    if dp == Some(fp) {
                        match line.origin() {
                            '+' => stats[i].0 += 1,
                            '-' => stats[i].1 += 1,
                            _ => {}
                        }
                        break;
                    }
                }
            }
            true
        }),
    )
    .context("Failed to iterate diff")?;

    Ok(stats)
}

fn collect_hunks_for_file(diff: &git2::Diff<'_>, file_index: usize) -> Result<Vec<Hunk>> {
    let mut hunks = Vec::new();
    let patch = git2::Patch::from_diff(diff, file_index)?;

    if let Some(patch) = patch {
        let num_hunks = patch.num_hunks();
        for h in 0..num_hunks {
            let (hunk, _) = patch.hunk(h)?;
            hunks.push(Hunk {
                new_start: hunk.new_start(),
            });
        }
    }

    Ok(hunks)
}

/// Get the content of a file at the merge-base commit (used by LSP hover).
#[allow(dead_code)]
pub fn get_base_file_content(
    repo_path: &Path,
    base_branch: &str,
    file_path: &str,
) -> Result<String> {
    let repo = Repository::discover(repo_path)?;
    let merge_base = find_merge_base(&repo, base_branch)?;
    let tree = merge_base.tree()?;
    let entry = tree
        .get_path(Path::new(file_path))
        .with_context(|| format!("File '{}' not found at base", file_path))?;
    let blob = entry.to_object(&repo)?.peel_to_blob()?;
    let content = String::from_utf8_lossy(blob.content()).to_string();
    Ok(content)
}

/// Compute diff hunks for a specific file between merge-base and HEAD.
pub fn diff_hunks_for_file(
    repo_path: &Path,
    base_branch: &str,
    file_path: &str,
) -> Result<Vec<DiffHunk>> {
    let repo = Repository::discover(repo_path)?;
    let merge_base = find_merge_base(&repo, base_branch)?;
    let head_commit = repo.head()?.peel_to_commit()?;

    let merge_base_tree = merge_base.tree()?;
    let head_tree = head_commit.tree()?;

    let mut diff_opts = DiffOptions::new();
    diff_opts.patience(true);
    diff_opts.context_lines(3);
    diff_opts.pathspec(file_path);

    let diff = repo.diff_tree_to_tree(
        Some(&merge_base_tree),
        Some(&head_tree),
        Some(&mut diff_opts),
    )?;

    let mut hunks = Vec::new();

    for di in 0..diff.deltas().len() {
        let patch = git2::Patch::from_diff(&diff, di)?;
        if let Some(patch) = patch {
            for h in 0..patch.num_hunks() {
                let (hunk_info, num_lines) = patch.hunk(h)?;
                let header = String::from_utf8_lossy(hunk_info.header())
                    .trim()
                    .to_string();
                let mut lines = Vec::new();

                for l in 0..num_lines {
                    let line = patch.line_in_hunk(h, l)?;
                    let kind = match line.origin() {
                        '-' => DiffLineKind::Removed,
                        '+' => DiffLineKind::Added,
                        ' ' => DiffLineKind::Context,
                        _ => continue,
                    };
                    lines.push(DiffLine {
                        kind,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        content: String::from_utf8_lossy(line.content()).into_owned(),
                    });
                }

                hunks.push(DiffHunk {
                    new_start: hunk_info.new_start(),
                    new_lines: hunk_info.new_lines(),
                    header,
                    lines,
                });
            }
        }
    }

    Ok(hunks)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    pub fn removed_lines(&self) -> impl Iterator<Item = (u32, &str)> {
        self.lines
            .iter()
            .filter(|l| l.kind == DiffLineKind::Removed)
            .filter_map(|l| l.old_lineno.map(|n| (n, l.content.as_str())))
    }

    pub fn added_lines(&self) -> impl Iterator<Item = (u32, &str)> {
        self.lines
            .iter()
            .filter(|l| l.kind == DiffLineKind::Added)
            .filter_map(|l| l.new_lineno.map(|n| (n, l.content.as_str())))
    }
}
