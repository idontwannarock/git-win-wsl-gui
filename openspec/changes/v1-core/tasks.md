## 1. Slice: open a repo and show the last commits (Windows + WSL), end to end

- [x] 1.1 Create the Cargo workspace (`gitcore`, `app`). Pin `gpui` and `gpui-component` with `=`. Commit `Cargo.lock`. A window opens and shows "hello".
- [x] 1.2 (tdd) `gitcore`: path-to-runner mapping for `C:\...`, `\\wsl$\<d>\...`, `\\wsl.localhost\<d>\...`. Tests cover all three forms and a non-path input.
- [x] 1.3 (tdd) `gitcore`: run a command through a runner. Test with a real temp repository on the Windows runner. Test the WSL runner when a distro exists (skip with a visible message otherwise).
- [x] 1.4 (tdd) `gitcore`: `git log` parser for the NUL-separated format. Fixture-based tests, including merge commits and empty body.
- [x] 1.5 `app`: folder picker, open repository, show the last 100 commits as a plain list. Reject a non-repository with an error. Check by hand on a Windows repo and a WSL repo.
- [ ] 1.6 Record the syntax-highlighter answer in `design.md` Open Questions (check `gpui-component` first). Check Traditional Chinese text input in a `gpui-component` input box.

## 2. Slice: commit graph (blocked by 1)

- [ ] 2.1 (tdd) `gitcore`: incremental lane layout on fixed histories (linear, fork, merge, octopus).
- [ ] 2.2 `gitcore`: streaming `git log --all --topo-order` reader with batches and back-pressure. Kill on close. Test with a temp repository.
- [ ] 2.3 `app`: virtual graph list with lanes, subject, author, date, hash, ref labels. Load the next page on scroll.
- [ ] 2.4 (tdd) `gitcore`: `for-each-ref` parser for ref labels.
- [ ] 2.5 Benchmark harness: generate a 100,000-commit repository. Measure time to first page and frame time while scrolling, on the Windows runner and on the WSL runner. Record the numbers in this file.

## 3. Slice: commit detail and unified diff (blocked by 2)

- [ ] 3.1 (tdd) `gitcore`: unified diff parser (hunks, line numbers, binary, rename, no-newline marker). Fixture-based tests.
- [ ] 3.2 `gitcore`: changed-file list for a commit (`git show --name-status -z`) with a parser test.
- [ ] 3.3 `app`: select a commit, show message and changed files. Select a file, show its diff in a virtual list with line numbers, markers, and colors.
- [ ] 3.4 `app`: syntax color for visible lines (per the answer in 1.6).
- [ ] 3.5 Benchmark: scroll a 50,000-line diff. Record the numbers in this file.

## 4. Slice: working tree, stage, commit (blocked by 3)

- [ ] 4.1 (tdd) `gitcore`: `status --porcelain=v2 -z --branch` parser. Fixtures for modified, added, deleted, renamed, untracked, conflicted.
- [ ] 4.2 (tdd) `gitcore`: stage, unstage, stage all, commit. Real temp repository tests, including a failing pre-commit hook.
- [ ] 4.3 `app`: staged and unstaged lists, diff of the selected file (index vs HEAD, work tree vs index), Space to toggle, "Stage all".
- [ ] 4.4 `app`: commit message box and commit action. Disable on empty message or nothing staged. Show hook output on failure.
- [ ] 4.5 `app`: refresh on focus, after own actions, and every 2 seconds while focused.

## 5. Slice: branches (blocked by 2)

- [ ] 5.1 (tdd) `gitcore`: branch list (local, remote, current) and checkout, including the tracking branch for a remote name. Real temp repository tests, including a refused checkout.
- [ ] 5.2 `app`: branch side panel, double-click and menu checkout, error text on failure. Graph and status update after checkout.

## 6. Slice: theme and accessibility (can start after 1; finish last)

- [ ] 6.1 (tdd) `app::theme`: Console tokens, mapped to the `gpui-component` theme. A test checks every text/background pair against the 4.5:1 and 3:1 floors.
- [ ] 6.2 Bundle Host Grotesk and Atkinson Hyperlegible Mono. Keep their OFL license files in the repository.
- [ ] 6.3 Visible focus ring and Tab order on every view. Check every v1 action by keyboard only.
- [ ] 6.4 Turn off all animation when Windows "show animations" is off.
- [ ] 6.5 Design review against the "no ornament" list in the `ui-theme` spec.

## 7. Final checks (blocked by 1-6)

- [ ] 7.1 Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo build --release`. Fix all failures.
- [ ] 7.2 Re-run the benchmarks from 2.5 and 3.5. Confirm every target in the specs. Record any miss as a follow-up.
- [ ] 7.3 Write `README.md`: what it is, how to build, requirements (git on Windows, git in each WSL distro).
