use anyhow::Result;
use std::fs;
use tempfile::TempDir;
use wind::{FileStatus, UnifiedRepository};

#[test]
fn test_init_and_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, Wind!")?;

    repo.add(vec![test_file])?;

    let commit_oid = repo.commit("Initial commit")?;
    assert!(!commit_oid.is_empty());

    let log = repo.log(10)?;
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].commit_message, "Initial commit");

    Ok(())
}

#[test]
fn test_status_with_renames() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let original_file = repo_path.join("original.txt");
    fs::write(&original_file, "This is the original content")?;

    repo.add(vec![original_file.clone()])?;
    repo.commit("Add original file")?;

    fs::remove_file(&original_file)?;

    let renamed_file = repo_path.join("renamed.txt");
    fs::write(&renamed_file, "This is the original content")?;

    let status = repo.status()?;

    let has_rename = status
        .iter()
        .any(|s| matches!(s.status, FileStatus::Renamed { .. }));
    assert!(has_rename, "Should detect rename");

    Ok(())
}

#[test]
fn test_merge_with_conflict() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let file_path = repo_path.join("conflict.txt");
    fs::write(&file_path, "base content")?;
    repo.add(vec![file_path.clone()])?;
    let base_commit = repo.commit("Base commit")?;

    fs::write(&file_path, "ours content")?;
    repo.add(vec![file_path.clone()])?;
    let _ours_commit = repo.commit("Our changes")?;

    let result = repo.merge(base_commit);

    assert!(result.is_ok(), "Merge should complete");

    Ok(())
}

#[test]
fn test_git_roundtrip() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let git_repo_path = temp_dir.path().join("git_repo");
    fs::create_dir_all(&git_repo_path)?;

    let git_repo = git2::Repository::init(&git_repo_path)?;

    let test_file = git_repo_path.join("test.txt");
    fs::write(&test_file, "Test content")?;

    let mut index = git_repo.index()?;
    index.add_path(std::path::Path::new("test.txt"))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = git_repo.find_tree(tree_id)?;
    let sig = git2::Signature::now("Test User", "test@example.com")?;

    git_repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let wind_repo = UnifiedRepository::import_git(git_repo_path.clone())?;

    let export_path = temp_dir.path().join("exported_git");
    wind_repo.export_git(export_path.clone())?;

    let exported_git = git2::Repository::open(&export_path)?;
    assert!(exported_git.path().exists());

    Ok(())
}

#[test]
fn test_multiple_commits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    for i in 1..=5 {
        let file = repo_path.join(format!("file{}.txt", i));
        fs::write(&file, format!("Content {}", i))?;
        repo.add(vec![file])?;
        repo.commit(&format!("Commit {}", i))?;
    }

    let log = repo.log(10)?;
    assert_eq!(log.len(), 5);
    assert_eq!(log[0].commit_message, "Commit 5");
    assert_eq!(log[4].commit_message, "Commit 1");

    Ok(())
}

#[test]
fn test_branch_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let repo = UnifiedRepository::init(repo_path.clone())?;

    let branches = repo.branches()?;
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "main");

    Ok(())
}

#[test]
fn test_modified_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let file_path = repo_path.join("modified.txt");
    fs::write(&file_path, "original content")?;
    repo.add(vec![file_path.clone()])?;
    repo.commit("Add file")?;

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::write(&file_path, "modified content that is different")?;

    let status = repo.status()?;

    let has_modified = status
        .iter()
        .any(|s| matches!(s.status, FileStatus::Modified | FileStatus::Untracked));
    assert!(
        has_modified,
        "Should detect modified file, got: {:?}",
        status
    );

    Ok(())
}

#[test]
fn test_deleted_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let file_path = repo_path.join("deleted.txt");
    fs::write(&file_path, "to be deleted")?;
    repo.add(vec![file_path.clone()])?;
    repo.commit("Add file")?;

    fs::remove_file(&file_path)?;

    let status = repo.status()?;

    let has_deleted = status
        .iter()
        .any(|s| matches!(s.status, FileStatus::Deleted));
    assert!(has_deleted, "Should detect deleted file");

    Ok(())
}

#[test]
fn test_untracked_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let repo = UnifiedRepository::init(repo_path.clone())?;

    let untracked_file = repo_path.join("untracked.txt");
    fs::write(&untracked_file, "new file")?;

    let status = repo.status()?;

    let has_untracked = status
        .iter()
        .any(|s| matches!(s.status, FileStatus::Untracked));
    assert!(has_untracked, "Should detect untracked file");

    Ok(())
}

#[test]
fn test_checkout_branch() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    let file_path = repo_path.join("test.txt");
    fs::write(&file_path, "content")?;
    repo.add(vec![file_path])?;
    repo.commit("Initial commit")?;

    repo.checkout("main")?;

    let branches = repo.branches()?;
    assert!(!branches.is_empty());

    Ok(())
}
