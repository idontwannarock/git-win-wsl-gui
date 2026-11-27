## ADDED Requirements

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
