use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tracing::info;

pub fn install_hooks<P: AsRef<Path>>(repo_path: P) -> Result<()> {
    let hooks_dir = repo_path.as_ref().join(".git").join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    install_post_commit_hook(&hooks_dir)?;
    install_post_merge_hook(&hooks_dir)?;
    install_post_checkout_hook(&hooks_dir)?;

    info!("Git hooks installed successfully");
    Ok(())
}

fn install_post_commit_hook(hooks_dir: &Path) -> Result<()> {
    let hook_path = hooks_dir.join("post-commit");
    let content = r#"#!/bin/sh
wind sync --quiet || true
"#;
    write_hook(&hook_path, content)?;
    Ok(())
}

fn install_post_merge_hook(hooks_dir: &Path) -> Result<()> {
    let hook_path = hooks_dir.join("post-merge");
    let content = r#"#!/bin/sh
wind sync --quiet || true
"#;
    write_hook(&hook_path, content)?;
    Ok(())
}

fn install_post_checkout_hook(hooks_dir: &Path) -> Result<()> {
    let hook_path = hooks_dir.join("post-checkout");
    let content = r#"#!/bin/sh
wind sync --quiet || true
"#;
    write_hook(&hook_path, content)?;
    Ok(())
}

fn write_hook(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }

    Ok(())
}
