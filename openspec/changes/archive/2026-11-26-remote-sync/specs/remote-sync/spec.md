## ADDED Requirements

### Requirement: Fetch
The system SHALL fetch every remote of the repository, and SHALL remove remote-tracking branches that no longer exist on the remote. After a successful fetch, the system SHALL reload the branch list and the history.

#### Scenario: A new commit on the remote
- **WHEN** the remote has a new commit and the user presses `Fetch`
- **THEN** the history shows the commit under its remote branch, and the working tree does not change

#### Scenario: A branch deleted on the remote
- **WHEN** a branch was deleted on the remote and the user presses `Fetch`
- **THEN** the branch list no longer shows its remote-tracking branch

### Requirement: Pull
The system SHALL update the current branch from its upstream only when the update is a fast-forward. When it is not a fast-forward, the system SHALL change no branch and no file, and SHALL show the message of Git.

#### Scenario: Fast-forward pull
- **WHEN** the upstream is ahead of the current branch and the user presses `Pull`
- **THEN** the current branch moves to the upstream commit, the working tree shows the new files, and the history reloads

#### Scenario: Diverged branches
- **WHEN** the current branch and its upstream both have new commits and the user presses `Pull`
- **THEN** the branch and the files stay as they were, and the app shows the message of Git

### Requirement: Push
The system SHALL push the current branch to its upstream. When the current branch has no upstream, the system SHALL push it to the remote `origin` and SHALL set that remote branch as its upstream. When the remote refuses the push, the system SHALL show the message of Git.

#### Scenario: Push to the upstream
- **WHEN** the current branch is ahead of its upstream and the user presses `Push`
- **THEN** the remote branch moves to the current commit

#### Scenario: A branch with no upstream
- **WHEN** the current branch has no upstream and the user presses `Push`
- **THEN** the remote `origin` gets the branch, and the branch tracks it

#### Scenario: The remote refuses
- **WHEN** the remote has commits that the current branch lacks and the user presses `Push`
- **THEN** the remote does not change, and the app shows the message of Git

### Requirement: Ahead and behind counts
The system SHALL show the number of commits the current branch is behind its upstream on the `Pull` button, and the number of commits it is ahead on the `Push` button. When a number is 0, or the branch has no upstream, the button shows no number.

#### Scenario: Ahead and behind
- **WHEN** the current branch is 2 commits ahead and 1 commit behind its upstream
- **THEN** the buttons read `Pull ↓1` and `Push ↑2`

#### Scenario: No upstream
- **WHEN** the current branch has no upstream
- **THEN** the buttons read `Pull` and `Push`

### Requirement: Remote commands show a busy state and never wait at a terminal
While a remote command runs, the system SHALL disable the three remote buttons, SHALL show which command runs, and SHALL NOT start staging, commit, or checkout. The system SHALL NOT let a remote command wait for input at a terminal. When Git needs a password it cannot get, the command SHALL fail and the app SHALL show the message of Git.

#### Scenario: A command runs
- **WHEN** the user presses `Fetch` and the remote answers slowly
- **THEN** the buttons are disabled, a label shows that a fetch runs, and the window stays responsive

#### Scenario: A password is missing
- **WHEN** a remote needs a password and no credential helper can supply it
- **THEN** the command ends within a few seconds, and the app shows the message of Git
