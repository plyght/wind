# wind-collab

Collaboration provider for Wind VCS, enabling multi-platform PR management with first-class support for stacked PRs.

## Features

- **CollabProvider trait**: Platform-agnostic interface for PR operations
- **GitHub integration**: Dual-mode authentication (gh CLI preferred, API fallback)
- **Stacked PRs**: Native support for parent-child PR relationships with automatic linking
- **Async operations**: Built on tokio for efficient concurrent operations

## Architecture

### Core Trait

```rust
#[async_trait]
pub trait CollabProvider {
    async fn create_pr(&self, req: CreatePrRequest) -> Result<PrRef>;
    async fn update_pr(&self, pr: &PrRef, update: PrUpdate) -> Result<()>;
    async fn list_prs(&self) -> Result<Vec<PrInfo>>;
    async fn get_pr_status(&self, pr: &PrRef) -> Result<PrStatus>;
}
```

### GitHub Integration

**Primary: gh CLI**
- Checks for `gh` installation via `which`
- Uses subprocess execution for all operations
- No token management required (uses gh's auth)

**Fallback: REST API**
- Requires `GH_TOKEN` environment variable
- Direct reqwest-based HTTP calls
- Full API v3 compatibility

### Stack Linking Design

Stacked PRs are linked through:
1. **Metadata serialization**: JSON embedded in PR body as HTML comment
2. **Human-readable links**: Parent PR URL displayed in description
3. **Bidirectional tracking**: Parent knows children, children know parent

Example stack metadata in PR body:
```markdown
<!-- WIND_STACK_METADATA
{"parent_pr":{"number":123,"url":"..."},"child_prs":[],"stack_position":2,"stack_size":3}
-->

**Stack:** Part 2/3 | Parent: #123
Parent PR: https://github.com/owner/repo/pull/123
```

## Usage

```rust
use wind_collab::{GitHubProvider, CollabProvider, CreatePrRequest, StackMetadata, PrRef};

let provider = GitHubProvider::new("owner".into(), "repo".into()).await?;

let req = CreatePrRequest {
    title: "Feature: Add caching".into(),
    body: "Implements caching layer".into(),
    head: "feature-branch".into(),
    base: "main".into(),
    draft: true,
    stack_metadata: Some(StackMetadata {
        parent_pr: Some(PrRef { number: 42, url: "...".into() }),
        child_prs: vec![],
        stack_position: 2,
        stack_size: 3,
    }),
};

let pr = provider.create_pr(req).await?;
```

## Error Handling

Clear error messages for:
- Missing authentication (no gh CLI and no GH_TOKEN)
- Network failures with context
- API rate limiting
- Invalid repository access

## Dependencies

- `tokio`: Async runtime and process execution
- `reqwest`: HTTP client for REST API
- `serde`/`serde_json`: Serialization for API and metadata
- `anyhow`: Error handling with context
- `which`: CLI detection
- `async-trait`: Trait async support
