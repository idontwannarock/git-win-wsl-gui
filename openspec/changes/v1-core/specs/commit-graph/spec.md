## ADDED Requirements

### Requirement: Multi-branch commit graph
The system SHALL show the commits of all local branches, remote branches, and tags in one graph. Each row SHALL show the graph lanes, the subject, the author, the date, the short hash, and the ref labels.

#### Scenario: Two branches diverge
- **WHEN** two branches have diverged from a common commit
- **THEN** the graph shows two lanes that join at the common commit

#### Scenario: Merge commit
- **WHEN** a commit has two parents
- **THEN** the graph draws a line from the commit to each parent

### Requirement: History is read in pages
The system SHALL read history with paged `git log` calls. The system SHALL NOT read the whole history before it shows the first rows. The system SHALL load the next page when the user scrolls near the end of the loaded rows.

#### Scenario: Large repository
- **WHEN** the user opens a repository with 100,000 commits
- **THEN** the first page of rows is visible within 200 ms of the command result and the rest loads on scroll

### Requirement: Graph scrolls at 60 fps
The system SHALL render only the visible rows. Scrolling the graph SHALL keep 60 fps on a 100,000-commit repository, for both Windows and WSL repositories.

#### Scenario: Fast scroll
- **WHEN** the user scrolls the graph quickly through 10,000 loaded rows
- **THEN** the frame time stays at or below 16.6 ms at the 95th percentile in the benchmark

### Requirement: User can select a commit
The system SHALL let the user select a commit. The system SHALL show the commit message, the author, and the list of changed files for the selected commit.

#### Scenario: Select a commit
- **WHEN** the user selects a commit row
- **THEN** the detail panel shows its full message and its changed files
