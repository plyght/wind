use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub is_main: bool,
}

pub fn is_worktree(path: &Path) -> Result<bool> {
    let git_path = path.join(".git");
    if !git_path.exists() {
        return Ok(false);
    }

    if git_path.is_file() {
        let content = fs::read_to_string(&git_path)?;
        Ok(content.starts_with("gitdir:"))
    } else {
        Ok(false)
    }
}

pub fn get_gitdir(path: &Path) -> Result<PathBuf> {
    let git_file = path.join(".git");
    if git_file.is_file() {
        let content = fs::read_to_string(&git_file)?;
        if let Some(gitdir_line) = content.lines().next() {
            if let Some(gitdir) = gitdir_line.strip_prefix("gitdir: ") {
                let gitdir_path = PathBuf::from(gitdir.trim());
                return if gitdir_path.is_absolute() {
                    Ok(gitdir_path)
                } else {
                    Ok(path.join(gitdir_path))
                };
            }
        }
    }
    anyhow::bail!("Not a worktree or invalid .git file format")
}

pub fn list_worktrees(repo_path: &Path) -> Result<Vec<Worktree>> {
    let git_dir = if repo_path.join(".git").is_dir() {
        repo_path.join(".git")
    } else if repo_path.join(".git").is_file() {
        get_gitdir(repo_path)?
    } else {
        anyhow::bail!("Not a git repository")
    };

    let worktrees_dir = git_dir.parent().unwrap().join(".git/worktrees");
    let mut worktrees = Vec::new();

    worktrees.push(Worktree {
        path: git_dir.parent().unwrap().to_path_buf(),
        branch: get_head_branch(&git_dir.join("HEAD"))?,
        is_main: true,
    });

    if worktrees_dir.exists() {
        for entry in fs::read_dir(&worktrees_dir)? {
            let entry = entry?;
            let worktree_name = entry.file_name();
            let gitdir_file = entry.path().join("gitdir");

            if !gitdir_file.exists() {
                continue;
            }

            let worktree_path = fs::read_to_string(&gitdir_file)?
                .trim()
                .strip_suffix("/.git")
                .unwrap_or("")
                .to_string();

            if worktree_path.is_empty() {
                continue;
            }

            let head_file = entry.path().join("HEAD");
            let branch = get_head_branch(&head_file)?;

            worktrees.push(Worktree {
                path: PathBuf::from(worktree_path),
                branch,
                is_main: false,
            });
        }
    }

    Ok(worktrees)
}

fn get_head_branch(head_path: &Path) -> Result<Option<String>> {
    if !head_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(head_path)?;
    if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
        Ok(Some(branch.trim().to_string()))
    } else {
        Ok(None)
    }
}

pub fn is_branch_checked_out(repo_path: &Path, branch_name: &str) -> Result<bool> {
    let worktrees = list_worktrees(repo_path)?;
    for wt in worktrees {
        if let Some(br) = wt.branch {
            if br == branch_name {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
