# git-win-wsl-gui

A fast Git GUI for Windows, written in Rust. It works with repositories on Windows and inside WSL.

The app does not reimplement Git. It runs the real `git` for every action:

- A Windows path (`C:\code\repo`) runs `git.exe`.
- A WSL path (`\\wsl$\Ubuntu\home\me\repo` or `\\wsl.localhost\Ubuntu\home\me\repo`) runs `wsl.exe -d Ubuntu --cd /home/me/repo --exec git`. The repository uses the Git of its own distro.

The UI uses [gpui](https://crates.io/crates/gpui-pre) and [gpui-component](https://crates.io/crates/gpui-component).

## What v1 does

- Open a repository.
- Show the multi-branch commit graph. The app reads history in pages, so a repository with 100,000 commits opens fast.
- Show a commit with its files and a unified diff.
- Show the working tree: staged and unstaged files, with a live diff.
- Stage and unstage a file. Stage all. Create a commit.
- List local and remote branches. Check out a branch.

v1 does not do: hunk or line staging, side-by-side diff, rebase, stash, merge editor, light theme, macOS, Linux.

## Requirements

- Windows. The app is tested on Windows 11.
- `git` on Windows, in `PATH`. The app uses `git switch`, so Git 2.23 or newer.
- For repositories in WSL: WSL, and `git` installed inside each distro that holds a repository.
- Open only repositories you trust. Git runs the hooks and the settings of a repository (for example `core.fsmonitor`). This app runs Git, so it does the same.
- To build: Rust (edition 2024, a current stable toolchain).

## Build and run

```
cargo build --release
target\release\app.exe
target\release\app.exe C:\code\repo
```

The optional argument is a repository to open at start. Without it, press **Open repository...**.

To check the code:

```
cargo test
cargo clippy -- -D warnings
cargo build --release
```

The WSL tests use the distro `Ubuntu`. Set `GITCORE_WSL_DISTRO` to use another distro. Without a usable distro, the WSL tests do nothing and still pass. A passing `cargo test` on such a machine does not test the WSL path.

## Keyboard

| Key | Action |
| --- | --- |
| Tab, Shift+Tab | Move focus in visual order |
| Up, Down | Select the previous or next row in a focused list |
| Enter | Check out the selected branch (branch list) |
| Space | Stage or unstage the selected file (changes list) |

The focused element has an accent-colored ring. Every v1 action works without a mouse.

## Large repositories

Run this once in a big repository:

```
git maintenance start
```

It keeps a commit-graph file up to date in the background. Git then reads history much faster, and so does this app. The app does not ask Git to write a commit-graph file into your repository.

## Debug log

The app writes a log to `%LOCALAPPDATA%\git-win-wsl-gui\app.log`. The log lists:

- each `git` command with its exit code, its run time, and the start of its error text,
- errors from the commit history reader,
- panics, with a backtrace.

When the file grows past 10 MB, the app renames it to `app.log.old` and starts a new file. The log can hold parts of your commit messages and paths.

## Fonts and license

The app embeds two fonts under the SIL Open Font License 1.1. The files and licenses are in `crates/app/assets/fonts/`:

- Host Grotesk (UI text)
- Atkinson Hyperlegible Mono (hashes and code)

The project declares the Apache-2.0 license in `Cargo.toml`. The repository has no `LICENSE` file yet.

## Project layout

- `crates/gitcore`: Git engine with no UI. It runs `git` and parses its output.
- `crates/app`: the gpui app.
- `openspec/`: specs, design, and tasks.
- `docs/architecture/`: architecture notes.
