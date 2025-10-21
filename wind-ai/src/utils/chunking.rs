use anyhow::Result;

pub fn chunk_diff(diff: &str, max_tokens: usize) -> Result<Vec<String>> {
    let estimated_tokens = estimate_tokens(diff);

    if estimated_tokens <= max_tokens {
        return Ok(vec![diff.to_string()]);
    }

    let lines: Vec<&str> = diff.lines().collect();
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_tokens = 0;

    let tokens_per_line = (estimated_tokens as f64 / lines.len() as f64).ceil() as usize;

    for line in lines {
        let line_tokens = tokens_per_line;

        if current_tokens + line_tokens > max_tokens && !current_chunk.is_empty() {
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_tokens = 0;
        }

        current_chunk.push_str(line);
        current_chunk.push('\n');
        current_tokens += line_tokens;
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    Ok(chunks)
}

fn estimate_tokens(text: &str) -> usize {
    (text.len() as f64 / 4.0).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_diff() {
        let diff = "line1\nline2\nline3";
        let chunks = chunk_diff(diff, 1000).unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_chunk_large_diff() {
        // Create a diff with many lines (each line is ~50 chars)
        let diff = (0..1000)
            .map(|i| format!("line {}: some diff content here", i))
            .collect::<Vec<_>>()
            .join("\n");
        let chunks = chunk_diff(&diff, 1000).unwrap();
        // ~50k chars / ~12.5k tokens, with 1000 token limit should produce ~13 chunks
        assert!(
            chunks.len() >= 10,
            "Expected at least 10 chunks, got {}",
            chunks.len()
        );
        // Verify no chunk exceeds token limit
        for chunk in &chunks {
            let tokens = (chunk.len() as f64 / 4.0).ceil() as usize;
            assert!(tokens <= 1100, "Chunk has {} tokens, exceeds limit", tokens);
            // Allow 10% overflow
        }
    }
}
