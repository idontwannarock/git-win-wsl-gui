## Why

The force delete (change `force-delete-branch`) is a second prompt that shows only after Git refuses. The user asked for one confirmation with a check box "Force", so the choice is visible before the first press. A later "delete remote branch" can then sit in the same confirmation.

## What Changes

- The delete confirmation ("Delete `<name>`?") gets a check box "Force (delete even if not merged)". It is off at the start.
- Off: `Delete` runs `git branch -d`. On: `Delete` runs `git branch -D`.
- When `git branch -d` fails, the app shows the git error and keeps the confirmation open, with the check box off, so the user can tick it and press `Delete` again.
- The second prompt "Force delete `<name>`?" and its `Force delete` button go away.
- Not in this change: delete of a remote branch (follow-up 8.30). Git has no force mode for `git push origin --delete`, so that change adds its own check box "Also delete the remote branch".

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `branches`: "Delete branch" scenarios for the force step.

## Impact

- `crates/app/src/branches.rs`: `DeletePrompt` toggle, view with a `Checkbox`, `finish_delete`.
- `crates/gitcore`: no change (`DeleteMode` stays).
- `README.md` Branches note. `openspec/follow-ups.md`: note the check box in 8.30.
