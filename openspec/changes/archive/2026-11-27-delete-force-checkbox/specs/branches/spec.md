## MODIFIED Requirements

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
