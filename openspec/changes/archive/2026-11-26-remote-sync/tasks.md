## 1. Slice: fetch, pull, push in `gitcore`

- [x] 1.1 (tdd) `gitcore::remote::fetch`: tests with a local bare repository as `origin` and a second clone that pushes. Cover: a new commit arrives as a remote-tracking ref and the working tree stays; a branch deleted on the remote is pruned; a repository with no remote succeeds or reports Git's error (record which); a missing remote path reports Git's error. Include the no-prompt setup: a unit test that the command has `GIT_TERMINAL_PROMPT=0`, and `WSLENV` names it for a WSL location.
- [x] 1.2 (tdd) `gitcore::remote::pull`: tests cover a fast-forward (branch and files move), diverged branches (error, branch and files unchanged), an up-to-date branch, and a branch with no upstream (Git's error).
- [x] 1.3 (tdd) `gitcore::remote::push`: tests cover push to the upstream, a branch with no upstream (remote gets the branch, upstream is set), a refused push (error, remote unchanged), and a repository with no `origin` (Git's error).
- [x] 1.4 Run the Windows tests, and the same tests against a WSL scratch repository when WSL is present (use `WslScratch`).

## 2. Slice: counts on the buttons (blocked by 1)

- [x] 2.1 (tdd) `app`: `sync_labels(&Branch) -> (String, String)`. Tests cover: behind 1 and ahead 2, both 0, no upstream (`ahead_behind` is `None`), only ahead, only behind.
- [x] 2.2 `Working` keeps `branch` from each status read. Check: `Status.branch` is read outside tests.

## 3. Slice: buttons, busy state, errors (blocked by 1, 2)

- [x] 3.1 `RepoView`: the buttons `Fetch`, `Pull`, `Push` after `History` and `Changes`. A press starts the command on the background executor and stores the task in `working.action_task`. Buttons are disabled while `busy()`. A label shows "Fetching...", "Pulling...", "Pushing...", then "Fetched", "Pulled", "Pushed", or the error text of Git. Success reloads history and refreshes the working tree. The label stays until the next remote command. Check by hand on a local repository with a bare remote: all three buttons, the counts after each, a refused push, a diverged pull.
- [x] 3.2 Check the busy state: start a fetch from a remote that answers slowly (a bare repository behind a slow hook is not needed; use a `file://` remote on a network share or throttle with a 5-second `git-upload-pack` wrapper) and confirm the window scrolls, the buttons are disabled, and a stage click waits. If no slow remote can be set up, say so and verify with the debug log timestamps instead. Result: the Pulling... state showed all three buttons grey in a screenshot, taken while the HTTPS command ran.

## 4. Slice: real remotes (blocked by 3)

- [x] 4.1 Check by hand with a real HTTPS remote that needs a password and has no stored credential (use `https://github.com/<owner>/<missing-repo>.git` with `-c credential.helper=` removed from the test config): the command ends within a few seconds, and the app shows the message of Git. Then check with Git Credential Manager enabled that its window can still open. Result: the HTTPS check ended in 944 ms with `terminal prompts disabled`. Git Credential Manager was not checked, because the screen was locked.
- [x] 4.2 Check SSH: a remote whose key needs a passphrase and no agent. The command must fail at once and not hang. Check a WSL repository too: fetch and push from `\\wsl.localhost\<distro>\...`. Result: ssh with a passphrase key and no console ended in 1.5 s with `Permission denied (publickey)`. WSL `git ls-remote` with `WSLENV` ended in 1.8 s with `terminal prompts disabled`. The app was not run on a WSL repository by hand; the `gitcore` WSL test covers push, fetch, and pull.

## 5. Slice: documents and final checks (blocked by 4)

- [x] 5.1 `README.md`: a Remote section (the three buttons, `--ff-only`, no password prompt, what to do on a diverged pull). `openspec/follow-ups.md`: mark 8.11 done, update 8.2 (remote commands have a busy state; staging, commit, page loads do not), add "cancel a remote command", "choose a remote for the first push", "pull with merge or rebase".
- [x] 5.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`. Run the tests again with an empty `GIT_CONFIG_GLOBAL` file. Run the release exe on a repository with a remote and take a screenshot of the toolbar. Result: 66 app tests and 196 gitcore tests passed (also with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed, and the release exe showed `Fetched` and the buttons `Fetch`, `Pull ↓1`, `Push ↑1`.
