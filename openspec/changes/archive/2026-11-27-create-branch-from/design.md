## Context

`gitcore::branch::create(location, name)` runs `git switch -c <name>`. The branch panel keeps a name field and a `Create` button. Commit rows (`repo_view.rs`, `rows_in`) and branch rows (`branches.rs`, `branch_rows`) are built inside `uniform_list`. `gpui-component` 0.10.0 offers `ContextMenuExt::context_menu` and `PopupMenuItem::on_click`.

## Goals / Non-Goals

**Goals:**
- Create a branch at a commit or at a branch that the user right-clicked.

**Non-Goals:**
- A start point typed as text, a tag as start point, create without checkout, other right-click items.

## Decisions

### D1. The start point is a type, not a string
```
enum StartPoint { Commit(Oid), Local(String), Remote(String) }
```
Its revision text is the full object id, `refs/heads/<name>`, or `refs/remotes/<name>`. A full ref name cannot begin with `-`, and an `Oid` is hex. So Git never reads the start point as an option. `git switch` has no `--` before the start point, so this is the guard (principle: type-system-discipline, boundary-discipline). A plain `&str` was rejected: a branch named `-Cmain` can exist.
`create(location, name, Option<&StartPoint>)` adds the revision text after the name.

### D2. The start point is panel state, set by the menu
`BranchPanel.start: Option<Start>` with `Start { point: StartPoint, label: String }`. The label is the short id (7 characters) or the branch name. The menu item calls `RepoView::start_branch_from(point, label, window, cx)`. It stores the start point and focuses the name field. `Create` reads the start point. A successful create clears it. A failed create keeps it. A clear button clears it.
Why state and not a dialog: the app has no dialog, and the name field already exists (principle: minimize-reader-load).
Alternative: a right-click on a commit that asks for a name in a pop-up. Rejected: new input surface for one action.

### D3. `show_refs` keeps the start point
A history reload must not drop the start point that the user chose. If the branch is gone after the reload, `Create` fails with the git error, and the user clears the start.

### D4. One menu item, one label
Both menus have the single item `Create branch here`. The menu handler needs the view: it clones the `Entity<RepoView>` (`cx.entity()`) and calls `update` (the same pattern as `cx.listener`).

### D5. Test seams
`gitcore::branch::create` with each `StartPoint` kind, in real repositories: a commit, a local branch, a remote branch (tracking result observed and asserted), an option-like branch name, a missing branch (git error, no branch). `BranchPanel` start transitions are pure: set, clear, keep on `show_refs`. The menu and the "Start:" line are checked by hand in the app.

## Risks / Trade-offs

- `switch -c` checks out the new branch. With a start point far from HEAD, Git can refuse when the work tree has conflicting changes. Git's message shows and nothing changes.
- The user can forget a set start point. The "Start:" line stays visible above the name field until it is cleared or used.
