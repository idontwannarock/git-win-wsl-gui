## ADDED Requirements

### Requirement: Discard all changes
The system SHALL let the user discard the work-tree changes of all unstaged tracked files with one item `Discard all changes` in the right-click menu of the Unstaged group header. The system SHALL ask for confirmation, and the question SHALL show the number of files. The system SHALL discard only unstaged files whose content changed (modified, deleted, or type changed). The system SHALL keep untracked files, staged changes, conflicted files, and files with an added, renamed, or copied state. The system SHALL disable the item while a command runs and when no file can be discarded.

#### Scenario: Discard all
- **WHEN** the work tree has modified and deleted tracked files, the user right-clicks the Unstaged header, chooses `Discard all changes`, and confirms
- **THEN** every such file has the content of the index, and none stays in the unstaged group

#### Scenario: The question shows the count
- **WHEN** 3 files can be discarded and the user chooses `Discard all changes`
- **THEN** the question reads "Discard changes to 3 files?"

#### Scenario: Untracked files stay
- **WHEN** the work tree has an untracked file and the user discards all
- **THEN** the untracked file is still on disk and still in the unstaged group

#### Scenario: Staged changes stay
- **WHEN** a file has staged changes and more unstaged changes, and the user discards all
- **THEN** the file stays in the staged group with its staged content

#### Scenario: Conflicted files stay
- **WHEN** the work tree has a conflicted file and a modified file, and the user discards all
- **THEN** the modified file is restored and the conflicted file is unchanged

#### Scenario: Cancel
- **WHEN** the user chooses `Discard all changes` and then presses `Cancel`
- **THEN** no file changes and the question disappears

#### Scenario: Nothing to discard
- **WHEN** the unstaged group has no file that can be discarded, or a command runs
- **THEN** the item `Discard all changes` is disabled
