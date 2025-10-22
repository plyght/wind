use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

#[cfg(test)]
use tempfile::TempDir;

struct TestRepo {
    dir: TempDir,
    path: PathBuf,
}

impl TestRepo {
    fn new() -> Result<Self> {
        let dir = TempDir::new()?;
        let path = dir.path().to_path_buf();
        Ok(Self { dir, path })
    }

    fn wind(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(env!("CARGO_BIN_EXE_wind"))
            .args(args)
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "wind command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn write_file(&self, name: &str, content: &str) -> Result<()> {
        std::fs::write(self.path.join(name), content)?;
        Ok(())
    }
}

#[test]
fn test_init_creates_wind_directory() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    assert!(repo.path.join(".wind").exists());
    assert!(repo.path.join(".git").exists());

    Ok(())
}

#[test]
fn test_init_with_git_compatibility() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    let git_status = repo.git(&["status"])?;
    assert!(git_status.contains("On branch"));

    Ok(())
}

#[test]
fn test_status_empty_repo() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    let status = repo.wind(&["status"])?;
    assert!(status.contains("nothing to commit"));

    Ok(())
}

#[test]
fn test_add_and_commit() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;
    repo.write_file("test.txt", "hello world")?;

    repo.wind(&["add", "test.txt"])?;
    let status = repo.wind(&["status"])?;
    assert!(status.contains("Changes to be committed"));

    repo.wind(&["commit", "-m", "Initial commit"])?;
    let log = repo.wind(&["log"])?;
    assert!(log.contains("Initial commit"));

    Ok(())
}

#[test]
fn test_branch_operations() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;
    repo.write_file("test.txt", "content")?;
    repo.wind(&["add", "test.txt"])?;
    repo.wind(&["commit", "-m", "First"])?;

    repo.wind(&["branch", "feature"])?;
    let branches = repo.wind(&["branch", "--list"])?;
    assert!(branches.contains("feature"));

    repo.wind(&["checkout", "feature"])?;
    let status = repo.wind(&["status"])?;
    assert!(status.contains("feature"));

    Ok(())
}

#[test]
fn test_wind_git_roundtrip() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;
    repo.write_file("file.txt", "wind content")?;
    repo.wind(&["add", "file.txt"])?;
    repo.wind(&["commit", "-m", "Via wind"])?;

    let git_log = repo.git(&["log", "--oneline"])?;
    assert!(git_log.contains("Via wind"));

    repo.write_file("file2.txt", "git content")?;
    repo.git(&["add", "file2.txt"])?;
    repo.git(&["commit", "-m", "Via git"])?;

    let wind_log = repo.wind(&["log"])?;
    assert!(wind_log.contains("Via wind"));
    assert!(wind_log.contains("Via git"));

    Ok(())
}

#[test]
fn test_log_with_limit() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    for i in 1..=5 {
        repo.write_file(&format!("file{}.txt", i), &format!("content {}", i))?;
        repo.wind(&["add", "-a"])?;
        repo.wind(&["commit", "-m", &format!("Commit {}", i)])?;
    }

    let log = repo.wind(&["log", "-n", "2"])?;
    let commit_count = log.matches("commit").count();
    assert_eq!(commit_count, 2);

    Ok(())
}

#[test]
fn test_status_staged_vs_unstaged() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    repo.write_file("staged.txt", "staged")?;
    repo.wind(&["add", "staged.txt"])?;

    repo.write_file("unstaged.txt", "unstaged")?;

    let status = repo.wind(&["status"])?;
    assert!(status.contains("Changes to be committed"));
    assert!(status.contains("staged.txt"));
    assert!(status.contains("Untracked files"));
    assert!(status.contains("unstaged.txt"));

    Ok(())
}

#[test]
fn test_branch_deletion() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;
    repo.write_file("init.txt", "init")?;
    repo.wind(&["add", "-a"])?;
    repo.wind(&["commit", "-m", "Initial"])?;

    repo.wind(&["branch", "temp"])?;
    repo.wind(&["branch", "-d", "temp"])?;

    let branches = repo.wind(&["branch", "--list"])?;
    assert!(!branches.contains("temp"));

    Ok(())
}

#[test]
fn test_config_operations() -> Result<()> {
    let repo = TestRepo::new()?;
    repo.wind(&["init"])?;

    repo.wind(&["config", "set", "user.name", "Test User"])?;
    let value = repo.wind(&["config", "get", "user.name"])?;
    assert!(value.contains("Test User"));

    Ok(())
}
