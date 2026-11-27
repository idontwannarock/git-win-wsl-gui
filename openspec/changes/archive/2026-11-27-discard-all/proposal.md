## Why

The user can discard the changes of one file only. To drop all work-tree changes, the user must select and discard every file. Follow-up 8.31 asks for "discard all". The user chose to split 8.31: this change is "discard all"; discard of one hunk or one line is a later change.

## What Changes

- The header of the Unstaged group gets a right-click menu with the item `Discard all changes`.
- The item is disabled when the app is busy or when no tracked file can be discarded.
- The item opens a confirmation "Discard changes to N files?" with `Discard` and `Cancel`.
- `Discard` restores every tracked unstaged file whose content changed (modified, deleted, type changed) to the content of the index.
- Untracked files stay on disk. Staged changes stay. Conflicted files and files with an added, renamed, or copied state stay.
- Not in this change: discard of one hunk or one line (follow-up 8.31, second part), and the removal of untracked files in bulk.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `working-tree`: new requirement "Discard all changes".

## Impact

- `crates/gitcore/src/index.rs`: new `discard_all`.
- `crates/app/src/working.rs`: `DiscardPrompt` holds a target (one file, or all), a context menu on the Unstaged header, and the confirmation text.
- `README.md` Working tree note. `openspec/follow-ups.md`: 8.31 split.
