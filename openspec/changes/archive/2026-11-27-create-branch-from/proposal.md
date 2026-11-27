## Why

`Create` always starts the new branch at the current commit (follow-up 8.29). A user who wants a branch at an older commit, or at another branch, must check that out first or open a terminal.

The user asked for a right-click menu instead of one more button for each action.

## What Changes

- A right-click on a commit row in the history shows a menu with `Create branch here`. A right-click on a branch row in the branch panel shows the same item.
- The item sets the start point of the next `Create`. The branch panel then shows "Start: `<short commit id>` or `<branch name>`" with a clear button above the name field, and the cursor goes to the name field.
- `Create` makes the branch at that start point and checks it out (`git switch -c <name> <start>`). Without a start point it works as before.
- After a successful create the start point is cleared. After a failure it stays, so the user can fix the name.
- Not in this change: moving `Checkout`, `Delete`, and other buttons into right-click menus (new follow-up 8.35), and a start point typed as text.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `branches`: "Create branch" gets a start point and the right-click item.

## Impact

- `crates/gitcore/src/branch.rs`: `StartPoint` and `create(location, name, Option<&StartPoint>)`.
- `crates/app/src/branches.rs`: the start point in `BranchPanel`, the "Start:" line, the menu on branch rows. `crates/app/src/repo_view.rs`: the menu on commit rows.
- `README.md` Branches note. `openspec/follow-ups.md`: 8.29 done, new 8.35.
- No new dependency (`gpui-component` already has `ContextMenuExt`).
