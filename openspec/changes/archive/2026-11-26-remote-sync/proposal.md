## Why

The app cannot talk to a remote. A user who wants to fetch, pull, or push must open a terminal. This is the largest gap that stops the app from replacing the terminal for daily work.

## What Changes

- Add three buttons next to `History` and `Changes`: `Fetch`, `Pull`, `Push`.
- `Fetch` runs `git fetch --all --prune`.
- `Pull` runs `git pull --ff-only`. When Git cannot fast-forward, the app shows the message of Git and changes nothing. A merge or a rebase is not part of this change.
- `Push` runs `git push`. When the current branch has no upstream, it runs `git push --set-upstream origin HEAD`.
- `Pull` and `Push` show how far the branch is behind and ahead of its upstream (`Pull ↓1`, `Push ↑2`). The numbers come from the status read that already runs every 2 seconds. They change after a fetch, not by themselves.
- While a remote command runs, the three buttons are disabled, a label shows what runs, and staging, commit, and checkout wait (the same rule as a running checkout).
- A remote command never waits for input at a terminal. A missing password fails with the message of Git. A credential helper with its own window (Git Credential Manager) still works.
- After a success, the app reloads history and refs and refreshes the working tree. After a failure, the app shows the message of Git until the next remote command.
- Fix follow-up 8.11: `Status.branch` is now read. Cover part of follow-up 8.2: remote commands show a busy state.
- Not in this change: force push, push of tags, merge or rebase on pull, choice of remote, cancel of a running command, a credential prompt inside the app.

## Capabilities

### New Capabilities
- `remote-sync`: fetch, pull, push, their busy state, and their errors.

### Modified Capabilities
- None. The existing requirements stay as written.

## Impact

- `crates/gitcore/src/remote.rs` (new): `fetch`, `pull`, `push`, and the no-prompt command setup.
- `crates/app/src/working.rs`: `Working` keeps the branch of the last status read.
- `crates/app/src/repo_view.rs`: the three buttons, the label, the error line.
- `README.md`: a Remote section. `openspec/follow-ups.md`: close 8.11, update 8.2, add "cancel a remote command" and "choose a remote".
- No new dependency.
