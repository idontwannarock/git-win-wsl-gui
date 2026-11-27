## 1. Slice: `DeleteMode` in `gitcore`

- [x] 1.1 (tdd) `delete(location, name, mode)`: tests cover `Force` on an unmerged branch (gone), `Force` on the current branch (error, branch stays), `IfMerged` on an unmerged branch (error, branch stays; existing test), and the existing merged, missing, and option-like name tests with `IfMerged`. A WSL test uses `Force`.

## 2. Slice: panel state (blocked by 1)

- [x] 2.1 (tdd) `DeletePrompt` and `offer_force`: tests cover ask → `IfMerged`; `offer_force` for the selected branch → `Force`; `offer_force` for another name → no prompt change; select, `show_refs`, and `cancel` clear it.

## 3. Slice: UI (blocked by 1, 2)

- [x] 3.1 `confirm_delete` passes the mode. A failed `IfMerged` delete calls `offer_force`. The prompt text and the confirm button label follow the mode.
- [x] 3.2 Checked by hand in the debug exe on a temp repository: an unmerged branch (git error and force prompt show), `Cancel` at the force prompt (branch stays), `Force delete` (branch gone). Not checked by hand: a merged branch and a refused force delete (the `gitcore` tests cover both).

## 4. Slice: documents and final checks (blocked by 3)

- [x] 4.1 `README.md` Branches note. `openspec/follow-ups.md`: mark 8.28 done.
- [x] 4.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests with an empty `GIT_CONFIG_GLOBAL`. Result: 79 app tests and 222 gitcore tests passed (also with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed.
