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
        
        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        fs::write(git_dir.join("config"), "git config").unwrap();
        
        let result = tokio::time::timeout(Duration::from_millis(500), watcher.recv()).await;
        assert!(result.is_err(), "Should not receive event for .git changes");
    }

    #[tokio::test]
    async fn test_file_watcher_ignores_wind_dir() {
        let temp_dir = TempDir::new().unwrap();
        let wind_dir = temp_dir.path().join(".wind");
        fs::create_dir(&wind_dir).unwrap();
        
        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        fs::write(wind_dir.join("config.toml"), "config").unwrap();
        
        let result = tokio::time::timeout(Duration::from_millis(500), watcher.recv()).await;
        assert!(result.is_err(), "Should not receive event for .wind changes");
    }

    #[tokio::test]
    async fn test_file_watcher_debounces_events() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let mut watcher = FileWatcher::new(temp_dir.path()).unwrap();
        
        sleep(Duration::from_millis(100)).await;
        
        for i in 0..5 {
            fs::write(&test_file, format!("content {}", i)).unwrap();
            sleep(Duration::from_millis(50)).await;
        }
        
        let start = std::time::Instant::now();
        let _event = tokio::time::timeout(Duration::from_secs(2), watcher.recv())
            .await
            .expect("Timeout waiting for debounced event")
            .expect("Expected debounced event");
        let elapsed = start.elapsed();
        
        assert!(elapsed >= Duration::from_millis(300), "Should debounce events");
    }
}
