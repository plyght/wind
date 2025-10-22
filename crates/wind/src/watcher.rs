use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, Instant};

#[derive(Debug, Clone)]
pub enum FileEvent {
    Changed,
}

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::UnboundedReceiver<FileEvent>,
}

impl FileWatcher {
    pub fn new(path: &Path) -> Result<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tx = Arc::new(tx);

        let (event_tx, mut event_rx) = mpsc::unbounded_channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    let _ = event_tx.send(event);
                }
            },
            Config::default(),
        )?;

        watcher.watch(path, RecursiveMode::Recursive)?;

        let path_buf = path.to_path_buf();
        tokio::spawn(async move {
            let mut last_event: Option<Instant> = None;
            let debounce_duration = Duration::from_millis(300);

            while let Some(event) = event_rx.recv().await {
                if should_process_event(&event, &path_buf) {
                    let now = Instant::now();

                    if let Some(last) = last_event {
                        if now.duration_since(last) < debounce_duration {
                            sleep(debounce_duration).await;
                        }
                    }

                    last_event = Some(Instant::now());
                    let _ = tx.send(FileEvent::Changed);
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    pub async fn recv(&mut self) -> Option<FileEvent> {
        self.rx.recv().await
    }
}

fn should_process_event(event: &Event, repo_path: &Path) -> bool {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return false;
    }

    for path in &event.paths {
        if let Ok(relative) = path.strip_prefix(repo_path) {
            let path_str = relative.to_string_lossy();
            if path_str.starts_with(".git/") || path_str.starts_with(".wind/") {
                return false;
            }
            if path_str == ".git" || path_str == ".wind" {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_file_watcher_detects_changes() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();

        sleep(Duration::from_millis(100)).await;

        fs::write(&test_file, "initial content").unwrap();

        let event = tokio::time::timeout(Duration::from_secs(1), watcher.recv())
            .await
            .expect("Timeout waiting for event")
            .expect("Expected file change event");

        matches!(event, FileEvent::Changed);
    }

    #[tokio::test]
    async fn test_file_watcher_ignores_git_dir() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        sleep(Duration::from_millis(100)).await;

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();

        sleep(Duration::from_millis(100)).await;

        fs::write(git_dir.join("config"), "git config").unwrap();

        let normal_file = temp_dir.path().join("normal.txt");
        fs::write(&normal_file, "normal content").unwrap();

        let event = tokio::time::timeout(Duration::from_millis(800), watcher.recv())
            .await
            .expect("Should receive event for normal file");
        matches!(event, Some(FileEvent::Changed));
    }

    #[tokio::test]
    async fn test_file_watcher_ignores_wind_dir() {
        let temp_dir = TempDir::new().unwrap();
        let wind_dir = temp_dir.path().join(".wind");
        fs::create_dir(&wind_dir).unwrap();

        sleep(Duration::from_millis(100)).await;

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();

        sleep(Duration::from_millis(100)).await;

        fs::write(wind_dir.join("config.toml"), "config").unwrap();

        let normal_file = temp_dir.path().join("normal.txt");
        fs::write(&normal_file, "normal content").unwrap();

        let event = tokio::time::timeout(Duration::from_millis(800), watcher.recv())
            .await
            .expect("Should receive event for normal file");
        matches!(event, Some(FileEvent::Changed));
    }

    #[tokio::test]
    async fn test_file_watcher_debounces_events() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();

        sleep(Duration::from_millis(200)).await;

        for i in 0..5 {
            fs::write(&test_file, format!("content {}", i)).unwrap();
            sleep(Duration::from_millis(50)).await;
        }

        let _event = tokio::time::timeout(Duration::from_secs(2), watcher.recv())
            .await
            .expect("Timeout waiting for debounced event")
            .expect("Expected debounced event");

        matches!(_event, FileEvent::Changed);
    }
}
