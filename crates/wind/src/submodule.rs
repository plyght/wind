use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Submodule {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub initialized: bool,
}

pub fn has_submodules(repo_path: &Path) -> Result<bool> {
    Ok(repo_path.join(".gitmodules").exists())
}

pub fn is_inside_submodule(path: &Path) -> Result<bool> {
    let mut current = path.to_path_buf();

    loop {
        let parent = match current.parent() {
            Some(p) => p,
            None => break,
        };

        if parent.join(".gitmodules").exists() {
            let git_path = current.join(".git");
            if git_path.exists() {
                return Ok(true);
            }
        }

        current = parent.to_path_buf();
    }

    Ok(false)
}

pub fn list_submodules(repo_path: &Path) -> Result<Vec<Submodule>> {
    let gitmodules_path = repo_path.join(".gitmodules");
    if !gitmodules_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&gitmodules_path).context("Failed to read .gitmodules")?;

    let mut submodules = Vec::new();
    let mut current_name = None;
    let mut current_path = None;
    let mut current_url = None;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("[submodule") {
            if let Some(name) = current_name.take() {
                if let (Some(path), Some(url)) = (current_path.take(), current_url.take()) {
                    let full_path = repo_path.join(&path);
                    let initialized = full_path.join(".git").exists();
                    submodules.push(Submodule {
                        name,
                        path: PathBuf::from(path),
                        url,
                        initialized,
                    });
                }
            }

            let name_start = line.find('"').unwrap_or(0) + 1;
            let name_end = line.rfind('"').unwrap_or(line.len());
            current_name = Some(line[name_start..name_end].to_string());
        } else if let Some(path_val) = line.strip_prefix("path = ") {
            current_path = Some(path_val.to_string());
        } else if let Some(url_val) = line.strip_prefix("url = ") {
            current_url = Some(url_val.to_string());
        }
    }

    if let Some(name) = current_name {
        if let (Some(path), Some(url)) = (current_path, current_url) {
            let full_path = repo_path.join(&path);
            let initialized = full_path.join(".git").exists();
            submodules.push(Submodule {
                name,
                path: PathBuf::from(path),
                url,
                initialized,
            });
        }
    }

    Ok(submodules)
}

pub fn get_submodule_status(repo_path: &Path, submodule: &Submodule) -> Result<String> {
    let submodule_path = repo_path.join(&submodule.path);

    if !submodule.initialized {
        return Ok("not initialized".to_string());
    }

    if !submodule_path.exists() {
        return Ok("missing".to_string());
    }

    let git_path = submodule_path.join(".git");
    if !git_path.exists() {
        return Ok("not initialized".to_string());
    }

    Ok("initialized".to_string())
}
