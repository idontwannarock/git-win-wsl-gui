## MODIFIED Requirements

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
