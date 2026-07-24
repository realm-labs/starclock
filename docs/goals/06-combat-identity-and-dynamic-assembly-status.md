# Goal 06 Status — Combat Identity and Dynamic Per-Battle Assembly

## Goal state

| Field | Value |
|---|---|
| Goal ID | `combat-identity-dynamic-assembly-v1` |
| State | `Ready` |
| Active phase | None |
| Active batch | None |
| Next unblocked batch | `G06-P0-B1` |
| Required snapshot | Goal 05 `standard-universe-end-to-end-v1` |
| Planned batches | 18 |
| Blocking condition | None |

Goal 06 has not started. This ledger becomes the resumable source of truth once
`G06-P0-B1` is marked `InProgress`.

## Batch ledger

| Batch | State | Evidence | Result |
|---|---|---|---|
| `G06-P0-B1` | `Pending` | — | Freeze execution package and identity terminology. |
| `G06-P0-B2` | `Pending` | — | Freeze digest/materialization debt probes. |
| `G06-P0-B3` | `Pending` | — | Freeze replay, performance and release scaffold. |
| `G06-P1-B1` | `Pending` | — | Compute combat input identity in combat-core. |
| `G06-P1-B2` | `Pending` | — | Migrate Activity handoff and settlement identity. |
| `G06-P1-B3` | `Pending` | — | Add replay v3 and preserve replay v2 verification. |
| `G06-P1-B4` | `Pending` | — | Migrate callers, event attribution and touched file layout. |
| `G06-P2-B1` | `Pending` | — | Separate catalog composition and assembly; define cache key. |
| `G06-P2-B2` | `Pending` | — | Project the current Activity contribution snapshot. |
| `G06-P2-B3` | `Pending` | — | Assemble every pending battle dynamically and atomically. |
| `G06-P2-B4` | `Pending` | — | Prove cache invalidation, rollback and retry. |
| `G06-P2-B5` | `Pending` | — | Prove acquire/upgrade/remove effects in real battles. |
| `G06-P3-B1` | `Pending` | — | Migrate CLI and baseline runs. |
| `G06-P3-B2` | `Pending` | — | Migrate Agent and MCP surfaces. |
| `G06-P3-B3` | `Pending` | — | Verify replay reconstruction and interface parity. |
| `G06-P4-B1` | `Pending` | — | Freeze performance and source-structure hardening. |
| `G06-P4-B2` | `Pending` | — | Run full matrix, corruption and native CI evidence. |
| `G06-P4-B3` | `Pending` | — | Freeze release and register immutable snapshot. |

## Starting debt

| Debt | Goal 05 state | Goal 06 closure |
|---|---|---|
| Battle-visible digest | Caller supplies `BattleSpecDigest`. | Combat-core computes `CombatInputDigest`; outer provenance is separate. |
| Production assembly | Factory freezes an empty entry-time contribution snapshot. | Every pending battle consumes the current Activity snapshot. |
| Replay | Component-addressed v2 records the frozen materialization. | v3 records exact assembly and combat-input identity per battle. |
| Cache | One immutable materialization avoids recomputation but becomes stale. | Immutable catalog plus bounded exact-key assembly cache. |
| Event outer source | `activity_source` is replay-encoded but has no production writer. | Resolve under the explicit event-codec revision. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-24 | Goal 06 does not claim completion of the 783 retained rules. | Dynamic selection and content implementation are independent denominators. |
| 2026-07-24 | The combat-owned digest and opaque assembly digest are separate. | Server verification must not trust a caller-provided identity, while combat must not understand build/mode data. |
| 2026-07-24 | Catalog definitions are composed once; each battle selects bindings. | Rebuilding the catalog per battle is unnecessary and harms service throughput. |
| 2026-07-24 | General nested-runner extraction is deferred until a second gameplay family needs it. | This goal should close observed identity/assembly debt without speculative abstraction. |

## Terminal record

| Field | Value |
|---|---|
| Final state | Not started |
| Completion commit | — |
| Combat identity revision | To be frozen |
| Replay revision | To be frozen |
| Dynamic assembly coverage | To be generated |
| Performance evidence | To be generated |
| Release evidence | To be generated |

