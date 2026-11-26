## Context

`gitcore::branch` has `checkout_local` and `checkout_remote`. Both call `RepoLocation::run_checked` and map a refusal to `CommandError::Failed(text)`. In the app, checkout runs on the background executor and stores its task in `Working.action_task`. `Working::busy()` is `action_task.is_some()`. `BranchPanel` holds the entries, the selection, and one `error` line. `finish_checkout` reloads history on success and sets `branches.error` on failure.

`gpui-component` has a one-line `Input` with `InputState`. The commit box already uses `TextareaState` with a subscription for changes.

## Goals / Non-Goals

**Goals:**
- Create a branch and delete a local branch from the branch panel, for Windows and WSL repositories.
- A delete cannot lose commits by accident.

**Non-Goals:**
- Create from another start point, rename, force delete, delete a remote branch, a key shortcut.

## Decisions

### D1. Two functions in `gitcore`, Git decides what is valid
`gitcore::branch::create(&RepoLocation, name)` and `delete(&RepoLocation, name)`. Both return `Result<(), CommandError>`. `create` runs `git check-ref-format --branch <name>` first and then `git switch -c <name>`. The check rejects a name that starts with `-`, so a name cannot pass as an option (principle: boundary-discipline). Every other rule (spaces, `..`, a name that exists) is Git's, and its message goes to the user. The app has no copy of those rules (principle: model-the-domain).
`delete` runs `git branch -d -- <name>`.
Alternative: validate the name in the UI. Rejected: it copies the rules of Git, and they change between versions.

### D2. Create also checks out
`git switch -c` makes the branch and moves HEAD to it. Fork and GitHub Desktop do the same, and a user who names a branch nearly always wants to work on it. Work-tree changes stay (Git carries them over). A new branch always starts at the current commit.
Alternative: `git branch <name>` without switching. Rejected: it needs a second click for the common case.

### D3. Delete is `-d`, with a confirmation step
`-d` keeps Git's own safety: a branch with commits that no other branch holds is refused. The app does not offer `-D`. Force delete is a follow-up; until then the terminal does it.
The confirmation is state in `BranchPanel`: `confirming_delete: Option<String>`. A press on `Delete` sets it; the panel then shows "Delete `<name>`?" with `Delete` and `Cancel`. A change of selection, a refs reload, or a `Cancel` clears it. One field, no dialog widget (principle: laziness-protocol).
Alternative: a modal dialog. Rejected: the app has no dialog code, and the inline step is as safe for a `-d` delete.

### D4. Only a local, non-current branch can be deleted
`BranchPanel::can_delete()` is true when the selection is a local branch and not HEAD. It is a pure method with a unit test, next to `can_check_out`. Git refuses the current branch too, but the button is disabled first, so the user does not see an error for a click the app could stop.

### D5. Busy state and finish step are shared
Create and delete store their task in `Working.action_task`, like checkout. `finish_checkout` becomes `finish_branch_command`, because the three commands end the same way: success → `reload_history` and `refresh_working`; failure → `branches.error`. No new mechanism.

### D6. The name field
The panel gets one `Input` and a `Create` button under the list. The button is disabled while the trimmed name is empty or the app is busy. After a successful create the field is cleared. The field does not keep keyboard focus rules of its own beyond what `Input` does.

## Risks / Trade-offs

- A user who wants to delete an unmerged branch must use a terminal → the message of Git names the command; force delete is a follow-up.
- `switch -c` on a repository with no commit renames the unborn branch instead of making a new one → this is Git's behavior and the app shows no error; a repository with no commit has no branch list, so the panel is empty anyway.
- A branch that a worktree has checked out is refused by Git → its message shows.
