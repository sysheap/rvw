mod app;
mod editor;
mod git;
mod input;
mod languages;
mod lsp;
mod review;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rvw", about = "Terminal code review tool for agent-produced branches")]
struct Cli {
    /// Base branch to diff against (default: auto-detect main/master)
    #[arg(short, long)]
    base: Option<String>,

    /// Editor command (default: hx, or $EDITOR)
    #[arg(short, long)]
    editor: Option<String>,

    /// Repository path (default: current directory)
    #[arg(short, long)]
    repo: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as LSP server (started by editor, not for direct use)
    Lsp {
        /// Base branch to diff against
        #[arg(long)]
        base: String,

        /// Repository path
        #[arg(long)]
        repo: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Lsp { base, repo }) => {
            lsp::run_lsp_server(repo, base).await?;
        }
        None => {
            let repo_path = cli.repo.unwrap_or_else(|| PathBuf::from("."));
            let repo_path = repo_path.canonicalize()?;
            app::run(repo_path, cli.base.as_deref(), cli.editor.as_deref()).await?;
        }
    }

    Ok(())
}
