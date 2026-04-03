use tower_lsp::lsp_types::*;

use crate::git::DiffHunk;

pub fn hunks_to_diagnostics(hunks: &[DiffHunk]) -> Vec<Diagnostic> {
    hunks
        .iter()
        .map(|hunk| {
            let start_line = hunk.new_start.saturating_sub(1); // LSP is 0-based
            let end_line = start_line + hunk.new_lines.saturating_sub(1);

            let added = hunk.added_lines.len();
            let removed = hunk.removed_lines.len();

            let message = if removed == 0 {
                format!("New code (+{} lines)", added)
            } else if added == 0 {
                format!("Removed code (-{} lines)", removed)
            } else {
                format!("Changed: +{} -{} lines", added, removed)
            };

            Diagnostic {
                range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: end_line,
                        character: u32::MAX,
                    },
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                source: Some("rvw".to_string()),
                message,
                ..Default::default()
            }
        })
        .collect()
}
