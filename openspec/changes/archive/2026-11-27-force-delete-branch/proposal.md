## Why

`Delete` runs `git branch -d`. Git refuses a branch that is not fully merged, and the user must open a terminal to run `git branch -D`. This is follow-up 8.28.

## What Changes

- When `git branch -d` fails, the confirmation changes to "Force delete `<name>`?" with `Force delete` and `Cancel`. The git error text stays visible above it.
- `Force delete` runs `git branch -D`. A second failure shows the git error text and ends the question.
- The app does not read the error text of Git. Any failure of `-d` offers the force step, and Git still decides whether `-D` works (for example, it refuses the current branch and a branch that another work tree has checked out).
- Not in this change: delete of a remote branch, rename.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `branches`: "Delete branch" gets a force step.

## Impact

- `crates/gitcore/src/branch.rs`: `delete` takes a `DeleteMode`.
- `crates/app/src/branches.rs`: `DeletePrompt`, the force step, the second confirmation text.
- `README.md`: the Branches note. `openspec/follow-ups.md`: mark 8.28 done.
