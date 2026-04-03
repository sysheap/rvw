use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub enum Action {
    None,
    Quit,
    OpenEditor { path: String, line: u32 },
}

pub fn handle_key(key: KeyEvent, app: &mut App) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            app.move_selection(1);
            Action::None
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.move_selection(-1);
            Action::None
        }
        KeyCode::Char('g') => {
            // Go to top
            app.selected = 0;
            Action::None
        }
        KeyCode::Char('G') => {
            // Go to bottom
            let len = app.filtered_files().len();
            if len > 0 {
                app.selected = len - 1;
            }
            Action::None
        }
        KeyCode::Char('f') => {
            app.toggle_filter();
            Action::None
        }
        KeyCode::Char('r') => {
            app.toggle_reviewed();
            Action::None
        }
        KeyCode::Enter => {
            if let Some(file) = app.selected_file() {
                if file.status == crate::git::FileStatus::Deleted || file.is_binary {
                    return Action::None;
                }
                let path = file.path.clone();
                let line = file
                    .hunks
                    .first()
                    .map(|h| h.new_start)
                    .unwrap_or(1);
                Action::OpenEditor { path, line }
            } else {
                Action::None
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let hunk_idx = (c as u8 - b'1') as usize;
            if let Some(file) = app.selected_file() {
                if file.status == crate::git::FileStatus::Deleted || file.is_binary {
                    return Action::None;
                }
                if let Some(hunk) = file.hunks.get(hunk_idx) {
                    let path = file.path.clone();
                    let line = hunk.new_start;
                    return Action::OpenEditor { path, line };
                }
            }
            Action::None
        }
        _ => Action::None,
    }
}
