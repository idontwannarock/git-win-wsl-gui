# repo-tabs Specification

## Purpose
Show several repositories as tabs, keep the state of each tab, and restore the tabs at the next start.

## Requirements

### Requirement: Tab bar shows one tab per open repository
The system SHALL show a tab bar above the main view. The tab bar SHALL hold one tab for each open repository, in the order the user opened them. The tab bar SHALL mark the active tab. The tab bar SHALL end with a `+` button.

#### Scenario: Two repositories open
- **WHEN** the user opens two different repositories
- **THEN** the tab bar shows two tabs and the second tab is active

#### Scenario: Plus button
- **WHEN** the user presses `+`
- **THEN** the system shows the folder picker and opens the picked repository as in "User can open a repository"

### Requirement: Opening a repository opens or activates a tab
The system SHALL open a repository in a new tab and activate that tab. When the repository is already open in a tab, the system SHALL activate that tab and SHALL NOT open a second tab. The system SHALL treat two locations as the same repository when they use the same runner and their paths are equal after normalization. Windows paths SHALL ignore case, drive-letter case, and a trailing separator. WSL paths SHALL ignore the distro name case and a trailing `/`, and SHALL keep the case of the Linux path. `\\wsl$\<distro>\...` and `\\wsl.localhost\<distro>\...` SHALL be the same repository.

#### Scenario: Same repository again
- **WHEN** `C:\code\repo` is open and the user opens `c:\code\repo\`
- **THEN** the system activates the existing tab and the tab count does not change

#### Scenario: Sub-folder of an open repository
- **WHEN** `C:\code\repo` is open and the user opens `C:\code\repo\src`
- **THEN** the system resolves the top-level directory, finds the repository already open, closes the new tab, and activates the existing tab

#### Scenario: Two WSL forms
- **WHEN** `\\wsl$\Ubuntu\home\me\repo` is open and the user opens `\\wsl.localhost\Ubuntu\home\me\repo`
- **THEN** the system activates the existing tab

#### Scenario: Linux path case
- **WHEN** `\\wsl$\Ubuntu\home\me\Repo` is open and the user opens `\\wsl$\Ubuntu\home\me\repo`
- **THEN** the system opens a second tab

#### Scenario: Command-line path
- **WHEN** the user starts the app with a repository path while tabs are restored
- **THEN** the system opens or activates the tab for that path and makes it active

#### Scenario: Open again after a failed open
- **WHEN** a tab shows an error for `C:\code\new` and the user opens `C:\code\new` again
- **THEN** the system activates that tab and reads the repository again in it

### Requirement: Tab title
The system SHALL title each tab with the last folder name of the repository path. When two tabs would have the same title, the system SHALL add the parent folder name to both titles. The system SHALL show the full path when the pointer rests on a tab.

#### Scenario: Unique name
- **WHEN** the only open tab is `C:\code\repo`
- **THEN** the title is `repo`

#### Scenario: Same folder name
- **WHEN** `C:\a\app` and `\\wsl$\Ubuntu\home\me\app` are open
- **THEN** the titles are `app · a` and `app · me`

### Requirement: Close a tab
The system SHALL close a tab when the user presses its `×` button or presses the middle mouse button on it. Closing SHALL stop the git processes and background tasks of that tab and SHALL drop its state. The system SHALL NOT ask for confirmation. When the closed tab was active, the system SHALL activate the tab to its right, or the tab to its left when none is on the right. When the last tab closes, the system SHALL show the empty screen with the "Open repository..." button.

#### Scenario: Close the active tab
- **WHEN** tabs A, B, and C are open, B is active, and the user closes B
- **THEN** C becomes active

#### Scenario: Close the last tab of the list
- **WHEN** tabs A and B are open, B is active, and the user closes B
- **THEN** A becomes active

#### Scenario: Close the only tab
- **WHEN** one tab is open and the user closes it
- **THEN** the system shows the empty screen

#### Scenario: Close a background tab
- **WHEN** tabs A and B are open, B is active, and the user closes A
- **THEN** B stays active and keeps the keyboard focus

#### Scenario: Running read stops
- **WHEN** the user closes a tab while it still reads history
- **THEN** its `git log` process ends and no later result reaches another tab

### Requirement: Each tab keeps its own state
The system SHALL keep for each tab: the loaded commits, the scroll position, the selected commit and file, the branch list, the working-tree lists and selection, the History or Changes mode, and the typed commit message. Activating a tab SHALL show that state without a new read of the history.

#### Scenario: Switch away and back
- **WHEN** the user selects a commit in tab A, switches to tab B, and switches back to tab A
- **THEN** tab A shows the same commit selected at the same scroll position

#### Scenario: Message survives a switch
- **WHEN** the user types a commit message in tab A and switches to tab B and back
- **THEN** the message in tab A is unchanged

### Requirement: A task result reaches only its own tab
The system SHALL deliver the result of every background task to the tab that started it. A result for a closed tab SHALL be dropped.

#### Scenario: Slow open then switch
- **WHEN** the user opens a large repository, switches to another tab before it finishes, and the read ends
- **THEN** the other tab does not change and the large repository shows its history in its own tab

### Requirement: Only the active tab runs git in the background
The system SHALL run the periodic status refresh for the active tab only. The system SHALL run one status refresh when a tab becomes active. A background tab SHALL NOT poll and SHALL NOT start a read on its own. A background tab MAY finish the read of an open that the user started.

#### Scenario: Background tab is quiet
- **WHEN** tab A is active and tab B is in the background for 10 seconds
- **THEN** the debug log shows `git status` commands for tab A and none for tab B

#### Scenario: Refresh on activation
- **WHEN** a file changed in the repository of tab B while it was in the background and the user activates tab B
- **THEN** tab B shows the changed file after one refresh

### Requirement: Tabs are restored at start
The system SHALL save the list of open repositories and the active tab in `%LOCALAPPDATA%\git-win-wsl-gui\tabs.txt` after each open, close, or activation. At start, the system SHALL restore the tabs in the saved order and activate the saved active tab. A restored tab SHALL read its repository when the user first activates it. A restored tab that cannot open SHALL show its error in that tab and SHALL NOT stop the other tabs. The system SHALL treat a missing, empty, or damaged file as "no saved tabs".

#### Scenario: Restart
- **WHEN** three tabs are open, the second is active, and the user restarts the app
- **THEN** three tabs appear, the second is active, and only the second has read its repository

#### Scenario: Saved path is the top-level directory
- **WHEN** the user opens `C:\code\repo\src`, restarts the app, and then opens `C:\code\repo`
- **THEN** the system activates the restored tab and opens no second tab

#### Scenario: Folder removed
- **WHEN** a saved path no longer exists
- **THEN** its tab shows the error and the other tabs work

#### Scenario: Damaged file
- **WHEN** `tabs.txt` holds text that is not a valid tab list
- **THEN** the app starts with no tabs and does not crash
