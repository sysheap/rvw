use std::path::Path;

pub struct LanguageInfo {
    pub name: &'static str,
    pub comment_prefix: &'static str,
    #[allow(dead_code)]
    pub comment_suffix: Option<&'static str>,
    pub default_lsp_servers: &'static [&'static str],
}

pub fn language_for_path(path: &Path) -> Option<LanguageInfo> {
    let ext = path.extension()?.to_str()?;
    language_for_extension(ext)
}

pub fn language_for_extension(ext: &str) -> Option<LanguageInfo> {
    let info = match ext {
        "rs" => LanguageInfo {
            name: "rust",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["rust-analyzer"],
        },
        "go" => LanguageInfo {
            name: "go",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["gopls"],
        },
        "js" | "mjs" | "cjs" => LanguageInfo {
            name: "javascript",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["typescript-language-server"],
        },
        "ts" | "mts" | "cts" => LanguageInfo {
            name: "typescript",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["typescript-language-server"],
        },
        "jsx" => LanguageInfo {
            name: "jsx",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["typescript-language-server"],
        },
        "tsx" => LanguageInfo {
            name: "tsx",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["typescript-language-server"],
        },
        "py" | "pyi" => LanguageInfo {
            name: "python",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["pylsp"],
        },
        "rb" => LanguageInfo {
            name: "ruby",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["solargraph"],
        },
        "c" | "h" => LanguageInfo {
            name: "c",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["clangd"],
        },
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => LanguageInfo {
            name: "cpp",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["clangd"],
        },
        "java" => LanguageInfo {
            name: "java",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["jdtls"],
        },
        "kt" | "kts" => LanguageInfo {
            name: "kotlin",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["kotlin-language-server"],
        },
        "swift" => LanguageInfo {
            name: "swift",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["sourcekit-lsp"],
        },
        "zig" => LanguageInfo {
            name: "zig",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["zls"],
        },
        "lua" => LanguageInfo {
            name: "lua",
            comment_prefix: "--",
            comment_suffix: None,
            default_lsp_servers: &["lua-language-server"],
        },
        "sh" | "bash" | "zsh" => LanguageInfo {
            name: "bash",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["bash-language-server"],
        },
        "css" => LanguageInfo {
            name: "css",
            comment_prefix: "/*",
            comment_suffix: Some("*/"),
            default_lsp_servers: &["vscode-css-language-server"],
        },
        "scss" => LanguageInfo {
            name: "scss",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["vscode-css-language-server"],
        },
        "html" | "htm" => LanguageInfo {
            name: "html",
            comment_prefix: "<!--",
            comment_suffix: Some("-->"),
            default_lsp_servers: &["vscode-html-language-server"],
        },
        "json" => LanguageInfo {
            name: "json",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["vscode-json-language-server"],
        },
        "toml" => LanguageInfo {
            name: "toml",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["taplo"],
        },
        "yaml" | "yml" => LanguageInfo {
            name: "yaml",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["yaml-language-server"],
        },
        "ex" | "exs" => LanguageInfo {
            name: "elixir",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["elixir-ls"],
        },
        "erl" | "hrl" => LanguageInfo {
            name: "erlang",
            comment_prefix: "%",
            comment_suffix: None,
            default_lsp_servers: &["erlang-ls"],
        },
        "cs" => LanguageInfo {
            name: "c-sharp",
            comment_prefix: "//",
            comment_suffix: None,
            default_lsp_servers: &["OmniSharp"],
        },
        "tf" => LanguageInfo {
            name: "hcl",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["terraform-ls"],
        },
        "nix" => LanguageInfo {
            name: "nix",
            comment_prefix: "#",
            comment_suffix: None,
            default_lsp_servers: &["nil"],
        },
        "sql" => LanguageInfo {
            name: "sql",
            comment_prefix: "--",
            comment_suffix: None,
            default_lsp_servers: &[],
        },
        "md" | "markdown" => LanguageInfo {
            name: "markdown",
            comment_prefix: "<!--",
            comment_suffix: Some("-->"),
            default_lsp_servers: &[],
        },
        _ => return None,
    };
    Some(info)
}

pub fn count_annotations_in_content(content: &str, path: &Path) -> usize {
    let lang = match language_for_path(path) {
        Some(l) => l,
        None => return 0,
    };
    let prefix = lang.comment_prefix;
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Look for "COMMENT_PREFIX REVIEW:" or "COMMENT_PREFIX REVIEW:" patterns
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let rest = rest.trim_start();
                rest.starts_with("REVIEW:")
            } else {
                false
            }
        })
        .count()
}
