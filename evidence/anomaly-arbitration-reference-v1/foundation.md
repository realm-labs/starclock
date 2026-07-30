# Goal 13 Foundation

`G13-P0-B1` starts `anomaly-arbitration-reference-v1` from the remotely
published Goal package commit
`5d1c04605d547ba147cf9661f33772b8153ef774`. Goal 13 produces a
Candidate-quality Version 4.4 Anomaly Arbitration reference bundle. It does
not lower data into the runtime or change `starclock-combat`,
`starclock-activity`, `starclock-mode-challenge`, `starclock-build`, shared
operations, shared schemas or shared generated readers.

## Frozen prerequisite

Goal 03 is complete at commit
`60ca52ed98c5c83d867d33bff7f88c69e0b389de` with tree
`3fc33d2b45ad9344522faf6e470c861ab75ff4c5`. The repository's immutable
release-snapshot verifier accepts its registered release policy and evidence.
The source boundary remains game Version 4.4, accessed on 2026-07-22:

| Repository | Revision | Tree |
|---|---|---|
| `Dimbreath/turnbasedgamedata` | `fd978d6ef09f941fba644c731ab54abd6f7c3568` | `2df8981c1bea512e21c8c900920c63002b381056` |
| `Mar-7th/StarRailRes` | `7b349e39ee0f6f3bf814567995829b99c95e7a93` | `1e6892227905e0dad002bb117d63464d7a5640a6` |

The six `ChallengePeak*` tables, direct battle-event ability/layout,
TextMaps and named shared tables are discovery entries only. They are not the
active-period denominator. Group 8, aliases 801–804, the five planning stage
IDs, targets 3000–3005/3007 and battle events 30502–30504 remain hypotheses
until `G13-P0-B3` closes the explicit active selector and transitive references.

## Source-cache reproduction

The main-worktree cache was inspected read-only. Both repositories were clean,
detached at the exact revisions, configured with canonical origins, able to
read the required commits/blobs and passed
`git fsck --connectivity-only --no-dangling`. Its sparse-checkout configuration
was not changed.

An attempted fresh GitLab HTTP/1.1 partial clone made no progress during a
bounded wait and was terminated. No fresh-network success is claimed. The
accepted substitute uses
`tools/anomaly-arbitration-reference/fetch-sources.sh` with a clean connected
fixed-commit seed. On macOS it creates a copy-on-write filesystem clone into a
new `/tmp` target, then independently sets the Goal 13 sparse paths. The
result retains the canonical origins and is detached, clean and connected.
Two consecutive reproducer/verifier runs against the isolated target must
pass.

The foundation freezes 18 turnbasedgamedata file receipts with canonical-list
digest
`762d4b79a8cc2afd1eaa01f68d41be2bcc2ba59b1e98b50f83f23300638c90df`
and three StarRailRes root receipts with digest
`864fa4ed09713b2564d8ec3304993d10fca103b0ce347debb1880ef258fa71a4`.
Every reproduced file was byte-identical to the corresponding Git blob in the
shared clean cache. Bulk source material stays ignored and uncommitted.

## Concurrent ownership checkpoints

Only committed Git blobs are reconciliation inputs. Other worktrees' staged,
unstaged and untracked files were not consumed as authority and were not
modified.

| Goal | Committed checkpoint | Boundary |
|---|---|---|
| Goal 07 | `4f466df77a25694777ccfddbdce7e6bdeabb0931` | Remote-merged retained audit and execution partitions: 2,201 records, 786 rules and 78 fixtures. |
| Goal 08 | `43b989c4f1f48842910329af3c60e76da8128d90` | Local committed manifest only: 7,913 records, 7,199 Gold and Gears and 714 Shared; no configured origin branch contains it. |
| Goal 09 | `9bd2ad285de4c10e7ab060f00bf078855923a09c` | Remote-backed manifest: 6,963 records, 6,305 Swarm Disaster and 658 Shared. |
| Goal 10 | `ce2f6b209ae24c1742365d49c57422fce0145683` | Remote-backed manifest: 5,377 records, 5,243 Unknowable Domain and 134 Shared. |
| Goal 11 | `f202c1bd0769922be394d4983dc4d0f0f3121779` | Remote-backed source inventory with 2,684 records; no committed ownership manifest was available. |
| Goal 12 | `74cb56a228d03f05ee4e27410c074aca159d5393` | Remote-backed foundation only; no committed source or ownership manifest was available. |

Later branch advancement cannot mutate these exact commits. Goal 08 must be
replaced by a remote-backed or merged checkpoint, and Goal 11/12 by committed
ownership manifests if they become available, during `G13-P4-B3`.

No checkpoint grants Anomaly Arbitration membership. A shared fact reconciles
by source path, stable row locator and evidence digest. Prefix, ID adjacency,
period-like numbering and equal localized names do not establish ownership or
reachability. Conflicts wait for merge coordination rather than changing
another Goal's artifacts.

## Scope and authoring boundary

Included work covers the stable family and active Version 4.4 period; entry
eligibility; Knight and King flow; three disjoint recorded teams; replacement
and reset; King protection and Plight; shortcut and Quadrant decisions;
independent clocks and first-cycle/wave/low-cycle boundaries; targets, stars
and settlement; reachable shared records; audited analogous content pools;
stages, waves, enemies and battle contributions; bilingual provenance;
coverage and semantic fixtures; and isolated Excel/Sora authoring.

Story and presentation prose, assets, audio, UI, rewards/history, avatar
frames, medals, item payloads, wall-clock rotation, historical periods,
unreleased material and unrelated modes are excluded except for bounded
ownership or prerequisite evidence. Runtime lowering and shared-runtime work
belong to a later goal.

Excel `.xlsx` files authored with Python `openpyxl` are the production
authoring surface. Sora 0.3.0 is the schema, generation and export authority.
The host `PATH` reports Sora 0.2.0, which is not authoritative. JSON remains
staging/debug only.

## Artifact isolation

Goal 13 runs only on
`codex/goal13-anomaly-arbitration-reference`, tracking the identically named
`origin` branch. It owns these six roots plus its three Goal documents:

- `content-manifests/anomaly-arbitration-v1/`
- `content-reference/anomaly-arbitration-v1/`
- `config/anomaly-arbitration/`
- `config/anomaly-arbitration-generated/`
- `tools/anomaly-arbitration-reference/`
- `evidence/anomaly-arbitration-reference-v1/`

Every Goal 07–12 artifact, `config/universe-generated/` and
`config/generated/` is protected. The plan contains 25 atomic batches across
five phases. A completed batch is pushed and remotely resolved before the next
batch starts.
