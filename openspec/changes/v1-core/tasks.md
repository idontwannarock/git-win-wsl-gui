## 1. Slice: open a repo and show the last commits (Windows + WSL), end to end

- [x] 1.1 Create the Cargo workspace (`gitcore`, `app`). Pin `gpui` and `gpui-component` with `=`. Commit `Cargo.lock`. A window opens and shows "hello".
- [x] 1.2 (tdd) `gitcore`: path-to-runner mapping for `C:\...`, `\\wsl$\<d>\...`, `\\wsl.localhost\<d>\...`. Tests cover all three forms and a non-path input.
- [x] 1.3 (tdd) `gitcore`: run a command through a runner. Test with a real temp repository on the Windows runner. Test the WSL runner when a distro exists (skip with a visible message otherwise).
- [x] 1.4 (tdd) `gitcore`: `git log` parser for the NUL-separated format. Fixture-based tests, including merge commits and empty body.
- [x] 1.5 `app`: folder picker, open repository, show the last 100 commits as a plain list. Reject a non-repository with an error. Check by hand on a Windows repo and a WSL repo.
- [x] 1.6 Record the syntax-highlighter answer in `design.md` (check `gpui-component` first). Check Traditional Chinese text input in a `gpui-component` input box. Result: syntax color comes from `Diff` (design D11). Unicode text shows correctly. Real IME composition is untested and needs the user.

## 2. Slice: commit graph (blocked by 1)

- [x] 2.1 (tdd) `gitcore`: incremental lane layout on fixed histories (linear, fork, merge, octopus).
- [x] 2.2 `gitcore`: streaming `git log --all --topo-order` reader with batches and back-pressure. Kill on close. Test with a temp repository.
- [x] 2.3 `app`: virtual graph list with lanes, subject, author, date, hash. Load the next page on scroll.
- [x] 2.4 (tdd) `gitcore`: `for-each-ref` parser for ref labels. Then draw the labels in the graph list.
- [x] 2.5 Benchmark harness: generate a 100,000-commit repository. Measure time to first page and frame time while scrolling, on the Windows runner and on the WSL runner. Record the numbers in this file.

  How to run (release build, 2026-11-20; Core i7-13620H, Git for Windows 2.56.0, WSL Ubuntu git 2.34.1, display 60 Hz):
  - `cargo run --release -p gitcore --example first_page -- generate D:\ws\bench100k 100000 10` (100,000 commits, 9,999 merges; also run on `\\wsl.localhost\Ubuntu\home\me\bench100k`)
  - `first_page.exe measure <path>` prints the time of each open step, 5 runs.
  - `set GITGUI_BENCH=1` then `target\release\app.exe <path>` scrolls 30,000 rows at 10 rows per frame and prints frame times.

  Results (median of the runs; ms):

  | | Windows runner | WSL runner |
  |---|---|---|
  | open to first rows, no commit-graph | 1,290 - 3,450 (noisy) | 1,710 - 1,880 |
  | open to first rows, with commit-graph | 870 - 940 | 1,460 - 1,590 |
  | git result arrives to rows visible (app) | 21 - 22 | 24 |
  | frame time p50 / p95 / p99 / max (app) | 16.7 / 17.7 / 18.4 / 20.5 | 16.7 / 17.7 / 18.3 / 22.4 |
  | frames over 20 ms (of 2,999) | 2 | 2 |
  | rows waited for data while scrolling | 0 | 0 |

  Explain the numbers (explain-the-number):
  - Spec "first page within 200 ms of the command result": met. The app shows the rows 21 - 24 ms after the batch arrives.
  - Open to first rows is 0.7 - 1.9 s. Three `git` calls run one after the other (`rev-parse`, `for-each-ref`, `log`). Each call pays the cost of starting a process. Measured from PowerShell on this machine (the numbers include PowerShell itself): `cmd /c exit` takes 73 - 88 ms, `git.exe` from `cmd\git.exe` takes 171 - 216 ms (it starts the real git a second time), the real `ucrt64\bin\git.exe` takes 93 - 130 ms, and `wsl.exe -d Ubuntu --exec true` takes 456 - 492 ms. In the benchmark each Windows step takes about 300 ms and each WSL step 540 - 680 ms (the last step is shorter with a commit-graph). The work inside git is small by comparison.
  - `git log --all --topo-order` must sort all 100,000 commits before it prints the first one. First byte: 500 ms without a commit-graph, 210 ms with one (real git, measured with the process start included). On WSL the `log` step takes 600 - 650 ms without a commit-graph and 320 - 350 ms with one, so the sort costs about 280 ms there.
  - Frame time: p50 16.7 ms is the 60 Hz screen interval, so the app is limited by the screen, not by our work. Building the visible rows costs 65 us on average. No frame took 33 ms or more, so no frame was dropped. The p95 of 17.7 ms is 1 ms above the 16.6 ms in the spec: a 60 Hz screen cannot show a frame in less than 16.7 ms, so that threshold cannot be met as written. This benchmark does not show how much time is left inside each frame, because the screen limits the rate.
  - Not proven: scrolling by mouse wheel (the benchmark moves the list by code), and the effect of paint and GPU time on a slower machine.

