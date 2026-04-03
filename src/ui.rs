use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

use crate::app::App;
use crate::input;

pub async fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        terminal.draw(|f| render(f, app))?;

        if let Event::Key(key) = event::read()? {
            let action = input::handle_key(key, app);
            match action {
                input::Action::None => {}
                input::Action::Quit => break,
                input::Action::OpenEditor { path, line } => {
                    // Suspend TUI
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

                    // Spawn editor
                    let opened_path = path.clone();
                    crate::editor::open_editor(
                        &app.editor_cmd,
                        &app.repo_info.repo_path,
                        &path,
                        line,
                    )?;

                    // Mark as reviewed
                    app.mark_reviewed(&opened_path);

                    // Resume TUI
                    enable_raw_mode()?;
                    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
                    terminal.clear()?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // header
            Constraint::Min(5),    // file list
            Constraint::Length(8), // detail pane
            Constraint::Length(2), // help bar
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_file_list(f, app, chunks[1]);
    render_detail(f, app, chunks[2]);
    render_help(f, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let reviewed = app.reviewed_count();
    let total = app.total_count();
    let header = format!(
        " rvw: {} → {} ── {} files, {} reviewed",
        app.repo_info.branch, app.repo_info.base_branch, total, reviewed
    );
    let filter_label = if app.filter != crate::app::FilterMode::All {
        format!(" [filter: {}]", app.filter.label())
    } else {
        String::new()
    };
    let line = Line::from(vec![
        Span::styled(header, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(filter_label, Style::default().fg(Color::Yellow)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn render_file_list(f: &mut Frame, app: &App, area: Rect) {
    let files = app.filtered_files();
    let items: Vec<ListItem> = files
        .iter()
        .map(|file| {
            let reviewed = if app.review_state.is_reviewed(&file.path) {
                "x"
            } else {
                " "
            };
            let annotation_count = app.annotation_count(&file.path);
            let annotations = if annotation_count > 0 {
                format!("  {} annotations", annotation_count)
            } else {
                String::new()
            };

            let display_path = if let Some(ref old) = file.old_path {
                format!("{} → {}", old, file.path)
            } else {
                file.path.clone()
            };

            let stats = if file.is_binary {
                "binary".to_string()
            } else {
                let add = if file.additions > 0 {
                    format!("+{}", file.additions)
                } else {
                    String::new()
                };
                let del = if file.deletions > 0 {
                    format!("-{}", file.deletions)
                } else {
                    String::new()
                };
                format!("{} {}", add, del).trim().to_string()
            };

            let line = format!(
                "  [{}] {:40} {}  {:>10}{}",
                reviewed,
                display_path,
                file.status.label(),
                stats,
                annotations,
            );

            let style = if app.review_state.is_reviewed(&file.path) {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(line, style)))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected));

    let list = List::new(items)
        .block(Block::default().borders(Borders::NONE))
        .highlight_style(Style::default().bg(Color::Indexed(236)).add_modifier(Modifier::BOLD))
        .highlight_symbol("▸ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(file) = app.selected_file() {
        let mut lines = vec![Line::from(Span::styled(
            format!(
                "  {} — {} (+{} -{})",
                file.path,
                match file.status {
                    crate::git::FileStatus::Added => "Added",
                    crate::git::FileStatus::Modified => "Modified",
                    crate::git::FileStatus::Deleted => "Deleted",
                    crate::git::FileStatus::Renamed => "Renamed",
                },
                file.additions,
                file.deletions
            ),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ))];

        if file.hunks.is_empty() {
            lines.push(Line::from("  No hunks (binary or empty diff)"));
        } else {
            lines.push(Line::from(Span::styled(
                "  Changed regions:",
                Style::default().fg(Color::Gray),
            )));
            for (i, hunk) in file.hunks.iter().enumerate() {
                let end_line = hunk.new_start + hunk.new_lines.saturating_sub(1);
                let range = if hunk.new_lines <= 1 {
                    format!("Line {}", hunk.new_start)
                } else {
                    format!("Lines {}-{}", hunk.new_start, end_line)
                };

                // Try to extract function name from hunk header
                let context = if hunk.header.contains("fn ")
                    || hunk.header.contains("def ")
                    || hunk.header.contains("func ")
                    || hunk.header.contains("function ")
                    || hunk.header.contains("class ")
                {
                    let trimmed = hunk.header.trim_start_matches("@@ ");
                    if let Some(pos) = trimmed.find("@@") {
                        let after = trimmed[pos + 2..].trim();
                        if !after.is_empty() {
                            format!("  ({})", after)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                lines.push(Line::from(Span::styled(
                    format!("    {}. {}{}", i + 1, range, context),
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        lines
    } else {
        vec![Line::from("  No file selected")]
    };

    let detail = Paragraph::new(content)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(detail, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("  Enter", Style::default().fg(Color::Green)),
        Span::raw(": open  "),
        Span::styled("1-9", Style::default().fg(Color::Green)),
        Span::raw(": open at hunk  "),
        Span::styled("r", Style::default().fg(Color::Green)),
        Span::raw(": toggle reviewed  "),
        Span::styled("f", Style::default().fg(Color::Green)),
        Span::raw(": filter  "),
        Span::styled("q", Style::default().fg(Color::Green)),
        Span::raw(": quit"),
    ]);
    let widget = Paragraph::new(help)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(widget, area);
}
