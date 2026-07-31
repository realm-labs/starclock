# G19-P4-B3 — Peer Reconciliation and Full Gate

The three concurrent frozen manifests are locked by immutable commit, path and
SHA-256:

| Peer | Commit | Manifest SHA-256 |
|---|---|---|
| Pure Fiction | `6b30afecdb271a52e06e0922b2800fa0590e4cff` | `96295180a866e7787c2fea43b49fa454aef39015913b0920eeb4651afd1b52d9` |
| Memory of Chaos | `50fa7e37e4a9d3c8656b809dd0f4db7cdfbd8be2` | `0928632dc99c314e4a2a72b88f90ad76dd1371532882f24366ce15197c0230ce` |
| Apocalyptic Shadow | `f9f70e208b2b69f74e31f01eef0e5d620fc959bb` | `d64e4e3609f6818e5e0d072205010f3d39082ddaab260e0e5b1ca20e037c23b1` |

The verifier reads each file through `git show <commit>:<path>`, rechecks the
digest and compares every discoverable exact path + locator + digest receipt.
There are zero exact shared receipts and zero same-locator digest conflicts.
Different locator schemes are not promoted to shared identity; names and ID
adjacency are deliberately ignored. The current pack digest is
`59bcb142e1d7be2b95f6a99ba3ad806f1afefa8ce8708066591b08bc51aa171d`.

Focused workbook and Sora regeneration remain byte-identical. The first full
gate stopped because the worktree lacked the checksum-bound Sora crate archive;
`node tools/sora/install.mjs` restored it. The second stopped because the legacy
Universe fixture invoked a default Python without openpyxl. With the Goal-owned
3.1.5 venv prepended to PATH, the exact full command passed 28 generated checks,
Clippy and 33 workspace harnesses in 451.0 seconds:

```text
PATH=/Users/mikai/.codex/worktrees/goal19-fate-star-rail-night/starclock/.cache/g19-venv/bin:$PATH fnm exec --using 24.15.0 node tools/repository-check/run.mjs --full
```
