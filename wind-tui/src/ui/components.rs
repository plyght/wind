use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    config::Config,
    state::{AppState, FileStatus, NotificationLevel, Pane},
};

pub fn render_header<'a>(f: &mut Frame, area: Rect, _state: &AppState<'a>, config: &Config) {
    let title = Paragraph::new("Wind TUI")
        .style(
            Style::default()
                .fg(config.theme.accent.into())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.theme.border.into())),
        );
    f.render_widget(title, area);
}

pub fn render_footer<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let help_text = match (state.is_commit_editor_open, state.command_palette_open) {
        (true, _) => "Ctrl+Enter: Commit | Esc: Cancel",
        (_, true) => "Enter: Execute | Esc: Cancel",
        _ => "q: Quit | Tab: Next Pane | Space: Stage | c: Commit | r: Refresh | Ctrl+p: Command Palette",
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(config.theme.fg.into()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.theme.border.into())),
        );
    f.render_widget(footer, area);
}

pub fn render_status<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let is_focused = state.active_pane == Pane::Status;
    let border_style = if is_focused {
        Style::default().fg(config.theme.accent.into())
    } else {
        Style::default().fg(config.theme.border.into())
    };

    let staged = state.files.iter().filter(|f| f.staged).count();
    let unstaged = state.files.len() - staged;

    let status_text = vec![
        Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(config.theme.fg.into())),
            Span::styled(
                &state.current_branch,
                Style::default()
                    .fg(config.theme.accent.into())
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Staged: {staged}"),
            Style::default().fg(config.theme.added.into()),
        )]),
        Line::from(vec![Span::styled(
            format!("Unstaged: {unstaged}"),
            Style::default().fg(config.theme.modified.into()),
        )]),
    ];

    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .title("Status")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(status, area);
}

pub fn render_files<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let is_focused = state.active_pane == Pane::Files;
    let border_style = if is_focused {
        Style::default().fg(config.theme.accent.into())
    } else {
        Style::default().fg(config.theme.border.into())
    };

    let items: Vec<ListItem> = state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let status_char = match file.status {
                FileStatus::Untracked => "??",
                FileStatus::Modified => "M ",
                FileStatus::Added => "A ",
                FileStatus::Deleted => "D ",
                FileStatus::Renamed => "R ",
                FileStatus::Conflicted => "U ",
            };

            let status_color = match file.status {
                FileStatus::Added => config.theme.added.into(),
                FileStatus::Modified => config.theme.modified.into(),
                FileStatus::Deleted => config.theme.removed.into(),
                FileStatus::Conflicted => config.theme.removed.into(),
                _ => config.theme.fg.into(),
            };

            let staged_marker = if file.staged { "[✓]" } else { "[ ]" };

            let mut style = Style::default().fg(config.theme.fg.into());
            if i == state.selected_index && is_focused {
                style = style.bg(config.theme.selection.into());
            }

            let line = Line::from(vec![
                Span::styled(
                    staged_marker,
                    Style::default().fg(config.theme.accent.into()),
                ),
                Span::raw(" "),
                Span::styled(status_char, Style::default().fg(status_color)),
                Span::raw(" "),
                Span::styled(&file.path, style),
            ]);

            ListItem::new(line)
        })
        .collect();

    let files_list = List::new(items).block(
        Block::default()
            .title("Files")
            .borders(Borders::ALL)
            .border_style(border_style),
    );

    f.render_widget(files_list, area);
}

pub fn render_diff<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let is_focused = state.active_pane == Pane::Diff;
    let border_style = if is_focused {
        Style::default().fg(config.theme.accent.into())
    } else {
        Style::default().fg(config.theme.border.into())
    };

    let lines: Vec<Line> = state
        .diff_content
        .lines()
        .map(|line| {
            let style = if line.starts_with('+') {
                Style::default().fg(config.theme.added.into())
            } else if line.starts_with('-') {
                Style::default().fg(config.theme.removed.into())
            } else if line.starts_with("@@") {
                Style::default()
                    .fg(config.theme.accent.into())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(config.theme.fg.into())
            };

            Line::from(Span::styled(line, style))
        })
        .collect();

    let diff = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Diff")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(diff, area);
}

