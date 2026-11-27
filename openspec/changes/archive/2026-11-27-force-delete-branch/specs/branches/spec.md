## MODIFIED Requirements

### Requirement: Delete branch
The system SHALL let the user delete a selected local branch that is not the current branch. The system SHALL ask for confirmation before it deletes. The system SHALL first delete only a branch that Git considers fully merged. When Git refuses, the system SHALL offer a second confirmation to force the delete.

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
- **THEN** the system shows the git error text, the branch stays, and the confirmation asks "Force delete `<name>`?" with `Force delete` and `Cancel`

#### Scenario: Force delete
- **WHEN** the user presses `Force delete`
- **THEN** the branch is gone from the list and from the repository

#### Scenario: Cancel the force step
- **WHEN** the user presses `Cancel` or selects another branch while the force question shows
- **THEN** the branch stays and the question disappears

#### Scenario: Force delete fails
- **WHEN** Git refuses the force delete
- **THEN** the system shows the git error text, the branch stays, and the question disappears
