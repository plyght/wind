use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use wind_core::Repository;

pub async fn execute(file: Option<String>) -> Result<()> {
    let repo = Repository::open(".")?;
    let conflicts = repo.detect_conflicts()?;

    if conflicts.is_empty() {
        println!("{}", "No conflicts to resolve".green());
        return Ok(());
    }

    if let Some(path) = file {
        resolve_file(&repo, &path).await?;
    } else {
        println!("{} conflicted files:", conflicts.len());
        for conflict in conflicts {
            println!("  {}", conflict.path.red());
        }
        println!(
            "\nRun {} to resolve a specific file",
            "wind resolve <file>".cyan()
        );
    }

    Ok(())
}

async fn resolve_file(repo: &Repository, path: &str) -> Result<()> {
    let content = repo
        .get_conflict_content(path)
        .context(format!("Failed to get conflict content for {}", path))?;

    println!("\n{} {}\n", "Conflict in:".bold(), path.yellow());

    if let Some(base) = &content.base {
        println!("{}", "=== BASE VERSION ===".cyan());
        println!("{}", base);
    }

    println!("{}", "=== OUR VERSION ===".green());
    println!("{}", content.ours);

    println!("{}", "=== THEIR VERSION ===".red());
    println!("{}", content.theirs);
    println!();

    loop {
        println!("Choose resolution:");
        println!("  {} Use ours", "(o)".green());
        println!("  {} Use theirs", "(t)".red());
        println!("  {} AI suggestion", "(a)".cyan());
        println!("  {} Edit manually", "(e)".yellow());
        println!("  {} Cancel", "(c)".white());

        print!("\nYour choice: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim().to_lowercase();

        match choice.as_str() {
            "o" | "ours" => {
                repo.apply_resolution(path, &content.ours)?;
                repo.mark_resolved(path)?;
                println!("{} Applied our version and marked as resolved", "✓".green());
                break;
            }
            "t" | "theirs" => {
                repo.apply_resolution(path, &content.theirs)?;
                repo.mark_resolved(path)?;
                println!(
                    "{} Applied their version and marked as resolved",
                    "✓".green()
                );
                break;
            }
            "a" | "ai" => {
                resolve_with_ai(repo, path, &content).await?;
                break;
            }
            "e" | "edit" => {
                println!("\n{}", "Opening editor...".yellow());
                println!("Please manually edit: {}", path);
                println!("After editing, run: wind resolve --mark {}", path);
                break;
            }
            "c" | "cancel" => {
                println!("{}", "Cancelled".yellow());
                break;
            }
            _ => {
                println!("{} Invalid choice, please try again\n", "✗".red());
            }
        }
    }

    Ok(())
}

async fn resolve_with_ai(
    repo: &Repository,
    path: &str,
    content: &wind_core::ConflictContent,
) -> Result<()> {
    println!("{}", "Generating AI suggestion...".cyan());

    let base = content.base.as_deref().unwrap_or("");
    let resolution = wind_ai::propose_conflict_resolution(base, &content.ours, &content.theirs)
        .await
        .context("AI resolution failed. Make sure AI is configured (wind ai configure)")?;

    println!("\n{}", "=== AI SUGGESTED RESOLUTION ===".cyan().bold());
    println!("{}", resolution);
    println!();

    print!("Apply this resolution? [y/N]: ");
    io::stdout().flush()?;

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() == "y" {
        let code_part = extract_code_from_ai_response(&resolution);
        repo.apply_resolution(path, &code_part)?;
        repo.mark_resolved(path)?;
        println!(
            "{} Applied AI resolution and marked as resolved",
            "✓".green()
        );
    } else {
        println!("{}", "AI resolution not applied".yellow());
    }

    Ok(())
}

fn extract_code_from_ai_response(response: &str) -> String {
    if let Some(start) = response.find("```") {
        if let Some(end) = response[start + 3..].find("```") {
            let code_block = &response[start + 3..start + 3 + end];
            if let Some(newline) = code_block.find('\n') {
                return code_block[newline + 1..].trim().to_string();
            }
        }
    }

    response
        .lines()
        .skip_while(|l| !l.starts_with(|c: char| !c.is_whitespace()))
        .take_while(|l| !l.contains("explanation") && !l.contains("Explanation"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
