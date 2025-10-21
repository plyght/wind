# Conflict Resolution Implementation

## Summary

Implemented interactive conflict resolution with AI assistance across wind-core, wind-cli, wind-ai, and wind-tui.

## Components Implemented

### 1. wind-core Conflict API

**New File:** [`wind-core/src/conflict.rs`](wind-core/src/conflict.rs)

- `ConflictResolver` - Main conflict resolution engine
- `detect_conflicts()` - Detects all conflicted files in the repository
- `get_conflict_content(path)` - Returns (base, ours, theirs) for a conflicted file
- `apply_resolution(path, content)` - Writes resolved content to file
- `mark_resolved(path)` - Stages the resolved file to remove conflict markers

**Updated:** [`wind-core/src/repository.rs`](wind-core/src/repository.rs)
- Added wrapper methods for conflict resolution operations

**Updated:** [`wind-core/src/lib.rs`](wind-core/src/lib.rs)
- Exported `ConflictResolver`, `ConflictFile`, `ConflictContent`

### 2. CLI Command: `wind resolve [file]`

**New File:** [`wind-cli/src/commands/resolve.rs`](wind-cli/src/commands/resolve.rs)

Interactive conflict resolution command with options:
- `(o)` Use ours - Apply our version
- `(t)` Use theirs - Apply their version
- `(a)` AI suggestion - Get AI-generated resolution
- `(e)` Edit manually - Open file in editor
- `(c)` Cancel

**Features:**
- Lists all conflicts when no file specified
- Shows 3-way diff (base | ours | theirs)
- AI integration with confirmation prompt
- Extracts code from AI responses (handles markdown code blocks)
- Auto-stages resolved files

**Updated:** [`wind-cli/src/main.rs`](wind-cli/src/main.rs)
- Added `Resolve` command enum variant
- Wired up command dispatch

**Updated:** [`wind-cli/src/commands/mod.rs`](wind-cli/src/commands/mod.rs)
- Exported resolve module

### 3. TUI ConflictView

**Updated:** [`wind-tui/src/state.rs`](wind-tui/src/state.rs)

Added conflict resolution support:
- New `Pane::Conflicts` variant
- `ConflictViewMode` enum (List, ThreeWay, AiSuggestion)
- State fields: `conflicts`, `conflict_view_mode`, `ai_suggestion`
- Methods:
  - `load_conflicts()` - Refresh conflict list
  - `show_conflicts()` - Switch to conflict pane
  - `resolve_conflict_with_ours()` - Apply ours
  - `resolve_conflict_with_theirs()` - Apply theirs
  - `show_three_way_diff()` - Show 3-way view
  - `request_ai_suggestion()` - Get AI resolution
  - `apply_ai_suggestion()` - Apply AI resolution

**Note:** TUI has compilation errors unrelated to conflict resolution (crossterm event-stream feature). The conflict resolution logic is implemented but needs the TUI base to be fixed first.

### 4. AI Integration

The existing `wind-ai::propose_conflict_resolution()` function is now fully wired:
- CLI calls it for `(a)` AI suggestion option
- TUI calls it via `request_ai_suggestion()`
- Responses are parsed to extract code from explanatory text
- Requires `OPENAI_API_KEY` environment variable (see `wind ai configure`)

## Usage Examples

### CLI Usage

```bash
wind resolve
wind resolve src/main.rs
```

Listing conflicts:
```
2 conflicted files:
  src/main.rs
  README.md

Run wind resolve <file> to resolve a specific file
```

Resolving a specific file:
```
Conflict in: src/main.rs

=== BASE VERSION ===
[base content]

=== OUR VERSION ===
[our content]

=== THEIR VERSION ===
[their content]

Choose resolution:
  (o) Use ours
  (t) Use theirs
  (a) AI suggestion
  (e) Edit manually
  (c) Cancel

Your choice: a

Generating AI suggestion...

=== AI SUGGESTED RESOLUTION ===
[AI generated code with explanation]

Apply this resolution? [y/N]: y
✓ Applied AI resolution and marked as resolved
```

### TUI Usage (once fixed)

1. Press key to show conflicts pane
2. Navigate with j/k or arrow keys
3. Press `o` to use ours
4. Press `t` to use theirs
5. Press `a` to request AI suggestion
6. Press `Enter` to apply AI suggestion
7. Press `e` to edit manually

## Testing

To test conflict resolution:

```bash
git init test-conflicts
cd test-conflicts

echo "line 1" > file.txt
git add file.txt
git commit -m "initial"

git checkout -b branch-a
echo "line 1 - version A" > file.txt
git commit -am "change A"

git checkout main  
git checkout -b branch-b
echo "line 1 - version B" > file.txt
git commit -am "change B"

git merge branch-a
```

Now test:

```bash
wind resolve
wind resolve file.txt
```

With AI (requires API key):

```bash
export OPENAI_API_KEY=sk-...
wind ai configure --api-key $OPENAI_API_KEY
wind resolve file.txt
```

## Implementation Notes

1. **Git2 Integration**: Uses git2-rs `index.conflicts()` API to detect merge conflicts
2. **3-Way Merge**: Extracts ancestor (base), ours, and theirs from index entries
3. **AI Code Extraction**: Handles both markdown code blocks and plain text responses
4. **Auto-staging**: Resolved files are automatically staged after resolution
5. **TUI Async**: AI requests in TUI happen asynchronously with progress notifications

## Known Issues

1. TUI has unrelated compilation errors (crossterm feature flag)
2. Manual edit mode just displays instructions (doesn't launch $EDITOR)
3. No undo for applied resolutions (user must use git reset)

## Next Steps

1. Fix TUI compilation errors
2. Add keybinding documentation to TUI
3. Implement $EDITOR integration for manual edit
4. Add preview diff before applying resolution
5. Support resolving multiple files at once
6. Add tests for conflict detection and resolution
