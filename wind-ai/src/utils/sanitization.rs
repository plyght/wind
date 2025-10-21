use anyhow::Result;
use regex::Regex;

lazy_static::lazy_static! {
    static ref SECRET_PATTERNS: Vec<Regex> = vec![
        Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[=:]\s*['"]?([a-zA-Z0-9_-]{20,})"#).unwrap(),
        Regex::new(r#"(?i)(secret|password|passwd|pwd)\s*[=:]\s*['"]?([^\s'"\n]{8,})"#).unwrap(),
        Regex::new(r#"(?i)(token)\s*[=:]\s*['"]?([a-zA-Z0-9_\-.]{20,})"#).unwrap(),
        Regex::new(r#"(?i)bearer\s+([a-zA-Z0-9_\-.]{20,})"#).unwrap(),
        Regex::new(r#"(?i)(aws|amazon)[_-]?(access|secret)[_-]?key['"]?\s*[=:]\s*['"]?([A-Z0-9]{16,})"#).unwrap(),
        Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        Regex::new(r"ghp_[a-zA-Z0-9]{36,}").unwrap(),
        Regex::new(r"gho_[a-zA-Z0-9]{36,}").unwrap(),
        Regex::new(r"github_pat_[a-zA-Z0-9]{22,}_[a-zA-Z0-9]{59,}").unwrap(),
    ];

    static ref ENV_FILE_PATTERN: Regex = Regex::new(r"(?m)^[+-]{3}\s+.*\.env").unwrap();
}

pub fn sanitize_diff(diff: &str) -> Result<String> {
    let mut sanitized = diff.to_string();

    if contains_env_file(&sanitized) {
        sanitized = remove_env_file_sections(&sanitized);
    }

    for pattern in SECRET_PATTERNS.iter() {
        sanitized = pattern.replace_all(&sanitized, "[REDACTED]").to_string();
    }

    Ok(sanitized)
}

fn contains_env_file(diff: &str) -> bool {
    ENV_FILE_PATTERN.is_match(diff)
}

fn remove_env_file_sections(diff: &str) -> String {
    let lines: Vec<&str> = diff.lines().collect();
    let mut result = Vec::new();
    let mut in_env_file = false;

    for line in lines {
        if line.starts_with("diff --git") {
            in_env_file = line.contains(".env");
        }

        if !in_env_file {
            result.push(line);
        } else if line.starts_with("diff --git") {
            result.push("[.env file content redacted for security]");
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_api_key() {
        let diff = "api_key = sk-1234567890abcdefghij";
        let sanitized = sanitize_diff(diff).unwrap();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("sk-1234567890"));
    }

    #[test]
    fn test_redact_github_token() {
        let diff = "token: ghp_1234567890abcdefghijklmnopqrstuvwxyz";
        let sanitized = sanitize_diff(diff).unwrap();
        assert!(sanitized.contains("[REDACTED]"));
        assert!(!sanitized.contains("ghp_"));
    }
}
