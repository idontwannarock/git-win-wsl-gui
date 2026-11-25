## Context

Today `Home` (`crates/app/src/home.rs`) is one gpui view that owns one `Screen`: `Empty`, `Loading`, `Loaded(GraphState)`, or `Failed`. `GraphState` holds all state of one repository. The methods in `home.rs`, `working.rs`, and `branches.rs` reach that state through `self.graph()` and `self.graph_mut()`. `Home` also owns the commit-list scroll handle, the 2-second poll task, and the window-activation observer.

Two facts shape the design:
- A task that finishes later calls `self.graph_mut()`. That method means "the repository on screen now". With tabs, a result for tab A can land in tab B. This is follow-up 8.8, and it grows with tabs.
- `open_repository` resolves a picked folder to the top-level directory of its work tree. The app knows the real repository only after the first read.

Fork is the reference. A web search on 2026-11-24 found no list of Fork's tab shortcuts. The tab behavior here follows common tab practice, not a checked Fork spec.

## Goals / Non-Goals

**Goals:**
- Open many repositories as tabs. Switch with one click. A switch shows the exact state the user left.
- A task never changes another tab's state.
- A background tab costs no git process.
- Restore the tabs at the next start.
- Keep the diff small: the per-repository code stays as it is.

**Non-Goals:**
- Keyboard shortcuts for tabs. Drag to reorder. A change mark on background tabs.
- More than one window.
- A "recent repositories" list.

## Decisions

### D1. Each tab is its own gpui entity <!-- evergreen-candidate -->
Rename today's `Home` to `RepoView`. It keeps `screen`, `scroll`, `focus`, `bench`, and every method in `home.rs`, `working.rs`, and `branches.rs`. It loses the "Open repository..." button and the poll. A new `Shell` view owns `tabs: Vec<Tab>` and `active: usize`. A `Tab` holds an `Entity<RepoView>`, the repository path as the user opened it, and the resolved `RepoKey` once known.
Every task is spawned on its `RepoView` (`cx.spawn` returns a weak handle to that entity). A task that ends after its tab closed finds a dead handle, and `update(...)` returns `Err`. The task code already ignores that error. A result can only reach the entity that started the task. This closes follow-up 8.8 without a generation counter.
Alternative: one `Home` with a `Vec<GraphState>` and an `active` index, and a tab id in each task. Rejected: every task and every `graph_mut()` call needs the id check, and one forgotten check is a cross-repository bug (principle: make-illegal-states-unrepresentable). The entity split makes the bug impossible by structure.

### D2. Close drops the entity
Closing a tab removes its `Tab` from the list and so drops the `Entity<RepoView>`. The `Task` fields inside it drop and cancel. The log stream already kills its git process when it drops (`stream.rs`). No extra stop code. Check this with a test on `LogStream` drop if it has none (see tasks).

### D3. The shell owns one poll, and only the active tab uses it
The 2-second loop moves to `Shell`. Each tick, when the window is active, it calls `poll_working` on the active `RepoView` only. A tab runs `refresh_working` once when the user activates it, and once at window activation, as today. No background tab polls. A background tab may finish the read of an open the user started: it reads its first page of commits and one status. That is one bounded read, not a poll.
Alternative: one poll per `RepoView` that checks an `active` flag. Rejected: it spreads the rule over many places and keeps N timers alive.

