## 1. Slice: create and delete in `gitcore`

- [x] 1.1 (tdd) `gitcore::branch::create`: tests cover a new branch at HEAD with HEAD moved to it, work-tree changes kept, an existing name (Git's error, HEAD unchanged), an invalid name with a space and with `..` (error, no branch), and a name that starts with `-` (error, no branch, no other branch touched).
- [x] 1.2 (tdd) `gitcore::branch::delete`: tests cover a merged branch (gone), an unmerged branch (error, branch stays), the current branch (error), a missing branch (error), and a branch whose name looks like an option (deleted as a name, other branches stay).
- [x] 1.3 A WSL test: create and delete inside a `WslScratch` repository when WSL is present.

## 2. Slice: panel state (blocked by 1)

- [x] 2.1 (tdd) `BranchPanel::can_delete` and the delete confirmation: tests cover local non-current → true; current, remote, and no selection → false; selecting another branch or showing new refs clears `confirming_delete`.

## 3. Slice: UI (blocked by 1, 2)

- [x] 3.1 Name `Input` and `Create` button under the branch list. The button is disabled while the trimmed name is empty or `busy()`. Success clears the field.
- [x] 3.2 `Delete` button with the inline confirmation (`Delete`, `Cancel`). Rename `finish_checkout` to `finish_branch_command` and use it for all three commands.
- [x] 3.3 Check by hand on a temp repository: create from a dirty tree, create an existing name, delete a merged branch, cancel a delete, delete an unmerged branch (error shows, branch stays), the current branch (button disabled).

## 4. Slice: documents and final checks (blocked by 3)

- [x] 4.1 `README.md`: v1 list and a Branches note. `openspec/follow-ups.md`: add force delete, create from a chosen start point, delete a remote branch.
- [x] 4.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests again with an empty `GIT_CONFIG_GLOBAL`. Take a release screenshot of the branch panel. Result: 71 app tests and 206 gitcore tests passed (also with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed. Checked by hand in the debug exe instead of a release screenshot.
