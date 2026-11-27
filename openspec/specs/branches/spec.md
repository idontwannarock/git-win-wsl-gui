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
The system SHALL let the user create a local branch from a name typed in the branch panel. The new branch SHALL start at the start point that the user chose, or at the current commit when none is chosen, and SHALL be checked out. The user SHALL choose a start point with the item `Create branch here` in the right-click menu of a commit row or of a branch row. The system SHALL NOT create a branch when the name is empty, or when Git refuses the name or the start point.

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

#### Scenario: Start at a commit
- **WHEN** the user right-clicks a commit row, chooses `Create branch here`, types `topic`, and presses `Create`
- **THEN** the branch panel showed "Start: `<short commit id>`" before the press, branch `topic` exists at that commit, and HEAD is on `topic`

#### Scenario: Start at a branch
- **WHEN** the user right-clicks a local or remote branch row, chooses `Create branch here`, types `topic`, and presses `Create`
- **THEN** branch `topic` exists at the commit of that branch, and HEAD is on `topic`

#### Scenario: Clear the start point
- **WHEN** a start point is set and the user presses its clear button
- **THEN** the "Start:" line disappears and `Create` starts at the current commit

#### Scenario: Start point after create
- **WHEN** a create with a start point succeeds
- **THEN** the "Start:" line disappears
- **WHEN** a create with a start point fails
- **THEN** the "Start:" line stays

#### Scenario: Newer start point stays
- **WHEN** the user chooses another start point while a create runs, and the create succeeds
- **THEN** the new start point stays, because only the start point that the create used is cleared

#### Scenario: No tracking
- **WHEN** the user creates a branch at a remote branch
- **THEN** the new branch has no upstream, and a later `Push` sets it with `--set-upstream origin HEAD`

#### Scenario: Branch name that looks like an option
- **WHEN** the user chooses `Create branch here` on a branch whose name begins with `-`
- **THEN** the new branch starts at that branch, and Git reads the name as a branch and not as an option

### Requirement: Delete branch
The system SHALL let the user delete a selected local branch that is not the current branch. The system SHALL ask for confirmation before it deletes. The confirmation SHALL have a check box "Force (delete even if not merged)", off at the start. When the box is off, the system SHALL delete only a branch that Git considers fully merged. When the box is on, the system SHALL delete the branch even if it is not merged.

#### Scenario: Delete a merged branch
- **WHEN** the user selects a merged local branch, presses `Delete`, and confirms with the box off
- **THEN** the branch is gone from the list and from the repository

#### Scenario: Cancel
- **WHEN** the user presses `Delete` and then `Cancel`, or selects another branch
- **THEN** no branch is deleted and the confirmation disappears

#### Scenario: Not deletable
- **WHEN** the selection is the current branch, a remote branch, or nothing
- **THEN** `Delete` is disabled

#### Scenario: Unmerged branch, box off
- **WHEN** the user confirms the delete of a branch that is not fully merged with the box off
- **THEN** the system shows the git error text, the branch stays, and the confirmation stays open with the box off

#### Scenario: Force delete
- **WHEN** the user ticks the box and confirms
- **THEN** the branch is gone from the list and from the repository, even if it was not merged

#### Scenario: Force delete fails
- **WHEN** Git refuses the delete with the box on
- **THEN** the system shows the git error text, the branch stays, and the confirmation disappears

