## Why

The app can list branches and check one out, but it cannot make or remove one. A user who wants a new branch or wants to clean up old branches must open a terminal. This is the first half of the "modify" gap (the second half is discard changes and amend, a later change).

## What Changes

- Add a name field and a `Create` button to the branch panel. `Create` makes a branch at the current commit and checks it out (`git switch -c`). Work-tree changes stay as they are.
- Add a `Delete` button to the branch panel. It works on the selected local branch, never the current one. The first press asks "Delete `<name>`?" with `Delete` and `Cancel`. The second press runs `git branch -d`.
- `git branch -d` refuses a branch that is not fully merged. The app shows the message of Git and keeps the branch. Force delete is not part of this change.
- Both commands use the busy state that checkout uses: while one runs, staging, commit, checkout, and remote commands wait.
- After a success, the app reloads history, refs, and the working tree. After a failure, the app shows the message of Git in the branch panel.
- Not in this change: create from another commit or branch, rename, force delete, delete of a remote branch, a key for delete.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `branches`: new requirements "Create branch" and "Delete branch".

## Impact

- `crates/gitcore/src/branch.rs`: `create` and `delete`.
- `crates/app/src/branches.rs`: the name field, the two buttons, the delete confirmation, and the shared finish step.
- `README.md`: a line in the v1 list and a Branches note. `openspec/follow-ups.md`: add "force delete", "create from a chosen commit or branch", "delete a remote branch".
- No new dependency.
