## Why

The app can stage, unstage, and commit, but it cannot throw away the changes of one file, and it cannot fix the last commit. A user must open a terminal for `git restore`, `git clean`, and `git commit --amend`. This is the second half of the "modify" gap (create and delete branch shipped in the last change).

## What Changes

- Add a `Discard` button to the Unstaged group. It works on the selected unstaged file. The first press asks "Discard changes to `<file>`?" with `Discard` and `Cancel`. The second press throws the changes away: `git restore --worktree` for a modified, deleted, or type-changed file, and `git clean -f` for an untracked file. Staged changes of the same file stay.
- `Discard` is disabled for a staged file, a conflicted file, and a file with an added, renamed, or copied state in the unstaged group. Discard of all files is not part of this change.
- Add an `Amend` toggle next to the `Commit` button. When it is on, the message box starts with the message of the last commit (only when the box is empty), the commit button says `Amend`, and it runs `git commit --amend -m <message>`. The staged files go into the last commit. An amend with nothing staged changes only the message.
- `Amend` is disabled while the branch has no commit. After a successful amend the toggle turns off and the message box is empty, as after a commit.
- Both actions use the busy state of the other index actions, and both show the message of Git when Git refuses.
- Not in this change: discard all, discard of a hunk or a line, undo of a discard, a warning that the last commit is already pushed (a later push is rejected by the remote; the app never forces a push), amend of an older commit.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `working-tree`: new requirements "Discard file changes" and "Amend the last commit".

## Impact

- `crates/gitcore/src/status.rs`: `WorkingFile::can_discard`.
- `crates/gitcore/src/index.rs`: `discard`, `amend`, `last_message`. `commit` and `amend` share one error path.
- `crates/app/src/working.rs`: `Working.confirming_discard`, `Working.amending`, the `Discard` button and its confirmation, the `Amend` toggle.
- `README.md`: a note for each action. `openspec/follow-ups.md`: add discard all, undo of discard, warn about a pushed amend.
- No new dependency.
