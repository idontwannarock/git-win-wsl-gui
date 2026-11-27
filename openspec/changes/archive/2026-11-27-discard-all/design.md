## Context

`discard(location, file)` restores one file (`git restore --worktree`) or removes one untracked file (`git clean -f`). `Working.discard: DiscardPrompt` holds the path of the file that waits for confirmation. `WorkingFile::can_discard()` is true for an untracked file and for an unstaged modified, deleted, or type-changed file.

## Goals / Non-Goals

**Goals:**
- One confirmed action that restores all discardable tracked files.
- The action lives in a right-click menu on the Unstaged header, not in a new button (user preference, follow-up 8.35).

**Non-Goals:**
- Discard of one hunk or one line.
- Removal of untracked files in bulk. A removed new file cannot come back, so bulk removal is a separate decision.
- Moving the existing `Discard` button into a menu (8.35).

## Decisions

### D1. Which files "all" means
"All" is every file in the Unstaged group with `can_discard()` and a diff that is not `Untracked`. One function `discard_all(location, files)` in `gitcore::index` selects them and runs one `git --literal-pathspecs restore --worktree -- <paths>`. It does nothing when the list is empty. The app does not pass `.` as the path, because Git refuses an unmerged path in the same command and the whole restore would fail (principle: boundary-discipline, the selection happens in one place and is tested).
Alternative: `git restore --worktree .`. Rejected: a conflicted file makes the command fail.
Alternative: include untracked files (`git clean -f`). Rejected by the user: a removed new file is lost for good.

### D2. The prompt holds a target
`DiscardPrompt` holds `Option<DiscardTarget>`, with `DiscardTarget::File(path)` and `DiscardTarget::All`. One confirmation view, one `confirm_discard`. `ask_all` opens `All` only when at least one file would be discarded. `keep_if_discardable` keeps `All` while at least one such file is left, and keeps `File` as before (principle: type-system-discipline, two targets are two enum cases, not two flags).
Alternative: a second prompt type next to the first. Rejected: two confirmations, two cancel paths, two places that reload rules must reach.

### D3. The text shows the count
The question reads "Discard changes to N files?" for `All`, and "Discard changes to `<name>`?" for a file. The count is computed from the current Unstaged group when the view draws, so it follows a reload. If the count drops to 0 the prompt closes (D2).

### D4. The menu is on the header
The Unstaged header gets an id and `.context_menu(...)` with one item `Discard all changes`. The item is disabled when the app is busy or the count is 0. Press calls `ask_discard_all`. The menu only asks. The confirmation runs the action, as for one file.

### D5. A very long path list
The paths go on the command line. A work tree with thousands of changed files can pass the operating-system limit, and Git then fails with an error that the app shows. This change does not split the list: no user hit this, and a split would be code for a case we have not seen (principle: laziness-protocol). It is a known limit, written in the README.

### D6. Test seams
- `gitcore`: `discard_all` on a temporary repository. Modified, deleted, and type-changed files come back; an untracked file, a staged-only file, and a conflicted file stay; staged changes of a file with more unstaged edits stay; an empty list is a no-op.
- `app`: pure transitions of `DiscardPrompt` (ask file, ask all, cancel, keep rule, count 0).
- The menu and the view are checked by hand in the app.

## Risks / Trade-offs

- One press confirms many files. The question shows the count, and `Cancel` is next to `Discard`.
- The Unstaged header right-click is hard to find. The existing `Discard` button stays, and 8.35 plans the same pattern for other actions.
