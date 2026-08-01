# Goal 20 — Swarm Disaster Runtime

## Objective

Implement and release the complete deterministic, engine-agnostic Version 4.4
Simulated Universe: Swarm Disaster runtime over the released
`starclock-activity`, `starclock-combat`, `starclock-build`, replay and
controller boundaries.

Promote the immutable Goal 09 Candidate reference bundle through private Sora
loading and validated domain lowering, execute the three-plane run and every
enabled Swarm Disaster mechanic, materialize real nested battles, and expose
the same offered-command surface through the baseline controller, CLI, agent
API and MCP adapters.

This goal releases one playable `swarm-disaster.profile.v1` profile. It does
not create a Swarm-specific state machine, command processor, RNG,
`BattleSpec` protocol, replay format or adapter API.

## Frozen prerequisites

- Goals 01–09 and Goal 14 are complete and their registered snapshots remain
  immutable.
- Goal 09 is complete at Version 4.4 / source access date 2026-07-22.
- Goal 09 profile ID is `swarm-disaster.profile.v1`.
- Goal 09 source manifest contains 6,963/6,963 `DataReady` obligations:
  6,305 Swarm-owned and 658 shared.
- Goal 09 runtime denominator contains 23 reference-only mechanic rules,
  23 semantic fixture families and 31 nonblocking reference-stage policy
  boundaries.
- The frozen Goal 09 normalized-pack SHA-256 is
  `82f3ffc444a1cdcd8bcba5a946bee3a3c8d58527b93a1c9d77f285697401b2d8`.
- The frozen Goal 09 Candidate Sora bundle SHA-256 is
  `385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362`.
- The Goal 09 release evidence is
  `evidence/swarm-disaster-reference-v1/release-evidence.json`.
- Current release-snapshot verification and the merged Candidate integration
  audit pass before the first runtime mutation.

Changing a prerequisite requires an explicit compatibility decision and new
Goal 20 evidence. Historical Goal evidence and hashes must never be rewritten
to match the current tree.

## Terminal outcome

- the frozen 65-table Goal 09 bundle loads through private generated readers
  and lowers into an immutable validated Swarm Disaster catalog;
- the catalog composes with combat, build, Activity core, released shared
  Universe content and static handler registries through component-aware
  identity;
- entry, five difficulties, Path, Audience Die, unlock/progression input and
  initial resources validate;
- the three-plane topology, domains, beacons, route choices, Countdown,
  Planar Disarray, boss decay and boss-choice consequences execute through
  generic Activity graphs, scopes, slots and operations;
- all eight Audience Dice, 42 faces, rolls, rerolls, cheats, targeting,
  graph mutations and no-candidate behavior execute deterministically;
- Communing Device choices, seven dimensions, cabinets, Communing Trail,
  Pathstrider objectives, Trailblaze Bonuses, Propagation and Resonance
  Interplays execute through ordinary Activity and battle contributions;
- all reachable shared and Swarm-owned Blessings, Curios, Occurrences,
  services and abstract Adventure outcomes execute with validated pools;
- every reachable encounter materializes a real immutable `BattleSpec`, uses
  current Activity inventory and mode contributions, and settles only through
  a verified declared result projection;
- all 6,963 source obligations and 23 mechanic rules receive exact-once runtime
  dispositions, and all 23 fixture families execute against production values;
- all 31 inherited policy boundaries become versioned executable policies,
  proven metadata, stronger-evidence replacements or terminal blockers; no
  released legal run reaches an unresolved fail-closed branch;
- seeded complete runs, replay, baseline AI, CLI, agent API and MCP use the
  same canonical offered commands and real nested battles; and
- cross-platform determinism, malformed-input, rollback, performance,
  dependency, architecture and clean-checkout release gates pass.

## Non-goals

- another Universe, challenge or event-mode runtime;
- a new generic Activity state machine or Swarm-only `apply` implementation;
- coordinates, free movement, rendering, UI, story presentation or assets;
- reproducing Adventure movement, aiming, physics or timing input;
- account rewards, achievements, collection rewards or live scheduling;
- exposing generated Sora rows as public/runtime domain types;
- relabeling Goal 09 `ProjectPolicy` facts as observed parity without retained
  evidence; or
