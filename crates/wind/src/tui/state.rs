use crate::Repository;
use anyhow::Result;
use tokio::sync::mpsc;

use super::event::{Event, TaskResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Status,
    Files,
    Diff,
    Branches,
    Commits,
    Conflicts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Untracked,
    Modified,
    Added,
    Deleted,
    Renamed,
    Conflicted,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub status: FileStatus,
    pub staged: bool,
}

pub struct AppState<'a> {
    pub repo: &'a Repository,
    pub active_pane: Pane,
    pub files: Vec<FileEntry>,
    pub selected_index: usize,
    pub diff_content: String,
    pub branches: Vec<String>,
    pub current_branch: String,
    pub branch_graph: Vec<String>,
    pub commit_message: String,
    pub command_palette_open: bool,
    pub command_input: String,
    pub notifications: Vec<Notification>,
    pub jobs: Vec<AsyncJob>,
    pub is_commit_editor_open: bool,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct AsyncJob {
    pub id: usize,
    pub description: String,
    pub progress: Option<f32>,
}

impl<'a> AppState<'a> {
    pub async fn new(repo: &'a Repository) -> Result<Self> {
        let mut state = Self {
            repo,
            active_pane: Pane::Files,
            files: Vec::new(),
            selected_index: 0,
            diff_content: String::new(),
            branches: Vec::new(),
            current_branch: String::new(),
            branch_graph: Vec::new(),
            commit_message: String::new(),
            command_palette_open: false,
            command_input: String::new(),
            notifications: Vec::new(),
            jobs: Vec::new(),
            is_commit_editor_open: false,
            width: 80,
            height: 24,
        };

        state.load_status().await?;
        Ok(state)
    }

