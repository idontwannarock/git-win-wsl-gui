# Follow-ups

Open work found while building `v1-core` (archived in `openspec/changes/archive/2026-11-24-v1-core/`). Item 8.7 covers the skipped tasks 3.4 and 7.2.

- [x] 8.1 Done: the graph column keeps a fixed width of 8 lanes. A row with more than 8 lanes draws the first 6 and shows `+N` for the rest (`lane_fit` in `crates/app/src/graph.rs`). The column does not grow, because the default window is already taller than a 1080-pixel screen (see 8.22). A reader still cannot follow a hidden lane; a hover or a scroll for the graph column is open work.
- [ ] 8.2 Show a busy state for staging, commit, checkout, and page loads (spec `repo-access`, "Slow command": more than 100 ms shows a busy state for that panel). Opening a repository and the remote commands (Fetch, Pull, Push) have one today.
- [ ] 8.3 Space stages or unstages the focused row, not only the selected row (`toggle_selected` in `crates/app/src/working.rs`).
- [ ] 8.4 Name the WSL distro in the error when `wsl.exe` or `git` is missing in it (`crates/gitcore/src/repo.rs`). Add a test.
- [ ] 8.5 Reload the graph when a branch changes outside the app (design D4 says to kill the log process on a ref change).
- [ ] 8.6 Make the contrast test cover every text pair the app uses, and the border color against the paper color (1.45:1 today; unused so far).
- [ ] 8.7 Benchmarks (tasks 3.4 and 7.2): 50,000-line diff scroll, re-run of the 2.5 numbers after 2.6, and the steady 33.3 ms slow state of the graph (cause unknown).
- [x] 8.8 Done in repo-tabs: each tab is its own `RepoView` entity, and every task reports to its own entity. A late result cannot reach another repository.
- [ ] 8.9 `git log --all` draws stash and notes commits as plain commits without a label (`crates/gitcore/src/stream.rs`). Use `--branches --remotes --tags HEAD`, or `--exclude=refs/stash --exclude=refs/notes/*`.
- [ ] 8.10 A conflicted file shows a failed diff, because `git diff` prints `diff --cc` and the `gpui-component` parser rejects it. Show a clear message for it.
- [x] 8.11 Done in remote-sync: `Working.branch` keeps `Status.branch`, and the Pull and Push buttons show its ahead and behind counts.
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
- [x] 8.22 Done: the window opens centered in the visible screen area, at 1280 x 800 pixels or 90 percent of the visible area, whichever is smaller (`default_size` in `crates/app/src/window_size.rs`). Before, gpui used 1536 x 1095, which on a 1080-pixel screen put the title bar above the screen edge. The app still does not remember the size or position of the last session; add that if it is wanted.
- [ ] 8.23 Cancel a running Fetch, Pull, or Push. A slow or stuck remote keeps the buttons grey until Git ends. Closing the tab does not stop the Git process.
- [ ] 8.24 Choose the remote for the first push of a branch. Today it is `origin`, and a repository without `origin` shows the error of Git.
- [ ] 8.25 Pull with merge or rebase, and show a conflict state. Today `Pull` is `--ff-only`, and a diverged branch needs a terminal.
- [ ] 8.26 The result line of a remote command ("Pushed", an error) sits above the branch list and moves the list down when it appears. Reserve the space, or show the line inside the toolbar.
- [ ] 8.27 The result line of a remote command stays until the next remote command. Clear it when the user starts another action (stage, commit, checkout).
- [ ] 8.28 Force delete (`git branch -D`) for a branch that is not fully merged. Today the app shows the message of Git, and the user runs the command in a terminal.
- [ ] 8.29 Create a branch from a chosen commit or branch. Today a new branch always starts at the current commit.
- [ ] 8.30 Delete a remote branch (`git push origin --delete <name>`), and rename a branch.
- [ ] 8.31 Discard all unstaged changes, and discard a hunk or a line. Today `Discard` works on one file.
- [ ] 8.32 Undo a discard. Today a discard cannot be undone, and only the confirmation step guards it.
- [ ] 8.33 Warn before an amend of a commit that is already pushed. Today the next push is refused and the app never forces a push.
- [ ] 8.34 The confirmation of a discard (and of a branch delete) moves the lists when it appears. Reserve the space or show it in a fixed place. Same cause as 8.26.
