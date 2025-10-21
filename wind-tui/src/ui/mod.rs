mod components;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::{config::Config, state::AppState};

pub fn render<'a>(f: &mut Frame, state: &AppState<'a>, config: &Config) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    components::render_header(f, chunks[0], state, config);
    render_main_area(f, chunks[1], state, config);
    components::render_footer(f, chunks[2], state, config);

    if state.command_palette_open {
        components::render_command_palette(f, f.area(), state, config);
    }

    if state.is_commit_editor_open {
        components::render_commit_editor(f, f.area(), state, config);
    }

    components::render_notifications(f, f.area(), state, config);
    components::render_jobs(f, f.area(), state, config);
}

fn render_main_area<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    components::render_status(f, left_chunks[0], state, config);
    components::render_files(f, left_chunks[1], state, config);

    match state.active_pane {
        crate::state::Pane::Diff => {
            components::render_diff(f, chunks[1], state, config);
        }
        crate::state::Pane::Branches | crate::state::Pane::Commits => {
            components::render_branches(f, chunks[1], state, config);
        }
        _ => {
            components::render_diff(f, chunks[1], state, config);
        }
    }
}
