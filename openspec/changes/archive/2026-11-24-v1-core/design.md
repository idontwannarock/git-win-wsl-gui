## Context

The project is new. The directory has no code. Fork is the feature reference. Glimpse (Tauri, Rust backend) is the WSL reference: it shells out to `git` and picks WSL git from the path. This project uses GPUI to get native rendering speed on large repositories.

Facts checked on 2026-11-20:
- `gpui` on crates.io is 0.2.2. `gpui-component` is 0.10.0 (Apache-2.0). It depends on `gpui-pre-*` snapshots of Zed's crates, so its API changes often.
- Hallmark is a web design skill. It gives design rules and CSS tokens. `modern-web-guidance` gives web-platform API advice and does not apply to GPUI.

## Goals / Non-Goals

**Goals:**
- A usable daily Git GUI for Windows and WSL repositories (v1 features in the proposal).
- 100,000-commit repository: first graph page under 200 ms, 60 fps scroll for graph and diff.
- Keep the git logic testable without a window.

**Non-Goals:**
- Hunk or line staging, side-by-side or word-level diff, rebase, stash, merge editor, light theme.
- macOS and Linux.
- Any git logic that does not come from the real `git` binary.

## Decisions

### D1. Two crates in one Cargo workspace
`gitcore` holds the runner, the output parsers, and the graph lane layout. It has no GPUI dependency. `app` holds the GPUI views and the theme. Tests and benchmarks for git logic run in `gitcore` without a window.
Alternative: one crate. Rejected: it mixes pure logic with UI and makes tests slow to build.

### D2. Shell out to the real git <!-- evergreen-candidate -->
All repository data comes from `git` output. We parse machine formats: `status --porcelain=v2 -z --branch`, `log` with a NUL-separated `--format`, `diff`/`show` unified text, `for-each-ref --format`.
Alternatives: `libgit2` (git2) or `gix`. Rejected: they do not run the distro's git, they ignore some hooks and credential helpers, and WSL repositories are slow or wrong through `\\wsl$`.

### D3. WSL path means the distro's git runner <!-- evergreen-candidate -->
`Runner` has two variants. `Windows` runs `git.exe -C <path>`. `Wsl { distro }` runs `wsl.exe -d <distro> --cd <linux-path> --exec git`. A pure function maps a path to a runner and a native path. It handles `\\wsl$\<d>\...` and `\\wsl.localhost\<d>\...`. Git never reads a WSL repository through the Windows share.
Alternative: always use Windows `git.exe` on the share. Rejected: very slow, and file mode and line-ending results differ.
A per-repository override of the git binary is out of scope for v1.

### D4. Stream the log, do not re-run it with `--skip`
One `git log --all --topo-order -z --format=...` process per open repository. A reader thread parses the pipe and sends batches of 500 commits to the UI. The reader stops reading when the UI has enough rows; the pipe back-pressure pauses git. The reader resumes when the user scrolls near the end. We kill the process when the repository closes or refs change.
Alternative: `--max-count` plus `--skip`. Rejected: git must walk all skipped commits again on every page, so each later page is slower.

### D5. Graph lanes are computed in `gitcore`, incrementally
A function takes the next batch of commits in topo order and the open-lane state, and returns lane data for each row plus the new state. It is pure, so tests use small fixed histories. The UI only draws the result.

### D6. Virtualized lists, GPUI paint in the visible window only
The graph and the file lists use a virtual list. The row height is fixed, so row index maps to y without measuring. The diff view is the `gpui-component` `Diff` (see D11), which is virtualized already.

### D7. Refresh without file watching on WSL
The status refreshes on window focus, after any action the app runs, and every 2 seconds while the window is focused. v1 does not watch the file system: change events on `\\wsl$` are not reliable.
Alternative: `notify` crate. Rejected for v1 for that reason. It can be added for Windows paths later.

### D8. Theme as one token module
`app::theme` holds the Console tokens as constants and maps them to the `gpui-component` theme. Views read tokens only from this module. A unit test checks the contrast ratios of every text/background pair (spec: ui-theme). Fonts (Host Grotesk for UI, Atkinson Hyperlegible Mono for code, both OFL) are bundled with the app.

### D9. Pin exact versions
`Cargo.toml` pins `gpui-pre` 0.3.12 (imported as `gpui`), `gpui-pre-platform` 0.3.12 (imported as `gpui_platform`), and `gpui-component` 0.10.0 with `=`. The app starts with `gpui_platform::application()`. The crate named `gpui` 0.2.2 on crates.io is not the one `gpui-component` uses. `Cargo.lock` is committed. We upgrade on purpose, one change at a time.

### D10. Test seams (tdd)
Test first for: (a) parsers for status, log, diff, refs, using fixture files; (b) path-to-runner mapping; (c) lane layout on fixed histories; (d) real temporary repositories for stage, unstage, commit, checkout through the Windows runner; (e) the contrast test; (f) benchmarks with a generated 100,000-commit repository, run for the Windows runner and, when a distro exists, for the WSL runner. We write no automated UI tests; we check views by hand.

### D11. Show diffs with the `gpui-component` `Diff`
`gpui_component::diff` is a read-only, virtualized patch view. `DiffFile::parse` reads the text of `git diff` or `git show`. `DiffState` holds the files. `Diff` draws line numbers, `+`/`-` markers, add/remove colors, word-level highlight, syntax color, binary and rename headers. A spike on 2026-11-20 showed all of this on a Rust patch that contains CJK text. We use unified mode only in v1. We write no diff parser and no diff list of our own.
Alternative: our own parser and virtual list. Rejected: it repeats working code (laziness-protocol, subtract-before-you-add).
The same component has split mode, annotations, and line decorations, which v2 can use for side-by-side diff and hunk staging.
Check later: scroll speed on a 50,000-line patch (task 3.4), and whether the colors fit the Console theme (task 6.1).

## Risks / Trade-offs

- [GPUI or gpui-component API changes] → Pin versions (D9). Keep UI code thin and logic in `gitcore` (D1).
- [GPUI on Windows has rough edges: IME, text input, high-DPI] → Build the first slice end to end early (task 1) to find them while the code is small. Test Traditional Chinese input in the commit message box.
- [`wsl.exe` start-up cost on each command] → Keep commands few and batched. Measure in the benchmark. If the cost is too high, keep one long-lived `wsl.exe` shell per distro (change later).
- [Performance target may fail on WSL] → The benchmark runs on both runners from the first slice, so a miss shows early.
- [Syntax highlighting cost on large diffs] → Highlight only visible lines. See Open Questions.

## Open Questions

- Syntax color comes from the `syntect` feature of `gpui-component`, through `Diff`. Its cost on a large patch is not measured yet (task 3.4).
- Chinese text typed as Unicode characters shows correctly in the input box. Typing through a real Windows IME (composition window, candidate list) is not tested. The user needs to try it once before task 4.4.
