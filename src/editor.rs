use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use crate::git::RepoInfo;
use crate::languages;

pub fn open_editor(editor_cmd: &str, repo_path: &Path, file_path: &str, line: u32) -> Result<()> {
    let full_path = repo_path.join(file_path);
    let full_path_str = full_path.to_string_lossy();

    let editor_name = Path::new(editor_cmd)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(editor_cmd);

    let status = match editor_name {
        "hx" | "helix" => Command::new(editor_cmd)
            .arg(format!("{}:{}", full_path_str, line))
            .current_dir(repo_path)
            .status(),
        "vim" | "nvim" | "vi" => Command::new(editor_cmd)
            .arg(format!("+{}", line))
            .arg(full_path_str.as_ref())
            .current_dir(repo_path)
            .status(),
        "code" | "code-insiders" => Command::new(editor_cmd)
            .arg("-g")
            .arg(format!("{}:{}", full_path_str, line))
            .current_dir(repo_path)
            .status(),
        "emacs" | "emacsclient" => Command::new(editor_cmd)
            .arg(format!("+{}", line))
            .arg(full_path_str.as_ref())
            .current_dir(repo_path)
            .status(),
        _ => {
            // Default: try helix-style file:line
            Command::new(editor_cmd)
                .arg(format!("{}:{}", full_path_str, line))
                .current_dir(repo_path)
                .status()
        }
    };

    status.with_context(|| format!("Failed to launch editor '{}'", editor_cmd))?;

    Ok(())
}

/// Cleanup state shared with the Ctrl+C handler.
static CLEANUP_STATE: Mutex<Option<CleanupInfo>> = Mutex::new(None);

#[derive(Clone)]
struct CleanupInfo {
    config_path: PathBuf,
    backup_path: PathBuf,
    had_existing: bool,
    helix_dir: PathBuf,
}

/// Manages the .helix/languages.toml configuration for the review LSP.
pub struct HelixConfig {
    helix_dir: PathBuf,
    config_path: PathBuf,
    backup_path: PathBuf,
    had_existing: bool,
    repo_path: PathBuf,
    base_branch: String,
    languages_in_diff: Vec<String>,
}

impl HelixConfig {
    pub fn new(repo_info: &RepoInfo) -> Result<Self> {
        let helix_dir = repo_info.repo_path.join(".helix");
        let config_path = helix_dir.join("languages.toml");
        let backup_path = helix_dir.join("languages.toml.rvw-backup");

        // Collect unique language names from changed files
        let mut lang_names: Vec<String> = repo_info
            .files
            .iter()
            .filter_map(|f| {
                let path = Path::new(&f.path);
                languages::language_for_path(path).map(|l| l.name.to_string())
            })
            .collect();
        lang_names.sort();
        lang_names.dedup();

        Ok(Self {
            had_existing: config_path.exists(),
            helix_dir,
            config_path,
            backup_path,
            repo_path: repo_info.repo_path.clone(),
            base_branch: repo_info.base_branch.clone(),
            languages_in_diff: lang_names,
        })
    }

    pub fn install(&self) -> Result<()> {
        // Check for stale backup from a crashed session
        if self.backup_path.exists() {
            eprintln!(
                "Warning: Found stale .helix/languages.toml.rvw-backup from a previous session. Restoring it."
            );
            std::fs::rename(&self.backup_path, &self.config_path)
                .context("Failed to restore stale backup")?;
        }

        // Create .helix dir if needed
        if !self.helix_dir.exists() {
            std::fs::create_dir_all(&self.helix_dir)
                .context("Failed to create .helix directory")?;
        }

        // Backup existing config
        if self.config_path.exists() {
            std::fs::copy(&self.config_path, &self.backup_path)
                .context("Failed to backup .helix/languages.toml")?;
        }

        // Generate merged config
        let new_config = self.generate_config()?;
        std::fs::write(&self.config_path, new_config)
            .context("Failed to write .helix/languages.toml")?;

        // Register Ctrl+C cleanup handler
        let cleanup = CleanupInfo {
            config_path: self.config_path.clone(),
            backup_path: self.backup_path.clone(),
            had_existing: self.had_existing,
            helix_dir: self.helix_dir.clone(),
        };
        *CLEANUP_STATE.lock().unwrap() = Some(cleanup);
        let _ = ctrlc::set_handler(move || {
            if let Some(info) = CLEANUP_STATE.lock().unwrap().take() {
                do_cleanup(&info);
            }
            // Re-raise the signal for default behavior (terminal cleanup etc.)
            std::process::exit(130);
        });

        Ok(())
    }

    pub fn uninstall(&self) -> Result<()> {
        // Clear the Ctrl+C handler state
        *CLEANUP_STATE.lock().unwrap() = None;

        let info = CleanupInfo {
            config_path: self.config_path.clone(),
            backup_path: self.backup_path.clone(),
            had_existing: self.had_existing,
            helix_dir: self.helix_dir.clone(),
        };
        do_cleanup(&info);
        Ok(())
    }

    fn generate_config(&self) -> Result<String> {
        let rvw_binary = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("rvw"));
        let rvw_binary_str = rvw_binary.to_string_lossy();

