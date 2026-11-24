## 1. Slice: shell and repository view, one tab, same behavior as today

- [ ] 1.1 (tdd) `app`: a pure `TabList` type with no gpui types. It holds entries and an active index. Tests cover: open adds at the end and activates; activate; close of a middle, last, and only entry; which entry becomes active after each close (right neighbor, else left, else none).
- [ ] 1.2 Rename `Home` to `RepoView`. Remove its "Open repository..." button, its poll, and its window-activation observer. Add `Shell` with a `TabList` and an `Entity<RepoView>` for each entry. `main.rs` opens `Shell`. The command-line path calls `Shell::open`. With one repository, the app behaves as before. Check by hand on a Windows repository: open, history, select commit, Changes, stage, commit, branch switch, 2-second refresh. Do not change logic in this step.

## 2. Slice: many tabs with the mouse (blocked by 1)

- [ ] 2.1 (tdd) `app`: `titles(paths) -> Vec<String>`. Tests cover: one tab, two different names, two equal names (parent folder added to both), a WSL path, a root path with no parent.
- [ ] 2.2 `app`: the tab bar with `gpui-component` `TabBar` and `Tab`: title, full path on hover, `×` button in the tab, `+` button at the end, middle click closes, the bar scrolls when tabs overflow. Opening from `+` or from the empty-screen button opens a new tab and activates it. Record in `design.md` D8 whether `Tab` passes the middle click or the wrapper `div` was needed. Check by screenshot with three repositories.
- [ ] 2.3 `Shell`: move the 2-second poll and the window-activation observer into `Shell`. They act on the active tab only. Activating a tab focuses it and runs one `refresh_working`. Check: with two tabs open for 10 seconds, `%LOCALAPPDATA%\git-win-wsl-gui\app.log` shows `git status` for the active repository only.
- [ ] 2.4 Close drops the tab entity. When the last tab closes, show the empty screen. Test: dropping a `LogStream` ends its git process (add the test only if none exists). Check by hand: close a tab while a large repository still reads, and confirm no `git.exe` stays in Task Manager.
- [ ] 2.5 Check follow-up 8.8 by hand: open a large repository, switch to another tab before it finishes, and confirm the other tab does not change. Repeat with a branch switch (it reloads history) and a tab switch during the reload.

## 3. Slice: no duplicate tab (blocked by 2)

- [ ] 3.1 (tdd) `gitcore`: `RepoLocation::key() -> RepoKey`. Tests cover: drive-letter case, path case, trailing separator on Windows; `wsl$` equals `wsl.localhost`; distro-name case ignored; Linux path case kept; trailing `/` ignored; different runners never equal.
- [ ] 3.2 `Shell::open`: compare the parsed key with the open tabs before the read, and compare the resolved top-level key when the read ends. On a match, activate the old tab and close the new one. Check by hand: open the same folder twice with different case, open a sub-folder, open the WSL repository in both forms.

## 4. Slice: restore tabs (blocked by 2, 3)

- [ ] 4.1 (tdd) `app`: `encode` and `decode` for `tabs.txt`. Tests cover: round trip, empty input, non-number first line, index out of range, blank lines, a lone index with no paths.
- [ ] 4.2 `Shell`: save after each open, close, and activation. A write error goes to the debug log only. At start, read the file, create a `RepoView` with a `Pending` screen for each saved path, and open only the active one. A `Pending` view opens on its first activation. Add `Screen::Pending`. Check by hand: restart with three tabs, confirm only the active tab ran git (debug log); remove one folder and confirm only its tab shows an error; damage the file and confirm the app starts empty; start with a command-line path and confirm its tab is active.

## 5. Slice: documents and final checks (blocked by 4)

- [ ] 5.1 `README.md`: add a Tabs section (open, close, restore, background tabs are quiet). Change the sentence "Every v1 action works without a mouse" to name the tab bar as the exception. `openspec/follow-ups.md`: mark 8.8 done. Add: keyboard shortcuts for tabs, drag to reorder, a change mark on background tabs, drop the history of far-away tabs.
- [ ] 5.2 Run `cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`. Run the tests again with an empty `GIT_CONFIG_GLOBAL` file. Run the release exe with three tabs and take a screenshot. Confirm `grep -rl GITGUI_TMP crates` finds nothing.
