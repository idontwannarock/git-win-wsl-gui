# git-win-wsl-gui

Windows-only Git GUI in Rust. UI: `gpui` + `gpui-component`. Git: the real `git` binary, through `git.exe` or `wsl.exe -d <distro> --exec git`. Specs live in `openspec/`.

## Code rules (from pstack, confirmed by the user)

### No comments
Write no comments in code. This includes `//`, `/* */`, and `///`.
Only these exceptions may stay:
- A license header.
- Behavior forced by an outside dependency, platform, or protocol that we cannot change.
- A lint suppression whose rule is faulty, pedantic, or style-only.
- A doc comment that defines a public API contract.
- An issue or RFC link that code cannot express.
If a comment explains a surprise in our own code, rename, extract, or add a type so the code explains itself. Delete the comment.
A rule that code can enforce goes into a type, a test, or a lint. It does not go into prose.

### Principles
Name the principle in your reply when it changes a decision.
1. laziness-protocol: delete first. Make the smallest change that solves the problem.
2. subtract-before-you-add: remove dead weight before you build on it.
3. minimize-reader-load: fewer layers, less hidden state.
4. foundational-thinking: choose the data structures before the logic.
5. model-the-domain: put a repeated rule in one structure, not in scattered conditionals.
6. boundary-discipline: validate at the boundary (git output, user input). Trust internal types.
7. type-system-discipline: make illegal states unrepresentable (Rust enums, newtypes).
8. test-behavior-not-implementation: assert what a user of the code can observe.
9. prove-it-works: verify the real artifact, not a proxy.
10. fix-root-causes: reproduce first, then trace to the cause.
11. sequence-verifiable-units: end each small unit with a check before the next.
12. explain-the-number: for a benchmark, name what limits the number and rule out that it measured something else.

## Verify
`cargo test`, `cargo clippy -- -D warnings`, `cargo build --release`.