        // Read existing config from backup if we just backed it up,
        // otherwise from the config path
        let existing_config: Option<toml::Value> = if self.backup_path.exists() {
            let content = std::fs::read_to_string(&self.backup_path)?;
            Some(toml::from_str(&content)?)
        } else if self.had_existing {
            let content = std::fs::read_to_string(&self.config_path)?;
            Some(toml::from_str(&content)?)
        } else {
            None
        };

        let mut doc = existing_config.unwrap_or_else(|| toml::Value::Table(toml::map::Map::new()));

        // Ensure top-level is a table
        let table = doc.as_table_mut().context("Config is not a table")?;

        // Add [language-server.rvw]
        let ls_table = table
            .entry("language-server")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
            .as_table_mut()
            .context("language-server is not a table")?;

        let mut rvw_server = toml::map::Map::new();
        rvw_server.insert(
            "command".to_string(),
            toml::Value::String(rvw_binary_str.to_string()),
        );
        rvw_server.insert(
            "args".to_string(),
            toml::Value::Array(vec![
                toml::Value::String("lsp".to_string()),
                toml::Value::String("--base".to_string()),
                toml::Value::String(self.base_branch.clone()),
                toml::Value::String("--repo".to_string()),
                toml::Value::String(self.repo_path.to_string_lossy().to_string()),
            ]),
        );
        ls_table.insert("rvw".to_string(), toml::Value::Table(rvw_server));

        // Add/update [[language]] entries
        let language_array = table
            .entry("language")
            .or_insert_with(|| toml::Value::Array(Vec::new()));

        let lang_arr = language_array
            .as_array_mut()
            .context("language is not an array")?;

        for lang_name in &self.languages_in_diff {
            let lang_info = self.find_lang_info(lang_name);

            // Check if this language already exists in the array
            let existing = lang_arr.iter_mut().find(|entry| {
                entry
                    .as_table()
                    .and_then(|t| t.get("name"))
                    .and_then(|n| n.as_str())
                    == Some(lang_name)
            });

            if let Some(entry) = existing {
                // Append rvw to existing language-servers list
                let entry_table = entry.as_table_mut().unwrap();
                if let Some(servers) = entry_table.get_mut("language-servers") {
                    if let Some(arr) = servers.as_array_mut() {
                        let has_rvw = arr.iter().any(|v| v.as_str() == Some("rvw"));
                        if !has_rvw {
                            arr.push(toml::Value::String("rvw".to_string()));
                        }
                    }
                } else {
                    // No language-servers key, add default + rvw
                    let mut servers: Vec<toml::Value> = lang_info
                        .iter()
                        .flat_map(|li| li.default_lsp_servers.iter())
                        .map(|s| toml::Value::String(s.to_string()))
                        .collect();
                    servers.push(toml::Value::String("rvw".to_string()));
                    entry_table.insert("language-servers".to_string(), toml::Value::Array(servers));
                }
            } else {
                // Create new language entry
                let mut lang_table = toml::map::Map::new();
                lang_table.insert("name".to_string(), toml::Value::String(lang_name.clone()));
                let mut servers: Vec<toml::Value> = lang_info
                    .iter()
                    .flat_map(|li| li.default_lsp_servers.iter())
                    .map(|s| toml::Value::String(s.to_string()))
                    .collect();
                servers.push(toml::Value::String("rvw".to_string()));
                lang_table.insert("language-servers".to_string(), toml::Value::Array(servers));
                lang_arr.push(toml::Value::Table(lang_table));
            }
        }

        Ok(toml::to_string_pretty(&doc)?)
    }

    fn find_lang_info(&self, name: &str) -> Option<languages::LanguageInfo> {
        let test_ext = match name {
            "rust" => "rs",
            "python" => "py",
            "javascript" => "js",
            "typescript" => "ts",
            "jsx" => "jsx",
            "tsx" => "tsx",
            "go" => "go",
            "c" => "c",
            "cpp" => "cpp",
            "java" => "java",
            "kotlin" => "kt",
            "swift" => "swift",
            "zig" => "zig",
            "lua" => "lua",
            "bash" => "sh",
            "ruby" => "rb",
            "css" => "css",
            "scss" => "scss",
            "html" => "html",
            "json" => "json",
            "toml" => "toml",
            "yaml" => "yaml",
            "elixir" => "ex",
            "erlang" => "erl",
            "c-sharp" => "cs",
            "hcl" => "tf",
            "nix" => "nix",
            "sql" => "sql",
            "markdown" => "md",
            _ => return None,
        };
        languages::language_for_extension(test_ext)
    }
}

fn do_cleanup(info: &CleanupInfo) {
    if info.backup_path.exists() {
        let _ = std::fs::rename(&info.backup_path, &info.config_path);
    } else if info.config_path.exists() && !info.had_existing {
        let _ = std::fs::remove_file(&info.config_path);
        if info.helix_dir.exists()
            && let Ok(mut entries) = std::fs::read_dir(&info.helix_dir)
            && entries.next().is_none()
        {
            let _ = std::fs::remove_dir(&info.helix_dir);
        }
    }
}