- changing Standard Universe or Gold and Gears behavior merely to reuse a
  similarly named source row.

## Architecture invariants

1. `Activity::apply(ActivityCommand)` and `Battle::apply(Command)` remain the
   only authoritative mutation boundaries.
2. Swarm Disaster is a profile and bounded mode-owned component set in
   `starclock-mode-universe`, never a second run engine.
3. Shared crates never branch on Swarm content, die, Path, boss or mode IDs.
4. Generated Sora rows lower privately into immutable Starclock definitions;
   runtime never reads Excel, normalized JSON or debug exports.
5. Shared Path, Blessing, Resonance, Curio and combat semantics resolve by
   stable released identity; Swarm copies own only truthful differences.
6. Mode handlers are statically composed, bounded and return ordinary typed
   Activity or combat operations.
7. Graph generation and mutation use validated generic graph operations and
   preserve visit, option and transaction limits.
8. Countdown, Disarray, dice, Communing state and progression are typed bounded
   Activity state with explicit scope, reset, carry and visibility policies.
9. Random choices use project-owned labeled streams, stable candidate order and
   explicit empty-candidate draw behavior. Battle streams remain independent.
10. Rejected commands and nested results preserve state, RNG counters and
    hashes byte-identically.
11. Replay identity binds only consumed ordered components. Adding Swarm must
    not invalidate Standard or Gold and Gears replays.
12. Every accepted command settles to a decision, pending task, terminal result
    or versioned deterministic fault.

## Runtime-disposition contract

`G20-P0-B2` generates the exact runtime-disposition manifest. Each Goal 09
obligation receives one of `Integrated`, `Metadata`, `SharedIntegrated`,
`ExternalOutcome`, `Policy` or `Excluded`. Each mechanic rule additionally
binds a typed Activity operation, combat Rule IR contribution, reviewed static
handler, released shared executor or truthful non-executable disposition.
Loading a row or passing a reference fixture is not runtime evidence.

The frozen mechanic partitions are:

| Partition | Family | Rules |
|---|---|---:|
| `G20-P5-M01` | Profile entry | 1 |
| `G20-P5-M02` | Topology, events, domains and beacons | 4 |
| `G20-P5-M03` | Countdown, Planar Disarray and boss decay | 3 |
| `G20-P5-M04` | Audience Die lifecycle and targeting | 3 |
| `G20-P5-M05` | Communing choices and dimension points | 2 |
| `G20-P5-M06` | Communing Trail and Pathstrider progress | 2 |
| `G20-P5-M07` | Path/Propagation unlock and Resonance Interplay | 2 |
| `G20-P5-M08` | Curio lifecycle | 1 |
| `G20-P5-M09` | Occurrence choices | 1 |
| `G20-P5-M10` | Services and Adventure outcomes | 1 |
| `G20-P5-M11` | Boss and final-boss consequences | 2 |
| `G20-P5-M12` | Encounter selection | 1 |
| **Total** |  | **23** |

P0-B2 must bind the exact stable rule IDs to these partitions before mechanic
implementation starts. Later batches may not move or shrink the denominator
to make coverage pass.

## Artifact ownership and compatibility

Goal 20 may add:

```text
content-manifests/swarm-disaster-runtime-v1/
evidence/swarm-disaster-runtime-v1/
tools/goal20/
```

It may change responsibility-bounded Rust modules, tests, adapters, current
compatibility policy and current documentation. It must not rewrite:

```text
evidence/swarm-disaster-reference-v1/
content-manifests/swarm-disaster-v1/
content-reference/swarm-disaster-v1/
config/swarm-disaster/
config/swarm-disaster-generated/
```

Those roots are immutable runtime inputs. A genuine authoring defect requires
a new revisioned artifact and the documented `openpyxl`/Sora workflow; never
patch historical workbooks, generated Rust or `config.sora` directly.

## Execution and commit rules

- Execute the earliest unblocked batch and keep only one Goal 20 batch
  `InProgress` in this worktree.
- Update the ledger with exact commands, counts, hashes, decisions and blockers
  in the same batch change.
- Commit each batch atomically as
  `<type>(swarm-disaster): <batch-id> <imperative summary>`.
- Keep code, tests, dispositions and evidence for one responsibility together.
- Run focused tests per batch, the quick repository gate before ordinary batch
  completion, and the full gate at phase and release checkpoints.
