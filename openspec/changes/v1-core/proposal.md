## Why

Fork is a fast, pleasant Git GUI, but it has no first-class WSL support. Glimpse has WSL support, but it is a Tauri/WebView app. This project builds a Fork-like Git GUI in Rust with GPUI, so it stays fast on large repositories, and it drives repositories in WSL through that distro's own `git`.

## What Changes

- Create a new Windows-only desktop app in Rust, built on `gpui` and `gpui-component`.
- Run the real `git` for every operation. Use `git.exe` for Windows paths. Use `wsl.exe -d <distro> --exec git` for `\\wsl$` paths. Never reimplement git, and never link `libgit2` or `gix`.
- Add v1 features: open a repository, multi-branch commit graph, unified diff, working-tree status, stage/unstage by file, commit, branch list and checkout.
- Apply the Hallmark design principles and the Hallmark "Console" dark theme tokens to the UI. Skip `modern-web-guidance` (web-only).
- Set a performance target on a 100,000-commit repository and verify it with benchmarks.
- Out of scope for v1: hunk or line staging, side-by-side diff, word-level diff, rebase, stash, merge editor, light theme, macOS, Linux.

## Capabilities

### New Capabilities
- `repo-access`: Pick the git runner (Windows or WSL distro) from the repository path, open a repository, and run git commands through that runner.
- `commit-graph`: Show the multi-branch commit graph. Read history in pages.
- `diff-view`: Show a unified diff of a commit or a working-tree file, with line numbers and syntax color.
- `working-tree`: Show status. Stage and unstage by file. Create a commit.
- `branches`: List local and remote branches. Check out a branch.
- `ui-theme`: Define the Console dark theme, the accessibility floor, and the performance targets for scrolling.

### Modified Capabilities

(none)

## Impact

- New Cargo workspace in `D:\ws\github\git-win-wsl-gui`. No existing code is affected.
- New dependencies: `gpui` 0.2.2 and `gpui-component` 0.10.0 (Apache-2.0). Pin both to exact versions. `gpui-component` 0.10.0 builds on `gpui-pre-*` snapshots, so its API may change between releases.
- Runtime dependency: `git` on Windows, and `git` inside each WSL distro that holds a repository.
