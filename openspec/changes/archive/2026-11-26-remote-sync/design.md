## Context

`gitcore` runs Git through `RepoLocation::command` (Windows `git.exe -C <path>`, or `wsl.exe -d <distro> --cd <path> --exec git`). `exec::run_command` hides the console window (`CREATE_NO_WINDOW`) and `Command::output()` gives the child a null stdin. So a terminal prompt of Git or SSH cannot be answered, and could wait for ever.

`branches.rs` already runs one long action: checkout runs on the background executor and stores its task in `Working.action_task`. `Working::busy()` is `action_task.is_some()`, and staging, commit, and checkout check it. `Status.branch` (head, upstream, ahead and behind) is parsed on every status read and then dropped.

## Goals / Non-Goals

**Goals:**
- Fetch, pull, and push from the UI, for Windows and WSL repositories.
- Never freeze the window. Never wait for a terminal.
- Show whether a pull or a push has work to do.

**Non-Goals:**
- Merge or rebase on pull, force push, push tags, choose a remote, cancel a running command, an in-app credential prompt.

## Decisions

### D1. Three functions in `gitcore`, the policy lives there
`gitcore::remote` has `fetch(&RepoLocation)`, `pull(&RepoLocation)`, `push(&RepoLocation)`, all returning `Result<(), CommandError>`. `push` checks the upstream itself (`rev-parse --abbrev-ref --symbolic-full-name @{u}`): present → `push`; absent → `push --set-upstream origin HEAD`. The UI passes no flag (principle: boundary-discipline, the UI cannot call push with a wrong mode). Tests use a local bare repository as the remote, so they need no network.
Alternative: the UI reads `Status.branch.upstream` and picks the command. Rejected: the status can be 2 seconds old, and the rule would sit in the UI.

### D2. `pull --ff-only`, no merge
Git refuses a pull that is not a fast-forward, and the working tree stays as it was. A merge or a rebase needs a conflict view, which the app does not have. A diverged branch shows the message of Git; the user then fixes it in a terminal until the app has merge.
Alternative: `git pull` with the user's `pull.rebase` or `pull.ff` setting. Rejected: it can start a merge or a rebase that stops half way, and the app cannot show or finish that state (principle: type-system-discipline, no state the UI cannot represent).

### D3. Fetch uses `--all --prune`
`--prune` removes remote-tracking branches that the remote deleted, so the branch list matches the remote. Fork and GitHub Desktop do the same. The risk is low: it deletes only remote-tracking refs, never a local branch.

### D4. No terminal prompt
Every remote command gets `GIT_TERMINAL_PROMPT=0`. For a WSL repository the variable must cross `wsl.exe`, so the command also appends `GIT_TERMINAL_PROMPT` to `WSLENV`. A credential helper with its own window (Git Credential Manager) still opens it, because it does not use the terminal. The helper dialog is the way a user signs in.
SSH asks for a passphrase on a terminal. With no console (`CREATE_NO_WINDOW`) and a null stdin, SSH fails at once. The user must load the key into an agent first. Check this by hand (task 4.2).
Alternative: a timeout that kills the command. Rejected for now: a slow, healthy fetch looks the same as a stuck one, and a kill leaves a lock file.

### D5. Busy state reuses `Working.action_task`
A remote command is stored in `action_task`, like checkout. `busy()` is then true, and staging, commit, and checkout already wait. A pull changes files, so a stage or a commit during a pull would race with it. One mechanism, no new field (principle: laziness-protocol).
A `SyncNote` on `GraphState` (`Running`, `Done`, or `Failed(text)`) says what runs ("Fetching...", "Pulling...", "Pushing...") and holds the last result or error. The label stays until the next remote command (follow-up 8.27).

### D6. `Working` keeps the branch
`Working` stores `branch: Option<Branch>` from each status read. A pure function `sync_labels(&Branch) -> (String, String)` builds `Pull ↓N` and `Push ↑N`. The counts change after a fetch, because Git compares with the local copy of the upstream. This closes follow-up 8.11.

### D7. After success, reload everything
Success runs `reload_history` and `refresh_working`, the same as a successful checkout. The reload also refreshes the branch list. Fetch and push change only refs, and pull changes refs and files, so one rule fits all three.

## Risks / Trade-offs

- A pull that stops at a diverged branch is a dead end inside the app → the message says what Git says; a terminal fixes it. Merge is a later change.
- A fetch from a slow or dead remote can run for minutes and cannot be cancelled → the busy label shows it runs. Cancel is a follow-up.
- `origin` is hard-coded for the first push of a branch → a repository without `origin` shows Git's error. Choosing a remote is a follow-up.
- Closing a tab while a command runs does not stop the Git process → same behavior as checkout today; the process ends by itself.