- Never claim an unexecuted check passed.
- Keep handwritten Rust below 1,200 lines and split around 800 lines.

## Delivery phases

### Phase 0 — Contract, audit and frozen execution plan

| Batch | Deliverable |
|---|---|
| `G20-P0-B1` | Verify immutable prerequisites, merged Candidate compatibility, protected roots and the current generic Activity/combat/interface baseline. |
| `G20-P0-B2` | Generate exact runtime dispositions for 6,963 obligations, 23 rules, 23 fixture families and 31 inherited policy boundaries; freeze all P5 partitions. |
| `G20-P0-B3` | Freeze private catalog boundaries, typed scopes/slots, commands/events, component identities, RNG labels, replay migrations and failure policies. |
| `G20-P0-B4` | Freeze the valid seeded matrix, first vertical slice, policy owners, performance workloads, CI matrix and release scaffold. |

### Phase 1 — Bundle loading and immutable catalogs

| Batch | Deliverable |
|---|---|
| `G20-P1-B1` | Integrate private readers for the exact 65-table Candidate bundle and reject wrong schema, revision, digest or closure. |
| `G20-P1-B2` | Compose Swarm component/catalog/registry identities without changing Standard or Gold replay roots. |
| `G20-P1-B3` | Lower and validate profile, entry, difficulties, planes, boards, columns, nodes, edges, rooms, domains, beacons and boss choices. |
| `G20-P1-B4` | Lower and validate Countdown/Disarray, Audience Dice/faces, Communing Device/Trail, Pathstrider, bonuses, Paths and Interplays. |
| `G20-P1-B5` | Lower shared/mode content, services, Adventure outcomes, encounters, rules and all cross-catalog references; publish catalog coverage. |

### Phase 2 — Entry, topology, Countdown and Disarray

| Batch | Deliverable |
|---|---|
| `G20-P2-B1` | Compile entry, participant lock, difficulty, Path/Audience Die, progression input and initial resources into one Activity profile. |
| `G20-P2-B2` | Compile bounded three-plane graphs with canonical order, legal routes, reachability and visit budgets. |
| `G20-P2-B3` | Execute domain/beacon creation, replacement, copy, blanking and topology-event ordering through graph operations. |
| `G20-P2-B4` | Execute Countdown adjustment/carry, Planar Disarray transition/levels/cap and boss-decay contributions. |
| `G20-P2-B5` | Execute plane transitions, boss choices, final termination and all topology/Countdown rollback, hash and RNG fixtures. |

### Phase 3 — Audience Dice and Communing Device

| Batch | Deliverable |
|---|---|
| `G20-P3-B1` | Compile all eight Audience Dice, initial/passive effects, unlocks and Path-specific graph rules. |
| `G20-P3-B2` | Implement roll, reroll, cheat, abandon and explicit no-candidate behavior over labeled streams. |
| `G20-P3-B3` | Compile all 42 face selectors, parameters, durations, targets, graph effects and battle contributions. |
| `G20-P3-B4` | Implement Communing choices, seven dimension counters, cabinet eligibility, clamps, carry and ordered point changes. |
| `G20-P3-B5` | Prove simultaneous dice/Communing/movement/map ordering, rollback, causality and fixture parity. |

### Phase 4 — Progression, content and battle contributions

| Batch | Deliverable |
|---|---|
| `G20-P4-B1` | Implement Communing Trail prerequisites, thresholds, run/service/dice effects and immutable battle contributions. |
| `G20-P4-B2` | Implement Pathstrider objectives/progress/unlocks and mechanical chapter boundaries without story/account rewards. |
| `G20-P4-B3` | Implement Trailblaze Bonuses, Propagation unlock, Path boosts, Resonances/Formations and all Interplays. |
| `G20-P4-B4` | Reuse shared Blessing definitions and implement Swarm Curio copies, states, charges, repair, replacement and offer pools. |
| `G20-P4-B5` | Implement Occurrences, currencies, shops/services and offered Adventure outcomes with atomic costs, rewards and lifecycle effects. |

### Phase 5 — Complete mechanic partitions

Each `M` batch closes its frozen rules through production execution evidence.

