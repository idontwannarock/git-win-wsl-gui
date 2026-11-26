## Context

`gitcore::index` has `stage`, `unstage`, `stage_all`, and `commit`. Path commands use `--literal-pathspecs` and a `--` separator. `commit` runs `git commit -m` and turns a failure into `CommandError::Failed(stdout + stderr)`, so a hook message reaches the user. In the app, `Working` holds the two file lists, the selected file, the commit message box (`TextareaState`), and `action_task`. `run_index_action` runs a closure on the background executor, and `finish_action` refreshes the status. `Status.branch.oid` is `None` on a branch with no commit.

`status` runs with `--untracked-files=all`, so an untracked entry is always a file.

## Goals / Non-Goals

**Goals:**
- Discard the changes of one unstaged file, with a confirmation.
- Amend the last commit with the staged files and a message.
- Never lose data without a confirmation. Never offer an action that Git must refuse.

**Non-Goals:**
- Discard all, discard of a hunk, undo of a discard, amend of an older commit, a warning for a pushed commit.

## Decisions

### D1. The rule "what can be discarded" lives in `gitcore`
`WorkingFile::can_discard()` is true for an untracked file, and for an unstaged file in state modified, deleted, or type-changed. It is false for a staged file, a conflicted file, and an unstaged file with state added, renamed, or copied. `discard(&RepoLocation, &WorkingFile)` returns `CommandError::Failed` when `can_discard` is false, and otherwise runs the command. The UI asks `can_discard` for the button state and never builds a command (principle: boundary-discipline, model-the-domain).
Why the exclusions: `git restore` of an intent-to-add file (state added) removes its content from the work tree, and a conflicted file needs `--ours` or `--theirs`, which the app does not offer.

### D2. Two commands, chosen by state
Modified, deleted, or type-changed → `git --literal-pathspecs restore --worktree -- <path>`. It copies the index version back, so staged changes stay and a deleted file comes back. Untracked → `git --literal-pathspecs clean -f -- <path>`. `-f` is required by `clean.requireForce` and is safe here, because the path is one file the user selected and confirmed.
Alternative: `git checkout -- <path>`. Rejected: `restore` is the current command and cannot switch branches by mistake.

### D3. Confirmation is state, as for branch delete
`Working.confirming_discard: Option<String>` holds the path being confirmed. A press on `Discard` sets it when `can_discard` holds. The panel then shows "Discard changes to `<file name>`?" with `Discard` and `Cancel`. A change of selection, and a status refresh in which the file is gone or no longer discardable, clear it. Same pattern as `confirming_delete` in the branch panel (principle: minimize-reader-load, one pattern for destructive actions).
Alternative: a modal dialog. Rejected: the app has none.

### D4. Amend is a toggle, not a second button
`Working.amending: bool`. When it turns on and the message box is empty, the app reads the last message with `git log -1 --format=%B` on the background executor and puts it in the box. If the box has text, the text stays, because it is what the user typed. When it turns off, the text stays. The commit button reads `Amend` while the toggle is on.
`can_commit(message, amending)`: not busy, message not empty, and either staged files exist (commit) or the branch has a commit (amend: `Working.branch.oid` is `Some`). An amend with no staged file changes the message only.
Amend turns off when a status read shows another head commit than the one before (`keeps_amending`). Without this, a checkout or a pull after the toggle would amend a commit that the user never saw. Found in review. The message in the box stays, because it is visible and the user may have edited it.
Alternative: a separate `Amend` button next to `Commit`. Rejected: the user then retypes the old message, because nothing fills it.

### D5. `amend` and `commit` share the error path
`commit(location, message)` and `amend(location, message)` call one private function with the argument list (`["commit", "-m", message]` or `["commit", "--amend", "-m", message]`). Hooks and signing run as for a commit.
`last_message(location)` returns the body of the last commit without the trailing newline.

### D6. Busy state and refresh are shared
Discard goes through `run_index_action`, so `busy()` blocks the other actions and `finish_action` refreshes the status. Amend goes through the commit path (`commit_staged`), which reloads history, because the last commit changed.

## Risks / Trade-offs

- A discard cannot be undone → the confirmation names the file, and only one file at a time can go.
- A user who amends a pushed commit gets a rejected push later, and the app never forces a push → README says so. A warning is a follow-up.
- An amend of a merge commit or of a commit with a signature: Git decides, and its message shows.
- A status refresh during the confirmation can change the list → D3 clears the confirmation when the file is gone.