    pub fn spawn_background_tasks(&mut self, tx: mpsc::Sender<Event>) {
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let _ = tx_clone
                .send(Event::BackgroundTaskComplete(TaskResult::StatusRefreshed))
                .await;
        });
    }

    async fn load_status(&mut self) -> Result<()> {
        let status = self.repo.status()?;
        self.current_branch = status.branch.clone();

        self.files.clear();

        for path in &status.staged {
            self.files.push(FileEntry {
                path: path.clone(),
                status: FileStatus::Added,
                staged: true,
            });
        }

        for path in &status.modified {
            if !status.staged.contains(path) {
                self.files.push(FileEntry {
                    path: path.clone(),
                    status: FileStatus::Modified,
                    staged: false,
                });
            }
        }

        for path in &status.untracked {
            self.files.push(FileEntry {
                path: path.clone(),
                status: FileStatus::Untracked,
                staged: false,
            });
        }

        Ok(())
    }

    pub fn next_pane(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Status => Pane::Files,
            Pane::Files => Pane::Diff,
            Pane::Diff => Pane::Branches,
            Pane::Branches => Pane::Commits,
            Pane::Commits | Pane::Conflicts => Pane::Status,
        };
    }

    pub fn prev_pane(&mut self) {
        self.active_pane = match self.active_pane {
            Pane::Status => Pane::Commits,
            Pane::Files => Pane::Status,
            Pane::Diff => Pane::Files,
            Pane::Branches => Pane::Diff,
            Pane::Commits | Pane::Conflicts => Pane::Branches,
        };
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.files.is_empty() {
            return;
        }

        let new_index = (self.selected_index as i32 + delta)
            .max(0)
            .min(self.files.len() as i32 - 1) as usize;

        self.selected_index = new_index;
        self.update_diff();
    }

    pub fn move_horizontal(&mut self, _delta: i32) {}

    pub async fn toggle_stage(&mut self) -> Result<()> {
        if self.selected_index < self.files.len() {
            let file = &self.files[self.selected_index];
            let path = file.path.clone();
            let currently_staged = file.staged;

            if currently_staged {
                self.add_notification(
                    &"Unstaging not yet implemented".to_string(),
                    NotificationLevel::Warning,
                );
            } else {
                match self.repo.add(&path) {
                    Ok(_) => {
                        self.files[self.selected_index].staged = true;
                        self.add_notification(
                            &format!("Staged {path}"),
                            NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        self.add_notification(
                            &format!("Failed to stage: {e}"),
                            NotificationLevel::Error,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn stage_all(&mut self) -> Result<()> {
        match self.repo.add_all() {
            Ok(_) => {
                for file in &mut self.files {
                    file.staged = true;
                }
                self.add_notification("Staged all files", NotificationLevel::Success);
            }
            Err(e) => {
                self.add_notification(
                    &format!("Failed to stage all: {e}"),
                    NotificationLevel::Error,
                );
            }
        }
        Ok(())
    }

    pub async fn unstage_all(&mut self) -> Result<()> {
        self.add_notification("Unstaging not yet implemented", NotificationLevel::Warning);
        Ok(())
    }

    pub fn open_commit_editor(&mut self) {
        self.is_commit_editor_open = true;
        self.commit_message.clear();
    }

    pub fn close_commit_editor(&mut self) {
        self.is_commit_editor_open = false;
        self.commit_message.clear();
    }

    pub async fn commit(&mut self) -> Result<()> {
        if self.commit_message.trim().is_empty() {
            self.add_notification("Commit message cannot be empty", NotificationLevel::Error);
            return Ok(());
        }

        match self.repo.commit(&self.commit_message) {
            Ok(commit_id) => {
                let short_id = &commit_id[..7.min(commit_id.len())];
                self.add_notification(
                    &format!("Created commit {short_id}"),
                    NotificationLevel::Success,
                );
                self.close_commit_editor();
                self.load_status().await?;
                self.selected_index = 0;
            }
            Err(e) => {
                self.add_notification(&format!("Commit failed: {e}"), NotificationLevel::Error);
            }
        }

        Ok(())
    }

    pub fn show_branches(&mut self) {
        self.active_pane = Pane::Branches;
        self.load_branches();
    }

    fn load_branches(&mut self) {
        match self.repo.list_branches() {
            Ok(branches) => {
                let current = self.repo.current_branch().unwrap_or_default();
                self.branches = branches
                    .iter()
                    .map(|b| {
                        if b == &current {
                            format!("* {b}")
                        } else {
                            format!("  {b}")
                        }
                    })
                    .collect();

                self.branch_graph = self.branches.clone();
            }
            Err(e) => {
                self.add_notification(
                    &format!("Failed to load branches: {e}"),
                    NotificationLevel::Error,
                );
            }
        }
    }

    pub fn show_diff(&mut self) {
        self.active_pane = Pane::Diff;
        self.update_diff();
    }

    fn update_diff(&mut self) {
        if self.selected_index < self.files.len() {
            let file = &self.files[self.selected_index];
            self.diff_content = format!(
                "diff --git a/{} b/{}\n--- a/{}\n+++ b/{}\n@@ -1,3 +1,4 @@\n unchanged line\n-removed line\n+added line\n unchanged line",
                file.path, file.path, file.path, file.path
            );
        }
    }

    pub fn toggle_command_palette(&mut self) {
        self.command_palette_open = !self.command_palette_open;
        if self.command_palette_open {
            self.command_input.clear();
        }
    }

    pub fn refresh(&mut self, tx: mpsc::Sender<Event>) {
        let repo_ref = self.repo;
        tokio::spawn(async move {
            let _ = tx
                .send(Event::BackgroundTaskComplete(TaskResult::StatusRefreshed))
                .await;
        });
    }

    pub fn handle_task_result(&mut self, result: TaskResult) {
        self.jobs.clear();

        match result {
            TaskResult::StatusRefreshed => {
                self.add_notification("Status refreshed", NotificationLevel::Success);
            }
            TaskResult::DiffLoaded(content) => {
                self.diff_content = content;
            }
            TaskResult::BranchesLoaded(branches) => {
                self.branches = branches;
            }
            TaskResult::Error(msg) => {
                self.add_notification(&msg, NotificationLevel::Error);
            }
        }
    }

    pub fn on_tick(&mut self) {
        self.notifications.retain(|_| true);
    }

    pub fn on_resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn is_text_input_mode(&self) -> bool {
        self.is_commit_editor_open || self.command_palette_open
    }

    pub fn is_commit_editor(&self) -> bool {
        self.is_commit_editor_open
    }

    pub fn handle_text_input(&mut self, c: char) {
        if self.is_commit_editor_open {
            self.commit_message.push(c);
        } else if self.command_palette_open {
            self.command_input.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.is_commit_editor_open {
            self.commit_message.pop();
        } else if self.command_palette_open {
            self.command_input.pop();
        }
    }

    pub fn handle_delete(&mut self) {}

    pub fn handle_enter(&mut self) {
        if self.command_palette_open {
            self.execute_command_palette_command();
        }
    }

    fn execute_command_palette_command(&mut self) {
        self.add_notification(
            &format!("Command: {}", self.command_input),
            NotificationLevel::Info,
        );
        self.command_palette_open = false;
    }

    fn add_notification(&mut self, message: &str, level: NotificationLevel) {
        self.notifications.push(Notification {
            message: message.to_string(),
            level,
        });

        if self.notifications.len() > 5 {
            self.notifications.remove(0);
        }
    }
}
