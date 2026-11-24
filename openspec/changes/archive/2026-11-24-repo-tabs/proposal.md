## Why

The app shows one repository at a time. A user who works in several repositories must reopen each one with the folder picker, and each switch loses the history position, the selected commit, and the typed commit message. Fork, the feature reference of this project, solves this with tabs.

## What Changes

- Add a tab bar above the main view. One tab holds one repository.
- Opening a repository (button, `+`, or the command-line path) opens a new tab and activates it. When that repository is already open, the app activates its tab and opens no second tab.
- The tab bar works with the mouse only: click a tab to activate it, `×` or middle click to close it, `+` to open a folder picker.
- A tab shows the folder name. When two tabs have the same name, both add the parent folder. The full path shows on hover.
- Each tab keeps its own state in memory (history, scroll position, selection, branch list, working-tree state, commit message). Switching tabs shows the state at once.
- Only the active tab runs the 2-second `git status` poll. A tab runs one extra refresh when the user activates it.
- Closing a tab stops its git processes and its background tasks, and drops its state. The typed commit message is lost without a prompt. Closing the last tab shows the empty screen.
- The app saves the list of tabs and the active tab. At start, the app restores them. A restored tab reads its repository when the user first activates it. A path that no longer opens shows its error in its tab and does not stop the other tabs.
- Fix follow-up 8.8: each tab owns the tasks that open it and reload its history. A late result never reaches another repository.
- Not in this change: keyboard shortcuts for tabs, drag to reorder tabs, a "has changes" mark on background tabs.

## Capabilities

### New Capabilities
- `repo-tabs`: the tab bar, opening and closing tabs, per-tab state, which tab polls, and restoring tabs at start.

### Modified Capabilities
- `repo-access`: "User can open a repository" now opens the repository in a tab and detects a repository that is already open.
- `working-tree`: the periodic status refresh runs only for the active tab.

## Impact

- `crates/app/src/home.rs`: the `Home` view splits in two. A new shell view owns the tab list and the tab bar. The per-repository view keeps today's logic.
- `crates/app/src/main.rs`: start-up restores tabs. The command-line path opens as a tab.
- `crates/gitcore`: a pure key function that tells whether two locations are the same repository.
- A new file `%LOCALAPPDATA%\git-win-wsl-gui\tabs.txt` holds the saved tabs.
- `README.md`: a Tabs section. The "every action works without a mouse" sentence gets an exception for the tab bar.
- `openspec/follow-ups.md`: close 8.8. Add keyboard shortcuts for tabs, drag to reorder, and a change mark on background tabs.
- No new dependency.
