## MODIFIED Requirements

### Requirement: User can open a repository
The system SHALL let the user open a repository from a folder path. The system SHALL open the repository in a tab, as described in the `repo-tabs` capability. The system SHALL reject a folder that is not a git work tree and SHALL show the reason in that tab.

#### Scenario: Valid repository
- **WHEN** the user picks a folder inside a git work tree
- **THEN** the system opens the repository at its top-level directory in a tab

#### Scenario: Not a repository
- **WHEN** the user picks a folder that is not in a git work tree
- **THEN** the system shows an error in the new tab and keeps the other tabs unchanged

#### Scenario: Missing git
- **WHEN** the selected runner cannot start (no `git.exe`, or no `git` in the distro)
- **THEN** the system shows which binary or distro is missing
