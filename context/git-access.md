# Git access and WSL runner

These two decisions hold for the whole project. They came from the `v1-core` change (design D2 and D3).

## Shell out to the real git
All repository data comes from `git` output. We parse machine formats: `status --porcelain=v2 -z --branch`, `log` with a NUL-separated `--format`, `diff`/`show` unified text, `for-each-ref --format`.
Alternatives: `libgit2` (git2) or `gix`. Rejected: they do not run the distro's git, they ignore some hooks and credential helpers, and WSL repositories are slow or wrong through `\\wsl$`.

## WSL path means the distro's git runner
`Runner` has two variants. `Windows` runs `git.exe -C <path>`. `Wsl { distro }` runs `wsl.exe -d <distro> --cd <linux-path> --exec git`. A pure function maps a path to a runner and a native path. It handles `\\wsl$\<d>\...` and `\\wsl.localhost\<d>\...`. Git never reads a WSL repository through the Windows share.
Alternative: always use Windows `git.exe` on the share. Rejected: very slow, and file mode and line-ending results differ.
A per-repository override of the git binary is out of scope for v1.
