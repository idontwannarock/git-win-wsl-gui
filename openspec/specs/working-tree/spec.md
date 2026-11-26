# working-tree Specification

## Purpose
Show staged and unstaged files, stage and unstage them, discard the changes of a file, create a commit, and amend the last commit.
## Requirements
### Requirement: Working-tree status
The system SHALL list changed files in two groups: staged and unstaged. Untracked files SHALL appear in the unstaged group. Each file SHALL show its change kind (added, modified, deleted, renamed, untracked, conflicted). The periodic refresh SHALL run for the active tab only.

#### Scenario: Mixed changes
- **WHEN** one file is staged and another file is modified but not staged
- **THEN** the first file is in the staged group and the second file is in the unstaged group

#### Scenario: Status refresh
- **WHEN** a file changes on disk in the repository of the active tab
- **THEN** the system updates the lists within 2 seconds, or when the window gains focus

#### Scenario: Background tab
- **WHEN** a file changes on disk in the repository of a background tab
- **THEN** the system updates that tab when the user activates it

### Requirement: Stage and unstage by file
The system SHALL let the user stage and unstage a whole file. The system SHALL NOT offer hunk or line staging in v1.

#### Scenario: Stage a file
- **WHEN** the user stages an unstaged file
- **THEN** the file moves to the staged group

#### Scenario: Unstage a file
- **WHEN** the user unstages a staged file
- **THEN** the file moves to the unstaged group

#### Scenario: Stage all
- **WHEN** the user chooses "Stage all"
- **THEN** every unstaged file becomes staged

### Requirement: Commit
The system SHALL create a commit from the staged files and a message with `git commit`. The system SHALL disable the commit action when the message is empty or nothing is staged. The system SHALL apply the user's git config, hooks, and signing settings unchanged.

#### Scenario: Successful commit
- **WHEN** the user enters a message and commits
- **THEN** a new commit appears at the top of the graph and the staged group is empty

#### Scenario: Hook rejects the commit
- **WHEN** a pre-commit hook fails
- **THEN** the system shows the hook output and keeps the message and the staged files

#### Scenario: Empty message
- **WHEN** the message is empty
- **THEN** the commit action is disabled

### Requirement: Discard file changes
The system SHALL let the user discard the work-tree changes of one selected unstaged file. The system SHALL ask for confirmation before it discards. The system SHALL keep the staged changes of that file. The system SHALL NOT offer discard for a staged file, a conflicted file, or an unstaged file with an added, renamed, or copied state.

#### Scenario: Discard a modified file
- **WHEN** the user selects a modified unstaged file, presses `Discard`, and confirms
- **THEN** the file has the content of the index, and it leaves the unstaged group

#### Scenario: Discard a deleted file
- **WHEN** the user discards a file that was deleted in the work tree
- **THEN** the file is back on disk

#### Scenario: Discard an untracked file
- **WHEN** the user discards an untracked file
- **THEN** the file is removed from disk and leaves the unstaged group

#### Scenario: Staged changes stay
- **WHEN** a file has staged changes and more unstaged changes, and the user discards the unstaged changes
- **THEN** the file stays in the staged group with its staged content

#### Scenario: Cancel
- **WHEN** the user presses `Discard` and then `Cancel`, or selects another file
- **THEN** the file is unchanged and the confirmation disappears

#### Scenario: Not discardable
- **WHEN** the selection is a staged file, a conflicted file, or nothing
- **THEN** `Discard` is disabled

### Requirement: Amend the last commit
The system SHALL let the user amend the last commit with the staged files and a message, with `git commit --amend`. The system SHALL fill an empty message box with the message of the last commit when the user turns amend on. The system SHALL disable amend when the branch has no commit. The system SHALL apply the user's git config, hooks, and signing settings unchanged.

#### Scenario: Amend with staged files
- **WHEN** the user turns amend on, stages a file, and amends
- **THEN** the last commit holds the file and the message, the number of commits does not change, and the staged group is empty

#### Scenario: Amend the message only
- **WHEN** the user turns amend on, changes the message, and amends with nothing staged
- **THEN** the last commit has the new message and the same files

#### Scenario: Message is filled
- **WHEN** the message box is empty and the user turns amend on
- **THEN** the box shows the message of the last commit

#### Scenario: Typed message is kept
- **WHEN** the message box has text and the user turns amend on
- **THEN** the text stays

#### Scenario: No commit yet
- **WHEN** the branch has no commit
- **THEN** the amend toggle is disabled

#### Scenario: Hook rejects the amend
- **WHEN** a hook fails
- **THEN** the system shows the hook output, and keeps the message, the staged files, and the last commit

