use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use wind::UnifiedRepository;

pub async fn execute(remote: String, branch: Option<String>) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join(".wind").exists() {
        anyhow::bail!("Not a Wind repository");
    }

    // Step 1: Export to Git
    println!("{}", "Exporting Wind changesets to Git...".cyan());
    let repo = UnifiedRepository::open(current_dir.clone())?;
    repo.export_git(current_dir.clone())?;

    // Step 2: Get branch name
    let branch_name = if let Some(b) = branch {
        b
    } else {
        // Use current branch (default to "main")
        "main".to_string()
    };

    // Step 3: Push to remote
    println!(
        "{}",
        format!("Pushing to {}/{}...", remote, branch_name).cyan()
    );

    let output = Command::new("git")
        .args(["push", &remote, &branch_name])
        .current_dir(&current_dir)
        .output()?;

    if output.status.success() {
        println!("{} Pushed to {}/{}", "✓".green(), remote, branch_name);

        // Show git output if any
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git push failed:\n{}", stderr);
    }

    Ok(())
}
