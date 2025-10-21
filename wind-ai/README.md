# wind-ai

AI-powered features for Wind VCS. Provider-agnostic, minimal, and secure.

## Features

- **Provider-agnostic**: Supports OpenAI and Anthropic via trait abstraction
- **VCS-focused**: Commit messages, PR descriptions, conflict resolution
- **Security-first**: Sanitizes secrets, redacts .env files, no telemetry
- **Transparent costs**: Shows token usage and cost estimates

## Setup

Set an API key environment variable:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Usage

```rust
use wind_ai::{suggest_commit_message, suggest_pr_description, propose_conflict_resolution};

let diff = "...";
let message = suggest_commit_message(diff).await?;

let commits = vec![...];
let pr_desc = suggest_pr_description(commits).await?;

let resolution = propose_conflict_resolution(base, ours, theirs).await?;
```

## Provider Implementation

```rust
use wind_ai::provider::{AiProvider, OpenAiProvider, AnthropicProvider};

let provider = OpenAiProvider::new(api_key);
let response = provider.complete(prompt, opts).await?;
```

## Security

- Auto-detects and redacts API keys, tokens, passwords
- Filters .env file contents from diffs
- No data sent to third parties except chosen AI provider
- Token usage displayed for cost transparency
