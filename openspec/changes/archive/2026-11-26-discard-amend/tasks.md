## 1. Slice: discard in `gitcore`

- [x] 1.1 (tdd) `WorkingFile::can_discard`: tests cover untracked, unstaged modified, deleted, and type-changed → true; staged, conflicted, and unstaged added, renamed, and copied → false.
- [x] 1.2 (tdd) `index::discard`: tests cover a modified file (content of the index comes back), a deleted file (comes back), an untracked file (gone, other untracked files stay), a file with staged and unstaged changes (staged content stays), a name with a leading dash and a bracket (taken as a plain name), and a file that cannot be discarded (error, nothing changes).
- [x] 1.3 A WSL test: discard a modified and an untracked file inside a `WslScratch` repository.

## 2. Slice: amend in `gitcore` (independent of 1)

- [x] 2.1 (tdd) `index::amend` and `index::last_message`: tests cover amend with a staged file (commit count the same, file in the commit, new message), amend with nothing staged (message only), `last_message` of a multi-line message, a failed hook (error text, last commit unchanged), and a repository with no commit (error). `commit` keeps its tests and shares the error path.
- [x] 2.2 A WSL test: amend inside a `WslScratch` repository.

## 3. Slice: app state (blocked by 1, 2)

- [x] 3.1 (tdd) `Working`: `confirming_discard` (ask only when `can_discard`; selection change and a refresh that drops the file clear it) and `can_commit(message, amending)` (commit needs staged files; amend needs a branch with a commit and no staged file; both need a message and no busy action).

## 4. Slice: UI (blocked by 3)

- [x] 4.1 `Discard` button in the Unstaged header, and the inline confirmation (`Discard`, `Cancel`) under the unstaged list. Discard runs through `run_index_action`.
- [x] 4.2 `Amend` toggle by the `Commit` button. Turning it on fills an empty box with `last_message`. The commit button reads `Amend` while it is on. After success the toggle turns off and the box is empty.
- [x] 4.3 Check by hand on a temp repository: discard a modified, a deleted, and an untracked file; cancel a discard; a file with staged and unstaged changes; amend with a staged file; amend the message only; the box is filled; a typed message is kept; amend disabled on a repository with no commit.

## 5. Slice: documents and final checks (blocked by 4)

- [x] 5.1 `README.md`: a Discard note and an Amend note (including that an amended pushed commit needs a force push, which the app never does). `openspec/follow-ups.md`: add discard all, undo of a discard, a warning for a pushed amend.
- [x] 5.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests again with an empty `GIT_CONFIG_GLOBAL`. Result: 75 app tests and 220 gitcore tests passed (also with an empty `GIT_CONFIG_GLOBAL`, where `core.autocrlf=true` writes CRLF, so the discard tests ignore line endings), clippy was clean, the release build passed.
