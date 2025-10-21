use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_worktree_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    fs::write(repo_path.join("test.txt"), "content")?;

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["branch", "feature"])
        .current_dir(repo_path)
        .output()?;

    let worktree_dir = temp.path().join("worktree");
    Command::new("git")
        .args(["worktree", "add", worktree_dir.to_str().unwrap(), "feature"])
        .current_dir(repo_path)
        .output()?;

    let is_main_worktree = wind_core::worktree::is_worktree(repo_path)?;
    assert!(
        !is_main_worktree,
        "Main repo should not be detected as worktree"
    );

    let is_worktree = wind_core::worktree::is_worktree(&worktree_dir)?;
    assert!(is_worktree, "Should detect worktree");

    let gitdir = wind_core::worktree::get_gitdir(&worktree_dir)?;
    assert!(gitdir.exists(), "Gitdir should exist");
    assert!(
        gitdir.to_string_lossy().contains("worktrees"),
        "Gitdir should be in worktrees directory"
    );

    Ok(())
}

#[test]
fn test_list_worktrees() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    fs::write(repo_path.join("test.txt"), "content")?;

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["branch", "feature1"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["branch", "feature2"])
        .current_dir(repo_path)
        .output()?;

    let worktree1 = temp.path().join("wt1");
    Command::new("git")
        .args(["worktree", "add", worktree1.to_str().unwrap(), "feature1"])
        .current_dir(repo_path)
        .output()?;

    let worktree2 = temp.path().join("wt2");
    Command::new("git")
        .args(["worktree", "add", worktree2.to_str().unwrap(), "feature2"])
        .current_dir(repo_path)
        .output()?;

    let worktrees = wind_core::worktree::list_worktrees(repo_path)?;
    assert_eq!(
        worktrees.len(),
        3,
        "Should have 3 worktrees (main + 2 additional)"
    );

    let main_wt = worktrees.iter().find(|wt| wt.is_main);
    assert!(main_wt.is_some(), "Should have main worktree");

    let feature_wts: Vec<_> = worktrees.iter().filter(|wt| !wt.is_main).collect();
    assert_eq!(feature_wts.len(), 2, "Should have 2 feature worktrees");

    Ok(())
}

#[test]
fn test_branch_checked_out_protection() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    fs::write(repo_path.join("test.txt"), "content")?;

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Command::new("git")
        .args(["branch", "feature"])
        .current_dir(repo_path)
        .output()?;

    let worktree_dir = temp.path().join("worktree");
    Command::new("git")
        .args(["worktree", "add", worktree_dir.to_str().unwrap(), "feature"])
        .current_dir(repo_path)
        .output()?;

    let is_checked_out = wind_core::worktree::is_branch_checked_out(repo_path, "feature")?;
    assert!(
        is_checked_out,
        "Feature branch should be checked out in worktree"
    );

    let is_main_checked_out = wind_core::worktree::is_branch_checked_out(repo_path, "main")?;
    assert!(
        is_main_checked_out,
        "Main branch should be checked out in main repo"
    );

    Ok(())
}

#[test]
fn test_worktree_operations() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    let repo = wind_core::Repository::init(repo_path)?;

    fs::write(repo_path.join("file.txt"), "test")?;
    repo.add("file.txt")?;
    repo.commit("Add file")?;

    repo.create_branch("feature")?;

    let worktree_dir = temp.path().join("worktree");
    Command::new("git")
        .args(["worktree", "add", worktree_dir.to_str().unwrap(), "feature"])
        .current_dir(repo_path)
        .output()?;

    let repo2 = wind_core::Repository::open(&worktree_dir)?;
    let status = repo2.status()?;

    assert!(status.is_worktree, "Should be detected as worktree");
    assert_eq!(status.branch, "feature");

    let delete_result = repo.delete_branch("feature");
    assert!(
        delete_result.is_err(),
        "Should not allow deleting branch checked out in worktree"
    );

    Ok(())
}
