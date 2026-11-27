## Context

After `force-delete-branch`, `BranchPanel.confirming_delete` holds a `DeletePrompt { name, mode }`. `finish_delete` moves the prompt to `Force` after a failed `-d`. The view shows "Delete" or "Force delete" text from the mode.

## Goals / Non-Goals

**Goals:**
- One confirmation. The user picks the mode with a check box before the press.

**Non-Goals:**
- Delete of a remote branch.

## Decisions

### D1. The mode stays in the prompt, a check box flips it
`DeletePrompt.mode` is the state of the check box: `IfMerged` is off, `Force` is on. `toggle_force` flips it, and does nothing when no prompt shows. `ask_delete` starts with `IfMerged`. `gitcore::delete` does not change (principle: model-the-domain, the same `DeleteMode` rule from the earlier change).

### D2. A refused `-d` reopens the same question
After a failed `IfMerged` delete, `finish_delete` calls `ask_again(name)`. It opens a new prompt with `IfMerged` when that branch is still selected. The git error stays above it. The user ticks the box and presses `Delete`. A failed `Force` delete does not reopen the question.
This replaces `offer_force`. The guard "the branch is still selected" stays.
Alternative: close the question after a refusal. Rejected: the user then presses `Delete` twice again before they can tick the box.
Alternative: tick the box for the user after a refusal. Rejected: the user must decide to force, and a pre-ticked box is a default that destroys data.

### D3. The text of the question does not change with the mode
The label is always "Delete `<name>`?". The box carries the mode. The button label is always `Delete`.

### D4. Reload rule stays
`show_refs` clears the prompt only when the branch can no longer be deleted (from `force-delete-branch`, found in review). A reload must not reset the box.

### D5. Test seams
`BranchPanel` transitions are pure: ask, toggle, ask again, cancel, select, show refs. The view is checked by hand in the app.

## Risks / Trade-offs

- A user can tick `Force` before the first press and delete an unmerged branch without seeing the git warning. The box is off at the start, the label says "delete even if not merged", and the branch name is in the question.
