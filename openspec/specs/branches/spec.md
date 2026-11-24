# branches Specification

## Purpose
List local and remote branches and check one out.

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