### D4. Same repository means same `RepoKey` <!-- evergreen-candidate -->
`gitcore` gets `RepoLocation::key() -> RepoKey`. `RepoKey` holds the runner and a normalized path. The normal form:
- Windows runner: backslashes, no trailing separator, lowercase (Windows paths ignore case).
- WSL runner: distro name lowercase, Linux path exact (Linux paths keep case), no trailing `/`. `\\wsl$` and `\\wsl.localhost` already give the same `RepoLocation`.
The shell checks the key in two places. First, when the user opens a path, it parses the path and compares before it reads anything. Second, when a tab finishes opening, it compares the resolved top-level key. This second check catches a sub-folder of an open repository. When it finds an equal key in another tab, it closes the new tab and activates the old one. The pure key function carries the logic, so tests need no window (principle: model-the-domain).
Alternative: compare the raw strings. Rejected: `c:\Repo\` and `C:\repo` would open twice.

### D5. A restored tab loads on its first activation
At start, the shell creates one `RepoView` per saved path in a `Pending` screen. Only the active tab opens at once. Any other tab opens when the user first activates it. This follows the rule of D3: a tab that is not on screen runs no git command. It also keeps start-up fast with many saved tabs, and it keeps memory low.
`Screen` gets a `Pending(String)` variant. `RepoView::activate` opens it when it is `Pending`.
Alternative: open every saved tab at start. Rejected: ten WSL repositories would start ten `git log` processes at once.

### D6. Saved tabs are a plain text file
`%LOCALAPPDATA%\git-win-wsl-gui\tabs.txt`. Line 1 is the active index. Each next line is one repository path. A tab saves the path the user gave until its first read ends. Then the tab saves the top-level path of the repository (`RepoLocation::open_path`; a WSL repository becomes `\\wsl.localhost\<distro>\...`). This way a restored tab and a new open of the repository have the same key. The file has no serde and no new dependency. A pure module in `app` has `encode(&[String], usize) -> String` and `decode(&str) -> (Vec<String>, usize)`. `decode` is tolerant (boundary-discipline): a missing file, an empty file, a non-number first line, or an index out of range gives an empty list or index 0, and never a crash. The shell saves after each open, close, and activation. A write error goes to the debug log (`trace::record`) and nothing else.
Alternative: JSON with serde. Rejected: a new dependency for a list of strings.
A path that holds a newline cannot round-trip. Windows paths cannot hold one. A Linux path can, and we accept the loss.

### D7. Tab title
A pure function `titles(paths: &[String]) -> Vec<String>`. The title is the last path segment. When two tabs get the same title, both add ` · <parent segment>`. The full path goes into a hover tooltip. The title of a `Pending` or `Failed` tab uses the path the user opened.
The function lives in `app` (it only formats) and has unit tests.

### D8. Tab bar widget
Use the `gpui-component` `TabBar` and `Tab` (`tab::{Tab, TabBar}`). `Tab::suffix` carries the `×` button. `TabBar::suffix` carries the `+` button. The bar scrolls when tabs overflow through the default scroll handle of `TabBar`. The middle click handler is a `MouseButton::Middle` mouse-up listener on each tab. `Tab` implements `InteractiveElement`, so the listener sits on the tab itself (it compiles). The bar needs no wrapper. No machine check of a real middle click was possible (the test setup cannot send mouse input). The user must confirm it by hand.
Alternative: build the bar from `div`s. Rejected until the widget proves it cannot do this: less code, same theme.

### D9. Window-level pieces stay in the shell
The window-activation observer moves to the shell. On activation it calls `refresh_working` on the active tab and `follow_system(cx)`, as now. The command-line path becomes `shell.open(path)`.

### D10. Test seams (tdd)
Test first for:
- (a) `RepoLocation::key` and `RepoKey` equality in `gitcore`: case, trailing separator, `wsl$` vs `wsl.localhost`, Linux case kept.
- (b) `titles` in `app`.
- (c) `encode` and `decode` in `app`, including bad input.
- (d) the tab-list logic as a pure type `TabList` that holds `Vec<entry>` and `active`: open, dedupe, close, activate, what the active index becomes after a close. It holds no gpui type, so tests need no window. `Shell` holds a `TabList` and puts an entity next to each entry.
- (e) `LogStream` drop stops git (only if no test exists).
We write no automated UI test. We check the tab bar by hand and by screenshot, as in v1.

## Risks / Trade-offs

- [Risk] Rename `Home` to `RepoView` touches `home.rs`, `working.rs`, `branches.rs`, `detail.rs`, and `main.rs`. → Mitigation: it is a type rename, then the compiler lists every site. No logic change in the same step.
- [Risk] Each `RepoView` holds its history in memory. Many big repositories add up. → Mitigation: history already loads in pages of 500. A user who wants less closes tabs. A later change can drop the history of far-away tabs.
- [Risk] The `Pending` screen is a new state in every place that matches on `Screen`. → Mitigation: the compiler forces each `match` to handle it.
- [Risk] Keyboard focus. Each `RepoView` has its own focus handle, and the tab bar adds tab stops. → Mitigation: on activation, focus the `RepoView` handle. Check Tab-key order by hand.
- [Trade-off] The tab bar is mouse-only. README says every action works without a mouse. → README states the exception. Shortcuts are a follow-up.
- [Trade-off] A closed tab loses its typed commit message with no prompt. This follows the user's decision.
- [Risk] Save on every change writes a small file often. → It is a few lines and happens only on a user click.

