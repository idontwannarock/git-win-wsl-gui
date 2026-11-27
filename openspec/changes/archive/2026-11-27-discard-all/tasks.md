## 1. Slice: gitcore

- [x] 1.1 (tdd) `discard_all(location, files)` in `crates/gitcore/src/index.rs`. Tests on a temporary repository: modified and deleted files are restored (a type change is not tested: a symbolic link is not reliable on Windows); an untracked file, a staged-only file, and a conflicted file stay; staged changes of a file with more unstaged edits stay; an empty or non-discardable list is a no-op.

## 2. Slice: prompt state (blocked by 1)

- [x] 2.1 (tdd) `DiscardPrompt` holds `DiscardTarget { File, All }`. Tests: ask file; ask all with discardable files opens `All`; ask all with none opens nothing; cancel; the keep rule for `All` (kept while a file is left, closed at 0) and for `File` (as before).

## 3. Slice: UI (blocked by 2)

- [x] 3.1 The Unstaged header has a right-click menu with `Discard all changes`, disabled when busy or at count 0. The confirmation shows "Discard changes to N files?" for `All`. `confirm_discard` runs `discard_all` for `All`.
- [x] 3.2 Check by hand in the debug exe on a temp repository: right-click, count text, `Cancel`, `Discard` (modified and deleted restored, untracked stays), disabled item at count 0.

## 4. Slice: documents and final checks (blocked by 3)

- [x] 4.1 `README.md` Working tree note, including the long path list limit. `openspec/follow-ups.md`: split 8.31.
- [x] 4.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`, and the tests with an empty `GIT_CONFIG_GLOBAL`. Result: 94 app tests and 231 gitcore tests passed (with an empty `GIT_CONFIG_GLOBAL`), clippy was clean, the release build passed, `openspec validate` passed.
