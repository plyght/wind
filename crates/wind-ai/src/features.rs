use crate::utils::{chunk_diff, sanitize_diff, templates};
use crate::{provider, AiOpts};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct CommitSummary {
    pub id: String,
    pub message: String,
}

pub async fn suggest_commit_message(diff: &str) -> Result<String> {
    let provider = provider::get_provider()?;

    let sanitized = sanitize_diff(diff)?;

    let chunks = chunk_diff(&sanitized, 4000)?;
    let diff_text = if chunks.len() > 1 {
        format!(
            "{}...\n(diff truncated, showing first {} tokens)",
            chunks[0], 4000
        )
    } else {
        chunks[0].clone()
    };

    let prompt = templates::commit_message_prompt(&diff_text);

    let opts = AiOpts {
        max_tokens: Some(200),
        temperature: Some(0.7),
        stream: false,
    };

    let message = provider.complete(&prompt, opts).await?;

    Ok(message.trim().to_string())
}

pub async fn suggest_pr_description(commits: &[CommitSummary]) -> Result<String> {
    let provider = provider::get_provider()?;

    let mut summary = String::new();
    for commit in commits {
        summary.push_str(&format!("Commit: {}\n{}\n\n", commit.id, commit.message));
    }

    let sanitized = sanitize_diff(&summary)?;
    let chunks = chunk_diff(&sanitized, 6000)?;

    let diff_summary = if chunks.len() > 1 {
        format!("{}...\n(diff truncated)", chunks[0])
    } else {
        chunks[0].clone()
    };

    let prompt = templates::pr_description_prompt(&summary, &diff_summary);

    let opts = AiOpts {
        max_tokens: Some(800),
        temperature: Some(0.7),
        stream: false,
    };

    let description = provider.complete(&prompt, opts).await?;

    Ok(description.trim().to_string())
}

pub async fn propose_conflict_resolution(base: &str, ours: &str, theirs: &str) -> Result<String> {
    let provider = provider::get_provider()?;

    let prompt = templates::conflict_resolution_prompt(base, ours, theirs);

    let opts = AiOpts {
        max_tokens: Some(1000),
        temperature: Some(0.5),
        stream: false,
    };

    let resolution = provider.complete(&prompt, opts).await?;

    Ok(resolution.trim().to_string())
}
