# git-win-wsl-gui

A fast Git GUI for Windows, written in Rust. It works with repositories on Windows and inside WSL.

The app does not reimplement Git. It runs the real `git` for every action:

- A Windows path (`C:\code\repo`) runs `git.exe`.
- A WSL path (`\\wsl$\Ubuntu\home\me\repo` or `\\wsl.localhost\Ubuntu\home\me\repo`) runs `wsl.exe -d Ubuntu --cd /home/me/repo --exec git`. The repository uses the Git of its own distro.

The UI uses [gpui](https://crates.io/crates/gpui-pre) and [gpui-component](https://crates.io/crates/gpui-component).

## What v1 does

- Open a repository. Open several repositories in tabs.
- Show the multi-branch commit graph. The app reads history in pages, so a repository with 100,000 commits opens fast.
- Show a commit with its files and a unified diff.
- Show the working tree: staged and unstaged files, with a live diff.
- Stage and unstage a file. Stage all. Discard the changes of one file. Create a commit. Amend the last commit.
- List local and remote branches. Check out, create, and delete a branch.
- Fetch, pull, and push.

v1 does not do: hunk or line staging, side-by-side diff, rebase, stash, merge editor, light theme, macOS, Linux.

## Requirements

- Windows. The app is tested on Windows 11.
- `git` on Windows, in `PATH`. The app uses `git switch`, so Git 2.23 or newer.
- For repositories in WSL: WSL, and `git` installed inside each distro that holds a repository.
- Open only repositories you trust. Git runs the hooks and the settings of a repository (for example `core.fsmonitor`). This app runs Git, so it does the same.
- To build: Rust (edition 2024, a current stable toolchain).

## Install

Download `git-win-wsl-gui-<version>-windows-x64.zip` from the [Releases](../../releases) page. Unzip it and run `git-win-wsl-gui.exe`. The file is not code-signed, so Windows SmartScreen can show a warning. The `.sha256` file next to the zip holds its checksum.

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

The focused element has an accent-colored ring. Every v1 action works without a mouse, except the tab bar (see Tabs).

## Tabs

Each tab holds one repository. Press `+` to open a repository in a new tab. Click a tab to show it. Press `×` on a tab, or click it with the middle mouse button, to close it.

- A repository that is already open is not opened twice. The app shows its tab.
- A tab keeps its state while you look at another tab: the commits, the scroll position, the selection, and the commit message you typed.
- Only the tab you look at reads the Git status every 2 seconds. A tab in the background runs no Git command, except to finish the read of a repository you just opened.
- When you close a tab, the commit message you typed in it is lost. The app does not ask.
- The app saves the open tabs in `%LOCALAPPDATA%\git-win-wsl-gui\tabs.txt` and opens them again at the next start. A tab reads its repository when you first show it.

The tab bar works with the mouse only. Keyboard shortcuts for tabs are not available yet.

## Changes

- **Discard**: select a file in the Unstaged list, press `Discard`, then press `Discard` again to confirm. A changed or deleted file goes back to the content of the index (`git restore --worktree`). An untracked file is deleted from disk (`git clean -f`). Staged changes of the same file stay. A discard cannot be undone. `Discard` is grey for a staged file, a conflicted file, and a file you added with `git add -N`. The app has no "discard all".
- **Amend**: press `Amend` next to the commit button. If the message box is empty, it shows the message of the last commit. Change the message, stage more files if you want, and press `Amend commit`. The staged files go into the last commit. With nothing staged, only the message changes. `Amend` is grey on a branch with no commit. If the last commit is already pushed, the next push is refused, because the history changed. The app never forces a push.

## Branches

The branch panel is on the left of a repository.

- **Create**: type a name in the field and press `Create` (or `Enter`). The new branch starts at the current commit, and the app checks it out. Changes in your work tree stay. Git decides which names are valid, and the app shows its message when it refuses one.
- **Create at another place**: right-click a commit in the history, or a local or remote branch in the branch panel, and choose `Create branch here`. The panel then shows "Start: ..." above the name field. Type a name and press `Create`: the new branch starts there, and the app checks it out. `Clear` removes the start point. A start point stays after a failed `Create` and goes away after a successful one.
- **Delete**: select a local branch that is not the current branch, press `Delete`, then press `Delete` again to confirm. The confirmation has a box "Force (delete even if not merged)", off at the start. With the box off, the app runs `git branch -d`. Git refuses a branch that is not fully merged, and the app shows its message and keeps the confirmation open. With the box on, the app runs `git branch -D`, which drops the commits that no other branch holds. Delete removes only the local branch.
- The app does not delete remote branches.

## Remote

The buttons **Fetch**, **Pull**, and **Push** are at the top right of a repository.

- **Fetch** runs `git fetch --all --prune`. It also removes remote-tracking branches that the remote deleted.
- **Pull** runs `git pull --ff-only`. It moves the branch only when that is a fast-forward. When the branch and its upstream both have new commits, the app shows the message of Git and changes nothing. Fix that case in a terminal (`git merge` or `git rebase`).
- **Push** runs `git push`. A branch with no upstream goes to the remote `origin`, and the app sets that branch as its upstream. A push that the remote refuses shows the message of Git. The app never forces a push.
- `Pull ↓2` means the branch is 2 commits behind its upstream. `Push ↑1` means it is 1 commit ahead. The numbers change after a fetch.
- While a command runs, the three buttons are grey, and staging, commit, and checkout wait.
- A command never asks for a password at a terminal. It fails with the message of Git. A credential helper that has its own window (Git Credential Manager) still works. For SSH, load the key into `ssh-agent` first.

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

The project uses the Apache-2.0 license. See `LICENSE`.

## Project layout

- `crates/gitcore`: Git engine with no UI. It runs `git` and parses its output.
- `crates/app`: the gpui app.
- `openspec/`: specs, design, and tasks. `openspec/follow-ups.md` lists open work.
- `.github/workflows/`: `ci.yml` checks every push; `release.yml` builds and publishes a release when you push a tag such as `v0.1.0`.
- `docs/architecture/`: architecture notes.
