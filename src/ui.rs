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
use crate::git::DiffLineKind;
use crate::input;

pub async fn run_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    loop {
        app.ensure_diff_loaded();
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
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(5),    // main area (file list + diff preview)
            Constraint::Length(2), // help bar
        ])
        .split(f.area());

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // file list
            Constraint::Min(30),        // diff preview
        ])
        .split(rows[1]);

    render_header(f, app, rows[0]);
    render_file_list(f, app, cols[0]);
    render_diff_preview(f, app, cols[1]);
    render_help(f, rows[2]);
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
        Span::styled(
            header,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
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
                "  [{}] {:40} {}  {:>10}",
                reviewed,
                display_path,
                file.status.label(),
                stats,
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
        .highlight_style(
            Style::default()
                .bg(Color::Indexed(236))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_diff_preview(f: &mut Frame, app: &App, area: Rect) {
    let file = match app.selected_file() {
        Some(f) => f,
        None => {
            let empty =
                Paragraph::new("  No file selected").block(Block::default().borders(Borders::LEFT));
            f.render_widget(empty, area);
            return;
        }
    };

    let hunks = match app.diff_cache.get(&file.path) {
        Some(h) => h,
        None => {
            let loading =
                Paragraph::new("  Loading...").block(Block::default().borders(Borders::LEFT));
            f.render_widget(loading, area);
            return;
        }
    };

    if hunks.is_empty() {
        let msg = if file.is_binary {
            "  Binary file"
        } else {
            "  No diff"
        };
        let empty = Paragraph::new(msg).block(Block::default().borders(Borders::LEFT));
        f.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    for (i, hunk) in hunks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            format!(" {}", hunk.header),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::DIM),
        )));

        for diff_line in &hunk.lines {
            let (prefix, style) = match diff_line.kind {
                DiffLineKind::Context => {
                    let ln = diff_line.new_lineno.unwrap_or(0);
                    (
                        format!(" {:>4}  ", ln),
                        Style::default().fg(Color::DarkGray),
                    )
                }
                DiffLineKind::Added => {
                    let ln = diff_line.new_lineno.unwrap_or(0);
                    (format!(" {:>4} +", ln), Style::default().fg(Color::Green))
                }
                DiffLineKind::Removed => {
                    let ln = diff_line.old_lineno.unwrap_or(0);
                    (format!(" {:>4} -", ln), Style::default().fg(Color::Red))
                }
            };

            let content = diff_line.content.trim_end_matches('\n');
            lines.push(Line::from(Span::styled(
                format!("{}{}", prefix, content),
                style,
            )));
        }
    }

    let title = format!(" {} +{} -{} ", file.path, file.additions, file.deletions,);

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .scroll((app.diff_scroll, 0));

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("  Enter", Style::default().fg(Color::Green)),
        Span::raw(": open  "),
        Span::styled("1-9", Style::default().fg(Color::Green)),
        Span::raw(": hunk  "),
        Span::styled("r", Style::default().fg(Color::Green)),
        Span::raw(": reviewed  "),
        Span::styled("f", Style::default().fg(Color::Green)),
        Span::raw(": filter  "),
        Span::styled("Tab/S-Tab", Style::default().fg(Color::Green)),
        Span::raw(": scroll  "),
        Span::styled("^d/^u", Style::default().fg(Color::Green)),
        Span::raw(": page  "),
        Span::styled("q", Style::default().fg(Color::Green)),
        Span::raw(": quit"),
    ]);
    let widget = Paragraph::new(help).block(Block::default().borders(Borders::TOP));
    f.render_widget(widget, area);
}
