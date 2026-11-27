## 1. Slice: panel state

- [x] 1.1 (tdd) Replace `offer_force` with `ask_again(name)` and add `toggle_force`. Tests cover: ask → `IfMerged`; toggle → `Force`, toggle again → `IfMerged`; toggle with no prompt → no prompt; `ask_again` for the selected branch → `IfMerged` prompt; `ask_again` for another name → no prompt; select, `show_refs` for a gone or current branch, and `cancel` clear it; `show_refs` for a branch that can still be deleted keeps the prompt and its mode.

## 2. Slice: UI (blocked by 1)

- [x] 2.1 The confirmation shows a `Checkbox` "Force (delete even if not merged)". The label of the question and the button do not change with the mode. `finish_delete` calls `ask_again` after a failed `IfMerged` delete.
- [x] 2.2 Checked by hand in the debug exe on a temp repository: box off on an unmerged branch (git error shows, question stays open, branch stays), tick then `Delete` (branch gone), tick first on an unmerged branch (branch gone), `Cancel` (branch stays), and a merged branch with the box off (branch gone). Not checked by hand: a refused forced delete (the `gitcore` tests cover it).

## 3. Slice: documents and final checks (blocked by 2)

- [x] 3.1 `README.md` Branches note. `openspec/follow-ups.md`: note in 8.30 that the remote delete gets its own box "Also delete the remote branch".
- [x] 3.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests with an empty `GIT_CONFIG_GLOBAL`. Result: 83 app tests and 222 gitcore tests passed (also with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed.
