# branches Specification

## Purpose
List local and remote branches, check one out, create a branch, and delete a branch.
## Requirements
### Requirement: Branch list
The system SHALL list local branches and remote branches in a side panel. The current branch SHALL be marked.

#### Scenario: List
- **WHEN** a repository is open
- **THEN** the panel shows local branches and remote branches, with the current branch marked

### Requirement: Checkout
The system SHALL let the user check out a local branch by double-click or by a menu action. The system SHALL check out a remote branch by creating a tracking local branch.

#### Scenario: Checkout a local branch
- **WHEN** the user checks out a local branch and the work tree is clean
- **THEN** HEAD moves to that branch and the graph and status update

#### Scenario: Checkout fails
- **WHEN** git refuses the checkout (for example, local changes would be overwritten)
- **THEN** the system shows the git error text and HEAD does not change

#### Scenario: Remote branch
- **WHEN** the user checks out `origin/feature`
- **THEN** the system creates local `feature` that tracks `origin/feature` and checks it out

### Requirement: Create branch
The system SHALL let the user create a local branch from a name typed in the branch panel. The new branch SHALL start at the current commit and SHALL be checked out. The system SHALL NOT create a branch when the name is empty, or when Git refuses the name.

#### Scenario: Create
- **WHEN** the user types `topic` and presses `Create`
- **THEN** branch `topic` exists at the current commit, HEAD is on `topic`, the name field is empty, and the graph, branch list, and status update

#### Scenario: Work-tree changes stay
- **WHEN** the work tree has changes and the user creates a branch
- **THEN** the changes are still in the work tree on the new branch

#### Scenario: Empty name
- **WHEN** the name field is empty or holds only spaces
- **THEN** `Create` is disabled

#### Scenario: Create fails
- **WHEN** Git refuses the name (the branch exists, or the name is not valid)
- **THEN** the system shows the git error text, no branch is created, and HEAD does not change

#### Scenario: Name that looks like an option
- **WHEN** the user types `-x`
- **THEN** the system shows an error and runs no branch command with that name as an option

### Requirement: Delete branch
The system SHALL let the user delete a selected local branch that is not the current branch. The system SHALL ask for confirmation before it deletes. The system SHALL delete only a branch that Git considers fully merged.

#### Scenario: Delete a merged branch
- **WHEN** the user selects a merged local branch, presses `Delete`, and confirms
- **THEN** the branch is gone from the list and from the repository

#### Scenario: Cancel
- **WHEN** the user presses `Delete` and then `Cancel`, or selects another branch
- **THEN** no branch is deleted and the confirmation disappears

#### Scenario: Not deletable
- **WHEN** the selection is the current branch, a remote branch, or nothing
- **THEN** `Delete` is disabled

#### Scenario: Unmerged branch
- **WHEN** the user confirms the delete of a branch that is not fully merged
- **THEN** the system shows the git error text and the branch stays