pub fn render_branches<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let is_focused = state.active_pane == Pane::Branches || state.active_pane == Pane::Commits;
    let border_style = if is_focused {
        Style::default().fg(config.theme.accent.into())
    } else {
        Style::default().fg(config.theme.border.into())
    };

    let lines: Vec<Line> = if state.branch_graph.is_empty() {
        state
            .branches
            .iter()
            .map(|branch| {
                let style = if branch.starts_with('*') {
                    Style::default()
                        .fg(config.theme.accent.into())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(config.theme.fg.into())
                };
                Line::from(Span::styled(branch, style))
            })
            .collect()
    } else {
        state
            .branch_graph
            .iter()
            .map(|line| {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(config.theme.fg.into()),
                ))
            })
            .collect()
    };

    let branches = Paragraph::new(lines)
        .block(
            Block::default()
                .title("Branches")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(branches, area);
}

pub fn render_command_palette<'a>(
    f: &mut Frame,
    area: Rect,
    state: &AppState<'a>,
    config: &Config,
) {
    let popup_area = centered_rect(60, 20, area);

    f.render_widget(Clear, popup_area);

    let input_text = format!("> {}", state.command_input);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(config.theme.fg.into()))
        .block(
            Block::default()
                .title("Command Palette")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.theme.accent.into())),
        );

    f.render_widget(input, popup_area);
}

pub fn render_commit_editor<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    let popup_area = centered_rect(70, 40, area);

    f.render_widget(Clear, popup_area);

    let editor = Paragraph::new(state.commit_message.as_str())
        .style(Style::default().fg(config.theme.fg.into()))
        .block(
            Block::default()
                .title("Commit Message (Ctrl+Enter to commit, Esc to cancel)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(config.theme.accent.into())),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(editor, popup_area);
}

pub fn render_notifications<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    if state.notifications.is_empty() {
        return;
    }

    let notification_height = state.notifications.len().min(5) as u16 + 2;
    let notification_area = Rect {
        x: area.width.saturating_sub(50),
        y: area.height.saturating_sub(notification_height + 3),
        width: 48,
        height: notification_height,
    };

    let items: Vec<ListItem> = state
        .notifications
        .iter()
        .map(|notif| {
            let color = match notif.level {
                NotificationLevel::Info => config.theme.fg.into(),
                NotificationLevel::Success => config.theme.added.into(),
                NotificationLevel::Warning => config.theme.modified.into(),
                NotificationLevel::Error => config.theme.removed.into(),
            };

            ListItem::new(Line::from(Span::styled(
                &notif.message,
                Style::default().fg(color),
            )))
        })
        .collect();

    let notifications = List::new(items).block(
        Block::default()
            .title("Notifications")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(config.theme.border.into())),
    );

    f.render_widget(Clear, notification_area);
    f.render_widget(notifications, notification_area);
}

pub fn render_jobs<'a>(f: &mut Frame, area: Rect, state: &AppState<'a>, config: &Config) {
    if state.jobs.is_empty() {
        return;
    }

    let job_height = state.jobs.len() as u16 + 2;
    let job_area = Rect {
        x: 2,
        y: area.height.saturating_sub(job_height + 3),
        width: 40,
        height: job_height,
    };

    let items: Vec<ListItem> = state
        .jobs
        .iter()
        .map(|job| {
            let text = if let Some(progress) = job.progress {
                format!("{} ({}%)", job.description, (progress * 100.0) as u8)
            } else {
                format!("{} ...", job.description)
            };

            ListItem::new(Line::from(Span::styled(
                text,
                Style::default().fg(config.theme.accent.into()),
            )))
        })
        .collect();

    let jobs_list = List::new(items).block(
        Block::default()
            .title("Jobs")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(config.theme.border.into())),
    );

    f.render_widget(Clear, job_area);
    f.render_widget(jobs_list, job_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
