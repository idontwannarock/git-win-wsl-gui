## ADDED Requirements

### Requirement: Runner is chosen from the repository path
The system SHALL run every git command through one runner. The system SHALL choose the runner from the repository path. A Windows path SHALL use `git.exe`. A `\\wsl$\<distro>\...` or `\\wsl.localhost\<distro>\...` path SHALL use `wsl.exe -d <distro> --cd <linux-path> --exec git`.

#### Scenario: Windows path
- **WHEN** the user opens `C:\dev\repo`
- **THEN** the system runs `git.exe` with `C:\dev\repo` as the working directory

#### Scenario: WSL path
- **WHEN** the user opens `\\wsl$\Ubuntu\home\me\repo`
- **THEN** the system runs `wsl.exe -d Ubuntu --cd /home/me/repo --exec git`

#### Scenario: WSL localhost path
- **WHEN** the user opens `\\wsl.localhost\Ubuntu\home\me\repo`
- **THEN** the system selects the same runner as for the `\\wsl$` form

### Requirement: System never reimplements git
The system SHALL NOT link `libgit2` or `gix`. The system SHALL read repository data only from the output of the real `git` binary.

#### Scenario: Repository data comes from git output
- **WHEN** the system shows commits, status, or branches
- **THEN** each value comes from parsed output of a `git` command

### Requirement: User can open a repository
The system SHALL let the user open a repository from a folder path. The system SHALL reject a folder that is not a git work tree and SHALL show the reason.

#### Scenario: Valid repository
- **WHEN** the user picks a folder inside a git work tree
- **THEN** the system opens the repository at its top-level directory

#### Scenario: Not a repository
- **WHEN** the user picks a folder that is not in a git work tree
- **THEN** the system shows an error and opens nothing

#### Scenario: Missing git
- **WHEN** the selected runner cannot start (no `git.exe`, or no `git` in the distro)
- **THEN** the system shows which binary or distro is missing

### Requirement: Commands never block the UI
The system SHALL run git commands off the UI thread. The UI SHALL stay responsive while a command runs.

#### Scenario: Slow command
- **WHEN** a git command takes more than 100 ms
- **THEN** the UI keeps handling input and shows a busy state for that panel
