use anyhow::Result;
use std::fs;
use tempfile::TempDir;
use wind::{FileStatus, UnifiedRepository};

#[test]
fn test_init_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file.txt"), "content")?;
    repo.add(vec![repo_path.join("file.txt")])?;

    let commit_id = repo.commit("test commit")?;
    assert!(!commit_id.is_empty());

    let changesets = repo.log(10)?;
    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].commit_message, "test commit");

    Ok(())
}

#[test]
fn test_rename_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("old_name.txt"), "unique content")?;
    repo.add(vec![repo_path.join("old_name.txt")])?;
    repo.commit("add file")?;

    fs::rename(
        repo_path.join("old_name.txt"),
        repo_path.join("new_name.txt"),
    )?;

    let status = repo.status()?;

    let has_rename = status
        .iter()
        .any(|s| matches!(s.status, FileStatus::Renamed { .. }));
    assert!(has_rename, "Should detect rename via NodeID");

    Ok(())
}

#[test]
fn test_merge() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file.txt"), "base content")?;
    repo.add(vec![repo_path.join("file.txt")])?;
    let base_commit = repo.commit("base")?;

    fs::write(repo_path.join("file.txt"), "branch content")?;
    repo.add(vec![repo_path.join("file.txt")])?;
    let branch_commit = repo.commit("branch change")?;

    let merge_result = repo.merge(branch_commit)?;

    match merge_result {
        wind::MergeResult::Clean { new_changeset_id } => {
            assert!(!new_changeset_id.is_empty());
        }
        wind::MergeResult::Conflicts { conflicts } => {
            assert!(!conflicts.is_empty(), "Conflicts detected properly");
        }
    }

    Ok(())
}

#[test]
fn test_storage_verification() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("test.txt"), "test data")?;
    repo.add(vec![repo_path.join("test.txt")])?;
    let commit_id = repo.commit("store test")?;

    let wind_dir = repo_path.join(".wind");
    assert!(wind_dir.exists());
    assert!(wind_dir.join("storage").exists());
    assert!(wind_dir.join("index.db").exists());

    let changesets = repo.log(1)?;
    assert_eq!(changesets.len(), 1);

    Ok(())
}

#[test]
fn test_multiple_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("a.txt"), "a")?;
    fs::write(repo_path.join("b.txt"), "b")?;
    fs::write(repo_path.join("c.txt"), "c")?;

    repo.add(vec![
        repo_path.join("a.txt"),
        repo_path.join("b.txt"),
        repo_path.join("c.txt"),
    ])?;

    repo.commit("add multiple files")?;

    let changesets = repo.log(1)?;
    assert_eq!(changesets[0].changes.len(), 3);

    Ok(())
}

#[test]
fn test_branch_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file.txt"), "content")?;
    repo.add(vec![repo_path.join("file.txt")])?;
    repo.commit("initial")?;

    let branches = repo.branches()?;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");
    assert!(!branches[0].head.is_empty());

    Ok(())
}

#[test]
fn test_empty_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let commit_id = repo.commit("empty commit")?;
    assert!(!commit_id.is_empty());

    let changesets = repo.log(1)?;
    assert_eq!(changesets[0].changes.len(), 0);

    Ok(())
}
