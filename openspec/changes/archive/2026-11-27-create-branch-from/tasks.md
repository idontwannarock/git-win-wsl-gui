## 1. Slice: `StartPoint` in `gitcore`

- [x] 1.1 (tdd) `create(location, name, Option<&StartPoint>)`: tests cover none (current commit, as before), `Commit` (branch at that commit, HEAD on it), `Local` (branch at that branch), `Remote` (branch at the remote-tracking commit; assert what Git does with tracking), a local branch named `-Cmain` as start (works as a name), a missing branch (git error, no new branch, HEAD unchanged), and the existing create tests with `None`. A WSL test uses a `Commit` start.

## 2. Slice: panel state (blocked by 1)

- [x] 2.1 (tdd) `BranchPanel.start`: tests cover set → stored with its label; clear → none; `show_refs` keeps it; a second set replaces the first.

## 3. Slice: UI (blocked by 1, 2)

- [x] 3.1 `RepoView::start_branch_from(point, label, window, cx)`: stores the start point and focuses the name field. `create_branch` passes the start point. `finish_create` clears it on success only.
- [x] 3.2 The "Start: `<label>`" line with a clear button, above the name field.
- [x] 3.3 Right-click menu on commit rows (`rows_in`) and branch rows (`branch_rows`) with the item `Create branch here`. Branch rows map `Local` and `Remote` by `RefKind`.
- [x] 3.4 Checked by hand in the debug exe on a temp repository with a remote: right-click a commit and create (branch at that commit, HEAD on it, "Start:" line gone), right-click a remote branch and create (branch at its commit, tracking `origin/feature`), a failed create keeps the "Start:" line, and `Clear` removes it. Not checked by hand: a successful create from a local branch row (the `gitcore` test covers it).

## 4. Slice: documents and final checks (blocked by 3)

- [x] 4.1 `README.md` Branches note. `openspec/follow-ups.md`: mark 8.29 done, add 8.35 (move other buttons into right-click menus).
- [x] 4.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests with an empty `GIT_CONFIG_GLOBAL`. Result: 87 app tests and 227 gitcore tests passed (with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed.
