## Context

`gitcore::branch::delete(location, name)` runs `git branch -d -- <name>`. `BranchPanel.confirming_delete: Option<String>` holds the branch being confirmed. `confirm_delete` runs the command on the background executor and `finish_branch_command` shows an error or reloads.

## Goals / Non-Goals

**Goals:**
- Let the user force-delete a branch that Git calls not fully merged, with a second explicit confirmation.

**Non-Goals:**
- Delete of a remote branch, rename, undo.

## Decisions

### D1. A mode, not a second function
`delete(location, name, DeleteMode)` with `DeleteMode::{IfMerged, Force}`. The mode picks `-d` or `-D`. The `--` and the name stay the same (principle: model-the-domain, one rule in one place). A `bool` was rejected: a call `delete(.., true)` does not say what it does.

### D2. The prompt is state: name and mode
`DeletePrompt { name, mode }` replaces the bare name. `ask_delete` creates `IfMerged`. After a failed `IfMerged` delete, the panel moves the same prompt to `Force`, only when the failed branch is still the selected one. A selection change, new refs, and `Cancel` clear it, as before (principle: type-system-discipline, there is no "force" flag outside a prompt).

### D3. Do not parse the error text
The error text of Git depends on the language of the user. The app offers `Force delete` after any failure of `-d`. If `-D` fails too, the git error shows and the question ends. This costs one extra click in rare cases and removes a fragile text match (principle: boundary-discipline).
Alternative: ask `git branch --merged` before the first delete and show a warning in the first prompt. Rejected: a second Git call and a second source of truth for "merged", because `-d` also checks the upstream.

### D4. Test seams
`gitcore::branch::delete` with both modes (real repositories). `BranchPanel` prompt transitions (pure). The button flow is checked by hand in the app.

## Risks / Trade-offs

- A force delete drops commits that no other ref holds. They stay reachable through the reflog for a while. The prompt names the branch and says it is not merged. Undo is not offered.
