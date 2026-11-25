# Open repositories

These two decisions hold for any feature that handles more than one repository. They came from the `repo-tabs` change (design D1 and D4).

## One entity per open repository
Each open repository is its own gpui entity (`RepoView`). The entity owns the state of that repository and every background task that works on it. A task reports back to its own entity through a weak handle. When the entity drops, its tasks stop, and a late result finds a dead handle and is dropped.
A feature must not keep the state of several repositories in one view and tag each task with an id. One forgotten check then sends a result to the wrong repository. The entity split makes that bug impossible by structure.
`Shell` owns the list of entities (`TabList`), the tab bar, the one 2-second poll, and the saved tab list. Only the active entity polls.

## Same repository means same `RepoKey`
`RepoLocation::key()` gives a `RepoKey`. Two locations are the same repository when their keys are equal.
- Windows runner: `/` and `\` are the same, a trailing separator is ignored, case is ignored.
- WSL runner: the distro name ignores case, a trailing `/` is ignored, and the Linux path keeps its case.
- `\\wsl$\<d>\...` and `\\wsl.localhost\<d>\...` give the same location.
Compare keys, never raw path strings. A path that the user typed is not the top-level directory of the repository. A key made from the typed path can differ from the key made after `git` resolves the top-level directory. So the app checks the key before the read and again when the read ends. `RepoLocation::open_path()` turns a resolved location back into a path that opens the same repository. The app saves that path.
