# Goal 06 Status — Combat Identity and Dynamic Per-Battle Assembly

## Goal state

| Field | Value |
|---|---|
| Goal ID | `combat-identity-dynamic-assembly-v1` |
| State | `InProgress` |
| Active phase | Phase 2 — Dynamic per-battle assembly |
| Active batch | None |
| Next unblocked batch | `G06-P2-B5` |
| Required snapshot | Goal 05 `standard-universe-end-to-end-v1` |
| Planned batches | 18 |
| Blocking condition | None |

Goal 06 is active. Phase 1 is complete: combat, Activity and replay v3 carry
independent canonical input and assembly identities, all constructors use the
combat-owned identity boundary, and event payload v2 removes the unused outer
source slot without breaking replay-v2 verification. P2-B1 split the released
encounter/enemy catalog composition from selected contribution assembly and
added a bounded, non-authoritative exact-key cache. P2-B2 now projects one
typed immutable snapshot of all current battle contributions and participant
carry. P2-B3 resolves the selected encounter from that snapshot and atomically
replaces/seals the prepared placeholder. P2-B4 now bounds assembly work and
proves exact state/RNG preservation across stale, invalid, budget and cache
failure paths. P2-B5 will prove that inventory and carry transitions change
real successive battles.

## Batch ledger

| Batch | State | Evidence | Result |
|---|---|---|---|
| `G06-P0-B1` | `Complete` | `node tools/goal06/verify-foundation.mjs`; `node tools/repository-check/verify-release-snapshots.mjs`; quick repository gate | Froze the 5-phase/18-batch execution package and normative identity/assembly contract. Bound the exact Goal 05 completion commit/tree and starting 2,201/786/78 denominator. Defined combat-owned `CombatInputDigest`, opaque `AssemblyDigest`, `BattleAssemblyKey`, replay-v3 target and immutable replay-v2 history without claiming the remaining 783 rules. |
| `G06-P0-B2` | `Complete` | `node tools/goal06/verify-debt-probes.mjs`; quick repository gate | Froze 32 caller-supplied `BattleSpecDigest` constructor calls across 25 Rust files, the shared factory's one entry-time empty-inventory materialization, the unused current-Activity contribution seam and the single CLI/Agent/MCP authority that must migrate together. Added six ordered transition scenarios covering Blessing upgrade, Curio disable/remove, Resonance, Ability Tree, carry-only and provenance-only identity changes. |
| `G06-P0-B3` | `Complete` | `node tools/goal06/verify-phase0.mjs`; full repository gate | Froze historical component replay v2 and the v3 six-field nested identity/first-divergence contract, six identity/assembly/cache/concurrency/matrix performance workloads under the 180-second focused budget, a zero-new-dependency Cargo baseline and the five-phase/18-batch release scaffold. Phase 0 generated/drift checks now run mechanically. |
| `G06-P1-B1` | `Complete` | `node tools/goal06/verify-phase1-b1.mjs`; `cargo test -p starclock-combat`; combat clippy; workspace check | Added the combat-owned `SCBI` v1 canonical codec and computed `CombatInputDigest`, separated opaque `AssemblyDigest` in `BattleSpec` and runtime battle identity, and retained the historical state/replay bytes until the coordinated P1-B3 revision. The temporary constructor bridge was removed by P1-B4. |
| `G06-P1-B2` | `Complete` | `node tools/goal06/verify-phase1-b2.mjs`; Activity/replay/Universe tests; workspace check | Migrated pending battle views, deterministic battle seeds, handoffs, result envelopes, result digests and settlement validation to independent combat-input and assembly identities. Advanced authoritative Activity state to codec v3 / `sha256-v5`; dual-identity payloads emit current versions while released single-digest payloads retain explicit read-only decoders. Independent mismatch tests preserve byte-identical Activity state. |
| `G06-P1-B3` | `Complete` | `node tools/goal06/verify-phase1-b3.mjs`; combat/replay/Universe tests; focused Clippy; quick repository gate | Advanced combat state to `SCBS` v3 / `sha256-v4`, binding combat-input codec, computed input digest and assembly provenance independently. Added component-addressed replay v3 plus six-field nested battle identity payloads and Standard Universe verification with the frozen component → assembly → combat-input → command → event → state → result → Activity first-divergence order. Historical v2 decode/verification remains available and its exact envelope bytes are SHA-256 frozen. |
| `G06-P1-B4` | `Complete` | `node tools/goal06/verify-phase1-b4.mjs`; workspace no-run; replay/Universe/Agent tests; focused Clippy; quick repository gate | Unified all battle construction on `BattleSpec::new(..., AssemblyDigest, ...)`, leaving combat-input identity computed only by combat-core. Removed the unwritten `activity_source` field, advanced new replay-v3 recordings to event payload v2 while retaining byte-exact payload-v1 replay and event-commitment verification, and split event cause encoding plus Universe request construction out of near-limit files. |
| `G06-P2-B1` | `Complete` | `node tools/goal06/verify-phase2-b1.mjs`; battle-materialization tests; workspace check; focused Clippy; quick repository gate | Added a once-built immutable encounter/enemy catalog composition, a canonical six-field `BattleAssemblyKey`, and a default-64 deterministic FIFO cache whose entries validate their exact key. Production factory construction now reuses the composition, while cache clear/eviction remain outside authoritative state and preserve battle identities. |
| `G06-P2-B2` | `Complete` | `node tools/goal06/verify-phase2-b2.mjs`; Activity and encounter-runtime tests; workspace check; focused Clippy; quick repository gate | Added `StandardUniverseBattleSnapshot` with typed Path, Blessing, Resonance/Formation, Curio lifecycle, Ability Tree projection, final contributions and ordered carry. `battle_start_snapshot()` derives context from current Activity, binds the source state hash, and rejects mismatched explicit contexts; provenance-only changes alter the snapshot without fabricating combat changes. |
| `G06-P2-B3` | `Complete` | `node tools/goal06/verify-phase2-b3.mjs`; Activity, materialization and dynamic-start tests; workspace check; focused Clippy; quick repository gate | Added the shared `StandardUniverseBattleAssembler`, snapshot/carry-aware materialization and an atomic generic Activity replacement/start operation. The returned handoff is paired with its exact immutable combat catalog; encounter, preparation variant, dual identities, contract and seed are sealed together without RNG draws or partial Activity publication. |
| `G06-P2-B4` | `Complete` | `node tools/goal06/verify-phase2-b4.mjs`; dynamic assembly integration tests; focused Clippy; quick repository gate | Added explicit assembly budgets and configurable bounded cache policy. Integrated fixtures prove exact-key hits and FIFO eviction, reject stale snapshots, unregistered techniques and budget overflow before publication, preserve canonical Activity/RNG bytes, and successfully retry the same pending battle. |
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
| Event outer source | `activity_source` was replay-encoded but had no production writer. | Removed from combat; payload v2 uses `source_definition`, while historical payload v1 remains verifiable. |

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
