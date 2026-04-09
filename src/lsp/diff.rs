use tower_lsp::lsp_types::*;

use crate::git::DiffHunk;

pub fn hunks_to_diagnostics(hunks: &[DiffHunk]) -> Vec<Diagnostic> {
    hunks
        .iter()
        .map(|hunk| {
            let start_line = hunk.new_start.saturating_sub(1); // LSP is 0-based

            let added = hunk.added_lines().count();
            let removed = hunk.removed_lines().count();

            let message = if removed == 0 {
                format!("New code (+{} lines)", added)
            } else if added == 0 {
                format!("Removed code (-{} lines)", removed)
            } else {
                format!("Changed: +{} -{} lines", added, removed)
            };

            // Emit a zero-width HINT marker at the first line of the hunk
            // instead of spanning every added line. This keeps `]d`/`[d` and
            // hover working but avoids carpeting the buffer with squiggles.
            Diagnostic {
                range: Range {
                    start: Position {
                        line: start_line,
                        character: 0,
                    },
                    end: Position {
                        line: start_line,
                        character: 0,
                    },
                },
                severity: Some(DiagnosticSeverity::HINT),
                source: Some("rvw".to_string()),
                message,
                ..Default::default()
            }
        })
        .collect()
}
