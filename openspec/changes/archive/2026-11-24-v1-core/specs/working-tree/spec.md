## ADDED Requirements

### Requirement: Working-tree status
The system SHALL list changed files in two groups: staged and unstaged. Untracked files SHALL appear in the unstaged group. Each file SHALL show its change kind (added, modified, deleted, renamed, untracked, conflicted).

#### Scenario: Mixed changes
- **WHEN** one file is staged and another file is modified but not staged
- **THEN** the first file is in the staged group and the second file is in the unstaged group

#### Scenario: Status refresh
- **WHEN** a file changes on disk
- **THEN** the system updates the lists within 2 seconds, or when the window gains focus

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
