# diff-view Specification

## Purpose
Show unified diffs of commit files and working tree files.

## Requirements

### Requirement: Unified diff
The system SHALL show a diff as unified text. Each line SHALL show the old and new line numbers. Added lines SHALL use the `ok` color. Removed lines SHALL use the `bad` color. Color SHALL NOT be the only signal: each line SHALL also show a `+` or `-` marker.

#### Scenario: Modified file
- **WHEN** the user selects a modified file
- **THEN** the system shows hunks with line numbers, `+`/`-` markers, and colors

#### Scenario: Binary file
- **WHEN** the selected file is binary
- **THEN** the system shows "Binary file" and no hunk text

#### Scenario: Renamed file
- **WHEN** the selected file was renamed
- **THEN** the system shows the old and new path

### Requirement: Diff source
The system SHALL show the diff of a commit file, of an unstaged working-tree file, or of a staged file. The system SHALL get the diff from `git diff` or `git show` output.

#### Scenario: Staged vs unstaged
- **WHEN** the user selects a file in the staged list
- **THEN** the system shows the diff between HEAD and the index
- **WHEN** the user selects a file in the unstaged list
- **THEN** the system shows the diff between the index and the work tree

### Requirement: Syntax color
The system SHALL color source code in the diff by language, based on the file extension. The system SHALL show plain text for an unknown extension.

#### Scenario: Rust file
- **WHEN** the user opens the diff of a `.rs` file
- **THEN** keywords and strings use syntax colors

### Requirement: Diff scrolls at 60 fps
The system SHALL render only the visible diff lines. A diff of 50,000 lines SHALL scroll at 60 fps.

#### Scenario: Large diff
- **WHEN** the user scrolls a 50,000-line diff
- **THEN** at least 99 percent of the frames in the benchmark take 20 ms or less, and no frame takes 33 ms or more (no dropped frame on a 60 Hz screen)