- [x] 2.6 Shorten the open path (follow-up from 2.5). Done: (a) call the real `ucrt64\bin\git.exe` and skip `cmd\git.exe`; (b) `open_repository` runs `rev-parse`, `for-each-ref`, and `log` at the same time. Not done: (c) one `wsl.exe` call for all three steps (parallel calls already gave most of the gain; revisit only if 7.2 needs it; a long-lived shell per distro is the only way to remove the `wsl.exe` start cost); (d) writing a commit-graph into the repository. Decided no (user, 2026-11-20): the app does not change the repository. Git writes it in `git gc` and in `git maintenance`; the README tells the user to run `git maintenance start`.

  Open to first rows after 2.6 (ms; first_page example, 5 runs; before 2.6 in brackets):

  | | Windows runner | WSL runner |
  |---|---|---|
  | with commit-graph | 310 - 420 (870 - 940) | 730 - 950 (1,460 - 1,590) |
  | no commit-graph | 650 - 780 (1,290 - 3,450) | 900 - 1,210 (1,710 - 1,880) |

  - (a) alone: each Windows step fell from about 300 ms to 215 - 230 ms. Hooks still run with the real git, with only `C:\Windows\System32;C:\Program Files\Git\cmd` on `PATH` (checked with a failing `pre-commit` hook).
  - (b): the three steps now cost about as much as the slowest one plus some contention. The remaining Windows time is one process start (about 130 ms) plus the `--topo-order` sort (about 110 ms with a commit-graph, about 400 ms without).
  - WSL still pays one `wsl.exe` start (about 500 ms on this machine) before anything runs.
  - Not understood yet: frame time depends on the machine state. Three runs on the earlier clean state and one later run gave p50 16.7 ms and p99 18.4 - 18.8 ms. Other runs gave a steady 33.3 ms. The binary from before 2.6 gave 33.3 ms in the same period, so 2.6 did not cause it. CPU use by other programs was 7 - 49 % before those runs, both screens are 60 Hz, and the power cable was connected. Row building cost 65 us in the clean state and 117 - 129 us in the slow state. The benchmark cannot yet split frame time into CPU work and GPU or screen wait. Next step before 7.2: measure the time of one draw call.
- [x] 2.7 Fix the wording of the "Fast scroll" scenario in `specs/commit-graph/spec.md` and `specs/diff-view/spec.md`. A 60 Hz screen cannot show a frame in less than 16.7 ms, so "16.6 ms at p95" could not be met. New wording: at least 99 percent of the frames take 20 ms or less, and no frame takes 33 ms or more.

## 3. Slice: commit detail and unified diff (blocked by 2)

- [x] 3.1 (tdd) `gitcore`: get the patch text of one file of a commit (`git show`) with the right arguments (no color, no text conversion, rename detection, first parent for a merge). The caller passes both the new and the old path for a renamed file. `git show` does not run an external diff by default, so `--no-ext-diff` was left out. Real temp repository tests: modified, binary, renamed, no newline at end of file. Each patch must parse with `DiffFile::parse`.
- [x] 3.2 (tdd) `gitcore`: changed-file list for a commit (`git show --name-status -z`) with a parser test.
- [x] 3.3 `app`: select a commit, show message and changed files. Select a file, show its patch in the `gpui-component` `Diff` in unified mode (design D11).
- [ ] 3.4 (deferred by the user on 2026-11-20: other programs on the machine disturb the frame times; do it with 7.2) Benchmark: scroll a 50,000-line diff. Record the numbers in this file. If `Diff` misses the target, record the cause (explain-the-number) before choosing a fix.

## 4. Slice: working tree, stage, commit (blocked by 3)

- [x] 4.1 (tdd) `gitcore`: `status --porcelain=v2 -z --branch` parser. Fixtures for modified, added, deleted, renamed, untracked, conflicted.
- [x] 4.2 (tdd) `gitcore`: stage, unstage, stage all, commit. Real temp repository tests, including a failing pre-commit hook.
- [x] 4.3 `app`: staged and unstaged lists, diff of the selected file (index vs HEAD, work tree vs index), Space to toggle, "Stage all".
- [x] 4.4 `app`: commit message box and commit action. Disable on empty message or nothing staged. Show hook output on failure.
- [x] 4.5 `app`: refresh on focus, after own actions, and every 2 seconds while focused.

## 5. Slice: branches (blocked by 2)

- [x] 5.1 (tdd) `gitcore`: branch list (local, remote, current) and checkout, including the tracking branch for a remote name. Real temp repository tests, including a refused checkout.
- [x] 5.2 `app`: branch side panel, double-click and menu checkout, error text on failure. Graph and status update after checkout.

## 6. Slice: theme and accessibility (can start after 1; finish last)

- [ ] 6.1 (tdd) `app::theme`: Console tokens, mapped to the `gpui-component` theme. A test checks every text/background pair against the 4.5:1 and 3:1 floors.
- [ ] 6.2 Bundle Host Grotesk and Atkinson Hyperlegible Mono. Keep their OFL license files in the repository.
- [ ] 6.3 Visible focus ring and Tab order on every view. Check every v1 action by keyboard only.
- [ ] 6.4 Turn off all animation when Windows "show animations" is off.
- [ ] 6.5 Design review against the "no ornament" list in the `ui-theme` spec.

## 7. Final checks (blocked by 1-6)

- [ ] 7.1 Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo build --release`. Fix all failures.
- [ ] 7.2 Re-run the benchmarks from 2.5 and 3.4. Confirm every target in the specs. Record any miss as a follow-up.
- [ ] 7.3 Write `README.md`: what it is, how to build, requirements (git on Windows, git in each WSL distro), and the tip to run `git maintenance start` for faster big repositories.
