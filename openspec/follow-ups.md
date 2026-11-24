# Follow-ups

Open work found while building `v1-core` (archived in `openspec/changes/archive/2026-11-24-v1-core/`). Item 8.7 covers the skipped tasks 3.4 and 7.2.

- [ ] 8.1 Show more graph lanes. Today the graph draws 8 lanes (`VISIBLE_LANES` in `crates/app/src/graph.rs`); lines and dots for lane 8 and up are not drawn. The lane layout in `gitcore` has no limit. Decide the graph column width for many lanes (grow, scroll, or clip with a marker), then remove the fixed limit. Test with a repository that has more than 8 branches open at once.
- [ ] 8.2 Show a busy state for staging, commit, checkout, and page loads (spec `repo-access`, "Slow command": more than 100 ms shows a busy state for that panel). Only opening a repository has one today.
- [ ] 8.3 Space stages or unstages the focused row, not only the selected row (`toggle_selected` in `crates/app/src/working.rs`).
- [ ] 8.4 Name the WSL distro in the error when `wsl.exe` or `git` is missing in it (`crates/gitcore/src/repo.rs`). Add a test.
- [ ] 8.5 Reload the graph when a branch changes outside the app (design D4 says to kill the log process on a ref change).
- [ ] 8.6 Make the contrast test cover every text pair the app uses, and the border color against the paper color (1.45:1 today; unused so far).
- [ ] 8.7 Benchmarks (tasks 3.4 and 7.2): 50,000-line diff scroll, re-run of the 2.5 numbers after 2.6, and the steady 33.3 ms slow state of the graph (cause unknown).
