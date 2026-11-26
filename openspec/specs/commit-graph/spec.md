# commit-graph Specification

## Purpose
Show the multi-branch commit graph, read in pages, and show a selected commit.

## Requirements

### Requirement: Multi-branch commit graph
The system SHALL show the commits of all local branches, remote branches, and tags in one graph. Each row SHALL show the graph lanes, the subject, the author, the date, the short hash, and the ref labels.

#### Scenario: Two branches diverge
- **WHEN** two branches have diverged from a common commit
- **THEN** the graph shows two lanes that join at the common commit

#### Scenario: Merge commit
- **WHEN** a commit has two parents
- **THEN** the graph draws a line from the commit to each parent

### Requirement: The graph column has a fixed width
The system SHALL give the graph column the width of 8 lanes, whatever the number of open lanes. When a row has more than 8 lanes, the system SHALL draw the first 6, draw no line or dot past them, and show `+N` at the right end of the column, where N is the number of open lanes of that row that it did not draw. The system SHALL NOT draw a line that has one end in a lane it did not draw.

#### Scenario: Many branches open at once
- **WHEN** a row has 12 open lanes
- **THEN** the column keeps the width of 8 lanes, the row draws lanes 1 to 6, and the row shows `+6`

#### Scenario: Eight lanes or fewer
- **WHEN** a row has 8 open lanes or fewer
- **THEN** the row draws every lane and shows no marker

### Requirement: History is read in pages
The system SHALL read history from one streamed `git log` process and hand it to the UI in pages. The system SHALL NOT parse or draw the whole history before it shows the first rows. The system SHALL load the next page when the user scrolls near the end of the loaded rows.

#### Scenario: Large repository
- **WHEN** the user opens a repository with 100,000 commits
- **THEN** the first page of rows is visible within 200 ms of the command result and the rest loads on scroll

### Requirement: Graph scrolls at 60 fps
The system SHALL render only the visible rows. Scrolling the graph SHALL keep 60 fps on a 100,000-commit repository, for both Windows and WSL repositories.

#### Scenario: Fast scroll
- **WHEN** the user scrolls the graph quickly through 10,000 loaded rows
- **THEN** at least 99 percent of the frames in the benchmark take 20 ms or less, and no frame takes 33 ms or more (no dropped frame on a 60 Hz screen)

### Requirement: User can select a commit
The system SHALL let the user select a commit. The system SHALL show the commit message, the author, and the list of changed files for the selected commit.

#### Scenario: Select a commit
- **WHEN** the user selects a commit row
- **THEN** the detail panel shows its full message and its changed files
