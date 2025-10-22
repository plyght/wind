pub fn commit_message_prompt(diff: &str) -> String {
    format!(
        r#"You are a helpful assistant that writes concise, clear commit messages following conventional commit format.

Given the following git diff, write a commit message that:
1. Uses conventional commit format (type(scope): subject)
2. Has a clear, imperative subject line (max 72 chars)
3. Optionally includes a body explaining WHY if the change is complex
4. Common types: feat, fix, docs, style, refactor, test, chore

Diff:
```
{}
```

Respond with ONLY the commit message, no explanations or preamble."#,
        diff
    )
}

pub fn pr_description_prompt(commits_summary: &str, diff_summary: &str) -> String {
    format!(
        r#"You are a helpful assistant that writes clear, comprehensive pull request descriptions.

Given the following commits and diff summary, write a PR description that:
1. Starts with a clear summary of what changed and why
2. Lists key changes or features added
3. Notes any breaking changes or important considerations
4. Mentions related issues if evident from commits

Commits:
{}

Diff Summary:
```
{}
```

Respond with a well-structured PR description in Markdown format."#,
        commits_summary, diff_summary
    )
}

pub fn conflict_resolution_prompt(base: &str, ours: &str, theirs: &str) -> String {
    format!(
        r#"You are a helpful assistant that proposes git merge conflict resolutions.

Given the following conflict sections, propose a resolution that:
1. Preserves the intent of both changes when possible
2. Removes conflict markers
3. Ensures syntactic correctness
4. Explains your reasoning briefly

Base version:
```
{}
```

Our version:
```
{}
```

Their version:
```
{}
```

Respond with:
1. The proposed resolved code
2. A brief explanation of your resolution approach"#,
        base, ours, theirs
    )
}