| Batch | Partition |
|---|---|
| `G20-P5-M01` | Profile-entry rule. |
| `G20-P5-M02` | Topology/event/domain/beacon rules. |
| `G20-P5-M03` | Countdown/Disarray/boss-decay rules. |
| `G20-P5-M04` | Audience Die rules. |
| `G20-P5-M05` | Communing choice/dimension rules. |
| `G20-P5-M06` | Communing Trail/Pathstrider rules. |
| `G20-P5-M07` | Path/Propagation/Interplay rules. |
| `G20-P5-M08` | Curio lifecycle rule. |
| `G20-P5-M09` | Occurrence-choice rule. |
| `G20-P5-M10` | Service/Adventure rule. |
| `G20-P5-M11` | Boss/final-boss consequence rules. |
| `G20-P5-M12` | Encounter-selection rule. |
| `G20-P5-B1` | Execute all 23 Goal 09 fixture families against production values and compare ordered operations, events and hashes. |
| `G20-P5-B2` | Prove exact-once 6,963/23/23 coverage and reject enabled gaps, orphan rules, unowned handlers and scattered stable-ID branches. |

### Phase 6 — Encounters and full-run integration

| Batch | Deliverable |
|---|---|
| `G20-P6-B1` | Implement encounter-group selection, room/domain joins, effective difficulty, 81-series waves/slots and boss alternatives with explicit policies. |
| `G20-P6-B2` | Materialize real immutable `BattleSpec` values from current run inventory, participants, difficulty, Disarray, Path/Die/Trail/Curio contributions and exact enemy definitions. |
| `G20-P6-B3` | Execute nested battles, boss choices, result verification, carry, defeat/revival and atomic post-battle settlement. |
| `G20-P6-B4` | Complete the frozen seeded matrix across every difficulty, Path, Audience Die, Disarray boundary and policy family with fresh replay verification. |

### Phase 7 — Replay, controllers and external surfaces

| Batch | Deliverable |
|---|---|
| `G20-P7-B1` | Extend component-addressed Activity replay for Swarm commands, graph mutations, policies and nested-battle diagnostics. |
| `G20-P7-B2` | Add deterministic baseline scoring for routes, dice, Countdown, Communing, progression, rewards, services and Adventure using only offered commands. |
| `G20-P7-B3` | Add `starclock universe run --mode swarm-disaster` plus config/coverage/replay diagnostics in human and JSON modes. |
| `G20-P7-B4` | Extend agent Activity sessions with Swarm observations/actions while preserving existing compatibility and authority. |
| `G20-P7-B5` | Extend authorized MCP Activity tools/resources over the same session facade and preserve quota, tenant and idempotency behavior. |

### Phase 8 — Hardening and release

| Batch | Deliverable |
|---|---|
| `G20-P8-B1` | Run cross-platform command/event/hash goldens, stream perturbation, property, malformed replay, rejection identity and deterministic fault tests. |
| `G20-P8-B2` | Enforce performance/allocation budgets for catalog sharing, sessions, complete runs, battles, replay and concurrent verification. |
| `G20-P8-B3` | Run dependency/license, architecture, handler, provenance, generated-drift, prior-release and clean-checkout audits. |
| `G20-P8-B4` | Freeze public documentation, compatibility/coverage/golden evidence and the Goal 20 release contract; register the immutable snapshot. |

## Acceptance

- the exact frozen bundle and all 65 tables load privately;
- 6,963 obligations, 23 rules and 23 fixtures close exactly once;
- no runtime path reads Goal 09 JSON, Excel or debug exports;
- all graph/state/decision paths are bounded, canonical and transactional;
- all 31 inherited policy boundaries remain accurately labeled and terminal;
- every reachable encounter executes a real battle or blocks catalog release;
- rejected commands/results preserve authoritative bytes and RNG counters;
- independent RNG streams and component-aware replay identities are proved;
- baseline AI, CLI, agent API and MCP consume the same offered commands;
- the P0 matrix covers all five difficulties, eight Paths/Audience Dice,
  Disarray, mechanic fixtures and policy families;
- native platform, performance, dependency, architecture, compatibility and
  full clean-checkout gates pass; and
- `G20-P8-B4` is committed and the completion snapshot is registered.

Progress is authoritative only in
[the Goal 20 status ledger](20-swarm-disaster-runtime-status.md).
