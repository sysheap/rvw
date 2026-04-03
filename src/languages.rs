use std::path::Path;

pub struct LanguageInfo {
    pub name: &'static str,
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
            default_lsp_servers: &["rust-analyzer"],
        },
        "go" => LanguageInfo {
            name: "go",
            default_lsp_servers: &["gopls"],
        },
        "js" | "mjs" | "cjs" => LanguageInfo {
            name: "javascript",
            default_lsp_servers: &["typescript-language-server"],
        },
        "ts" | "mts" | "cts" => LanguageInfo {
            name: "typescript",
            default_lsp_servers: &["typescript-language-server"],
        },
        "jsx" => LanguageInfo {
            name: "jsx",
            default_lsp_servers: &["typescript-language-server"],
        },
        "tsx" => LanguageInfo {
            name: "tsx",
            default_lsp_servers: &["typescript-language-server"],
        },
        "py" | "pyi" => LanguageInfo {
            name: "python",
            default_lsp_servers: &["pylsp"],
        },
        "rb" => LanguageInfo {
            name: "ruby",
            default_lsp_servers: &["solargraph"],
        },
        "c" | "h" => LanguageInfo {
            name: "c",
            default_lsp_servers: &["clangd"],
        },
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => LanguageInfo {
            name: "cpp",
            default_lsp_servers: &["clangd"],
        },
        "java" => LanguageInfo {
            name: "java",
            default_lsp_servers: &["jdtls"],
        },
        "kt" | "kts" => LanguageInfo {
            name: "kotlin",
            default_lsp_servers: &["kotlin-language-server"],
        },
        "swift" => LanguageInfo {
            name: "swift",
            default_lsp_servers: &["sourcekit-lsp"],
        },
        "zig" => LanguageInfo {
            name: "zig",
            default_lsp_servers: &["zls"],
        },
        "lua" => LanguageInfo {
            name: "lua",
            default_lsp_servers: &["lua-language-server"],
        },
        "sh" | "bash" | "zsh" => LanguageInfo {
            name: "bash",
            default_lsp_servers: &["bash-language-server"],
        },
        "css" => LanguageInfo {
            name: "css",
            default_lsp_servers: &["vscode-css-language-server"],
        },
        "scss" => LanguageInfo {
            name: "scss",
            default_lsp_servers: &["vscode-css-language-server"],
        },
        "html" | "htm" => LanguageInfo {
            name: "html",
            default_lsp_servers: &["vscode-html-language-server"],
        },
        "json" => LanguageInfo {
            name: "json",
            default_lsp_servers: &["vscode-json-language-server"],
        },
        "toml" => LanguageInfo {
            name: "toml",
            default_lsp_servers: &["taplo"],
        },
        "yaml" | "yml" => LanguageInfo {
            name: "yaml",
            default_lsp_servers: &["yaml-language-server"],
        },
        "ex" | "exs" => LanguageInfo {
            name: "elixir",
            default_lsp_servers: &["elixir-ls"],
        },
        "erl" | "hrl" => LanguageInfo {
            name: "erlang",
            default_lsp_servers: &["erlang-ls"],
        },
        "cs" => LanguageInfo {
            name: "c-sharp",
            default_lsp_servers: &["OmniSharp"],
        },
        "tf" => LanguageInfo {
            name: "hcl",
            default_lsp_servers: &["terraform-ls"],
        },
        "nix" => LanguageInfo {
            name: "nix",
            default_lsp_servers: &["nil"],
        },
        "sql" => LanguageInfo {
            name: "sql",
            default_lsp_servers: &[],
        },
        "md" | "markdown" => LanguageInfo {
            name: "markdown",
            default_lsp_servers: &[],
        },
        _ => return None,
    };
    Some(info)
}
