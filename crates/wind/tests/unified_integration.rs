use anyhow::Result;
use std::fs;
use tempfile::TempDir;
use wind::UnifiedRepository;

#[test]
fn test_init_and_commit() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("test.txt"), "Hello, Wind!")?;

    repo.add(vec![repo_path.join("test.txt")])?;

    let commit_id = repo.commit("Initial commit")?;

    assert!(!commit_id.is_empty());

    let changesets = repo.log(10)?;
    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].commit_message, "Initial commit");

    Ok(())
}

#[test]
fn test_status_detection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file1.txt"), "content1")?;
    repo.add(vec![repo_path.join("file1.txt")])?;
    repo.commit("Add file1")?;

    fs::write(repo_path.join("file1.txt"), "modified content")?;

    let status = repo.status()?;
    assert!(!status.is_empty());

    Ok(())
}

#[test]
fn test_branch_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file.txt"), "content")?;
    repo.add(vec![repo_path.join("file.txt")])?;
    repo.commit("Initial commit")?;

    let branches = repo.branches()?;
    assert!(!branches.is_empty());
    assert_eq!(branches[0].name, "main");

    Ok(())
}

#[test]
fn test_open_existing_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    {
        let mut repo = UnifiedRepository::init(repo_path.clone())?;
        fs::write(repo_path.join("test.txt"), "test")?;
        repo.add(vec![repo_path.join("test.txt")])?;
        repo.commit("Test commit")?;
    }

    let repo = UnifiedRepository::open(repo_path.clone())?;
    let changesets = repo.log(10)?;
    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].commit_message, "Test commit");

    Ok(())
}

#[test]
fn test_multiple_commits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().to_path_buf();

    let mut repo = UnifiedRepository::init(repo_path.clone())?;

    fs::write(repo_path.join("file1.txt"), "first")?;
    repo.add(vec![repo_path.join("file1.txt")])?;
    repo.commit("First commit")?;

    fs::write(repo_path.join("file2.txt"), "second")?;
    repo.add(vec![repo_path.join("file2.txt")])?;
    repo.commit("Second commit")?;

    fs::write(repo_path.join("file3.txt"), "third")?;
    repo.add(vec![repo_path.join("file3.txt")])?;
    repo.commit("Third commit")?;

    let changesets = repo.log(10)?;
    assert_eq!(changesets.len(), 3);
    assert_eq!(changesets[0].commit_message, "Third commit");
    assert_eq!(changesets[1].commit_message, "Second commit");
    assert_eq!(changesets[2].commit_message, "First commit");

    Ok(())
}
