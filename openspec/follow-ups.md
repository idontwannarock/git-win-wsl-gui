# Follow-ups

Open work found while building `v1-core` (archived in `openspec/changes/archive/2026-11-24-v1-core/`). Item 8.7 covers the skipped tasks 3.4 and 7.2.

- [ ] 8.1 Show more graph lanes. Today the graph draws 8 lanes (`VISIBLE_LANES` in `crates/app/src/graph.rs`); lines and dots for lane 8 and up are not drawn. The lane layout in `gitcore` has no limit. Decide the graph column width for many lanes (grow, scroll, or clip with a marker), then remove the fixed limit. Test with a repository that has more than 8 branches open at once.
- [ ] 8.2 Show a busy state for staging, commit, checkout, and page loads (spec `repo-access`, "Slow command": more than 100 ms shows a busy state for that panel). Only opening a repository has one today.
- [ ] 8.3 Space stages or unstages the focused row, not only the selected row (`toggle_selected` in `crates/app/src/working.rs`).
- [ ] 8.4 Name the WSL distro in the error when `wsl.exe` or `git` is missing in it (`crates/gitcore/src/repo.rs`). Add a test.
- [ ] 8.5 Reload the graph when a branch changes outside the app (design D4 says to kill the log process on a ref change).
- [ ] 8.6 Make the contrast test cover every text pair the app uses, and the border color against the paper color (1.45:1 today; unused so far).
- [ ] 8.7 Benchmarks (tasks 3.4 and 7.2): 50,000-line diff scroll, re-run of the 2.5 numbers after 2.6, and the steady 33.3 ms slow state of the graph (cause unknown).
- [x] 8.8 Done in repo-tabs: each tab is its own `RepoView` entity, and every task reports to its own entity. A late result cannot reach another repository.
- [ ] 8.9 `git log --all` draws stash and notes commits as plain commits without a label (`crates/gitcore/src/stream.rs`). Use `--branches --remotes --tags HEAD`, or `--exclude=refs/stash --exclude=refs/notes/*`.
- [ ] 8.10 A conflicted file shows a failed diff, because `git diff` prints `diff --cc` and the `gpui-component` parser rejects it. Show a clear message for it.
- [ ] 8.11 `Status.branch` (`crates/gitcore/src/status.rs`) and `GraphRow.width` (`crates/gitcore/src/lanes.rs`) are never read outside tests. Remove them, or use them.
- [ ] 8.12 Show an error when the folder picker fails (`choose_folder` in `crates/app/src/shell.rs`); a cancel stays silent.
- [ ] 8.13 The parse errors of `status`, `changes`, and `refs` hide the bad record and do not reach `app.log`. Add the record to the message and to the log.
- [ ] 8.14 `wsl.exe` writes its own errors to stdout in UTF-16LE, so the UI shows an empty error for an unknown distro. Fold this into 8.4.
- [ ] 8.15 Opening an untrusted folder runs its `core.fsmonitor`, and later its hooks. This is the behavior of Git itself. Decide whether to add a trust prompt.
- [ ] 8.16 Design cleanup from the review: one status badge and one `patch_paths` rule instead of two, one diff panel `match` instead of two, one test repository helper instead of four, and `GITGUI_BENCH` outside the shipped `RepoView`.
- [ ] 8.17 Keyboard shortcuts for tabs: `Ctrl+Tab` and `Ctrl+Shift+Tab` to switch, `Ctrl+W` to close, `Ctrl+T` to open, and maybe `Ctrl+1` to `Ctrl+9`. Until then the tab bar is mouse-only; check whether the Tab key reaches it.
- [ ] 8.18 Drag a tab to reorder the tab bar.
- [ ] 8.19 Mark a background tab that has changes. This needs a status read for background tabs; decide how often.
- [ ] 8.20 Drop the loaded history of tabs that were not active for a long time, to save memory. Reload it on activation.
- [ ] 8.21 Tab bookkeeping from the repo-tabs review: `Tab.key` always equals `key_of(&Tab.source)`, so drop the field and compute the key in `find`; the "no duplicate tab" check sits in three places of `Shell` (`open`, `restore`, `on_opened`) and `TabList::open` pushes without a check, so give `TabList` a key rule with unit tests (design D10(d) planned this); `TabList::active()` returns a bare index and `Shell` uses `usize::MAX` and `0` as stand-ins, so add `active_entry()`.
