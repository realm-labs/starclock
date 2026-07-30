# Goal 14 — Gold and Gears Runtime

## Objective

Implement the complete deterministic, engine-agnostic Version 4.4 Simulated
Universe: Gold and Gears runtime over the released `starclock-activity`,
`starclock-combat`, `starclock-build`, replay and controller boundaries.

Promote the immutable Goal 08 Candidate reference bundle through private Sora
loading and validated domain lowering, execute the three-plane run and every
enabled mode mechanic, materialize real nested battles, and expose the same
offered-command surface through the baseline controller, CLI, agent API and
MCP adapters.

This goal releases one playable `gold-gears.profile.v1` profile. It does not
create a Gold-and-Gears-specific state machine, command processor, RNG,
BattleSpec protocol, replay format or adapter API.

## Frozen prerequisites

- Goals 01–07 are complete and their registered completion snapshots remain
  immutable.
- Goal 08 is complete at Version 4.4 / source access date 2026-07-22.
- Goal 08 profile ID is `gold-gears.profile.v1`.
- Goal 08 source manifest contains 7,913/7,913 `DataReady` obligations:
  7,199 Gold-and-Gears-owned and 714 shared.
- Goal 08 runtime denominator contains 1,224 reference-only mechanic rules,
  18 semantic fixture families and 16 nonblocking reference-stage policy
  boundaries.
- The frozen Goal 08 normalized-pack SHA-256 is
  `ea2f3a35807b9a7dae39be2d67fb5de955bfad7852718eb1d3393affed5a5623`.
- The frozen Goal 08 Candidate Sora bundle SHA-256 is
  `97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b`.
- The Goal 08 reference release evidence is
  `evidence/gold-and-gears-reference-v1/release/release-evidence.json`.
- The merged Goals 08–13 Candidate integration audit passes before the first
  runtime mutation.

Changing a prerequisite requires an explicit compatibility decision and new
Goal 14 evidence. Historical Goal evidence and hashes must never be rewritten
to match the current tree.

## Terminal outcome

- the frozen 52-table Goal 08 bundle loads through private generated readers
  and lowers into an immutable validated Gold and Gears mode catalog;
- the catalog composes with combat, build, Activity core, shared Universe
  content and static handler registries through component-aware identity;
- entry, five formal difficulties, Path, Custom Dice, six face slots, Neural
  Network input, both Conundrum tracks and initial resources validate;
- the three-plane chessboard flow, room/domain/beacon creation, route
  decisions, Cognition, Secrets, Knowledge and boss choices execute through
  generic Activity graphs, scopes, slots and operations;
- Custom Dice passives, rolls, rerolls, cheats, faces, target selection,
  Knowledge interactions and graph mutations execute deterministically;
- Neural Network nodes, Trailblaze Bonuses, Path boosts, Resonance
  Extrapolation/Interplay, Curio copies, Occurrences, services, Adventure
  outcomes and all reachable shared content execute;
- every structured encounter materializes a real immutable `BattleSpec`, uses
  current Activity inventory and mode contributions, and returns only through
  a verified declared result projection;
- all 7,913 source obligations and 1,224 mechanic rules have exact-once runtime
  dispositions, and all 18 semantic fixture families execute against
  production runtime values;
- all 16 inherited policy boundaries are either versioned executable policies,
  proven metadata-only, replaced by stronger retained evidence, or terminal
  blockers; no legal released run reaches an unresolved fail-closed branch;
- seeded complete runs, replay, baseline AI, CLI, agent API and MCP all use the
  same canonical offered commands and real nested battles;
- cross-platform determinism, malformed-input, rollback, performance,
  dependency, architecture and clean-checkout release gates pass; and
- the mode is promoted from Candidate reference data to a Released runtime
  component without changing unrelated mode replay identities.

## Non-goals

- Swarm Disaster, Unknowable Domain, Divergent Universe, Currency Wars,
  Anomaly Arbitration or another mode runtime;
- a new generic Activity state machine, Gold-and-Gears-only `apply`, battle
  resolver, replay verifier, controller protocol or session registry;
- coordinates, free movement, collision, rendering, animation, audio, UI or
  story presentation;
- reproducing Adventure movement, aiming, physics or timing input; Adventure
  remains a replay-recorded offered `ExternalOutcome`;
- account rewards, weekly points, achievements, collection rewards, gacha,
  persistence synchronization or live-service scheduling;
- treating generated Rust rows, normalized JSON or Excel as public/runtime
  domain types;
- relabeling a Goal 08 `ProjectPolicy` as observed or exact without retained
  evidence; or
- modifying another mode merely to share a similarly named source record.

## Architecture invariants

1. `Activity::apply(ActivityCommand)` is the only cross-battle mutation
   boundary and `Battle::apply(Command)` is the only battle mutation boundary.
2. Gold and Gears remains a profile and mode-owned component set in
   `starclock-mode-universe`; it does not own a second run engine.
3. `starclock-combat` never depends on a mode crate and never branches on a
   Gold and Gears content, dice, Path, Curio or encounter ID.
4. Generated Sora records lower privately into immutable Starclock domain
   definitions. Runtime never reads `.xlsx`, normalized JSON or debug exports.
5. Shared Blessing, Path, Resonance and other released semantics are linked by
   stable identity and verified digest. Mode-specific copies own only their
   truthful state, pool, parameter and lifecycle differences.
6. Mode handlers are statically composed, bounded, read-only over their input
   context and return ordinary typed Activity/combat operations.
7. Map generation and mutation use generic validated graph definitions and
   operations. They do not bypass graph visit limits, option generation or
   transaction rollback.
8. Cognition, Knowledge, dice state, Neural Network input and Conundrum are
   typed bounded Activity state with explicit scope, reset, carry and
   visibility policies.
9. All random choices use project-owned labeled Activity streams, stable
   candidates and explicit no-candidate draw behavior. Battle streams remain
   independently derived.
10. Rejected commands and rejected nested results preserve authoritative state,
    RNG counters and hashes byte-identically.
11. Replay identity binds only consumed ordered components and the composed
    registry. Adding or changing an unrelated mode must not invalidate a Gold
    and Gears replay, and adding Gold and Gears must not invalidate Standard
    Universe replays.
12. Every accepted command settles synchronously to a decision, pending task,
    terminal result or versioned deterministic fault.

## Runtime-disposition contract

`G14-P0-B2` generates, rather than hand-writes, the exact runtime-disposition
manifest. Each Goal 08 source obligation receives one disposition:

- `Integrated`: affects authoritative runtime behavior and has production
  execution evidence;
- `Metadata`: validated catalog/profile structure with no direct operation;
- `SharedIntegrated`: resolves to an already released shared definition plus
  Gold and Gears reachability evidence;
- `ExternalOutcome`: the external action is abstracted, while its offered
  result and resulting mutations execute atomically;
- `Policy`: a named, versioned deterministic policy with accuracy and
  replacement evidence; or
- `Excluded`: retained evidence that cannot enter the released profile.

Each of the 1,224 mechanic rules must additionally bind a typed Activity
operation, battle Rule IR contribution, statically reviewed native handler,
shared released executor, or an explicit non-executable disposition. Loading a
row, parsing a parameter, or passing a reference fixture is not execution
evidence.

The nine frozen mechanic partitions are:

| Partition | Family | Rules |
|---|---|---:|
| `G14-P5-M01` | Profile entry | 5 |
| `G14-P5-M02` | Stats Conundrum | 6 |
| `G14-P5-M03` | Auxiliary Conundrum | 6 |
| `G14-P5-M04` | Neural Network effects | 40 |
| `G14-P5-M05` | Curio lifecycle | 160 |
| `G14-P5-M06` | Occurrence choices | 384 |
| `G14-P5-M07` | Services and Adventure | 38 |
| `G14-P5-M08` | Path boosts | 495 |
| `G14-P5-M09` | Resonance Extrapolation | 90 |
| **Total** |  | **1,224** |

P0-B2 freezes the stable rule IDs assigned to every partition. Later batches
may not move or shrink the denominator merely to make coverage pass.

## Artifact ownership and compatibility

Goal 14 may add:

```text
content-manifests/gold-and-gears-runtime-v1/
evidence/gold-and-gears-runtime-v1/
tools/goal14/
```

It may change the relevant responsibility-bounded Rust modules, tests, CLI,
agent and MCP adapters, current compatibility policy and current
documentation. It must not rewrite:

```text
evidence/gold-and-gears-reference-v1/
content-manifests/gold-and-gears-v1/
content-reference/gold-and-gears-v1/
config/gold-and-gears/data/
config/gold-and-gears-generated/
```

These five roots are immutable runtime inputs by default. If execution finds
a genuine authored-data defect, stop that batch, record the incompatibility,
use the documented `openpyxl` plus Sora 0.3.0 generation path, create a new
revisioned current artifact/manifest, and preserve the Goal 08 completion
snapshot. Never patch a workbook, generated Rust or `config.sora` directly.

## Execution and commit rules

- Execute the earliest unblocked batch and keep only one Goal 14 batch
  `InProgress` per worktree.
- Update the status ledger with exact commands, counts, hashes, decisions,
  policy outcomes and remaining blockers in the same change.
- Commit each batch atomically using
  `<type>(gold-gears): <batch-id> <imperative summary>`.
- Keep runtime code, tests, dispositions and evidence for one responsibility in
  the same batch.
- Use `apply_patch` for handwritten files and the owning generator for
  generated artifacts.
- Run focused tests per batch, the change-aware repository gate before every
  batch completion, and the full gate at phase and release checkpoints.
- Do not claim an unexecuted check passed. Record exact external/toolchain
  failures and substitute evidence.
- New dependencies require exact pins, license/tool-policy updates,
  dependency-direction review and deterministic/compile-cost assessment.
- Handwritten Rust files remain below 1,200 physical lines and should split by
  responsibility before 800 lines.

## Delivery phases

### Phase 0 — Contract, audit and frozen execution plan

| Batch | Deliverable |
|---|---|
| `G14-P0-B1` | Verify Goals 01–08 snapshots and the merged Candidate audit; freeze the execution package, protected roots and current generic Activity/combat/interface baseline. |
| `G14-P0-B2` | Generate exact runtime dispositions for 7,913 source obligations, 1,224 rules and 18 fixture families; freeze all nine P5 rule partitions. |
| `G14-P0-B3` | Freeze public/private catalog boundaries, typed scopes/slots, command/event families, registry/component identities, RNG labels, replay migrations and failure policies. |
| `G14-P0-B4` | Freeze the valid seeded coverage matrix, first vertical slice, policy-gap ownership, performance workloads, CI matrix and Goal 14 release-contract scaffold. |

### Phase 1 — Bundle loading and immutable catalogs

| Batch | Deliverable |
|---|---|
| `G14-P1-B1` | Integrate private readers for the exact 52-table Goal 08 Sora bundle and reject wrong schema, revision, digest or table closure. |
| `G14-P1-B2` | Compose the Gold and Gears component/catalog/registry identities without changing Standard or unrelated mode compatibility roots. |
| `G14-P1-B3` | Lower and validate profile, entry, areas, difficulties, planes, chessboards, columns, nodes, edges, rooms, domains, beacons and boss choices. |
| `G14-P1-B4` | Lower and validate Cognition/Secret/constants, Custom Dice/slots/faces/tags/Knowledge, Neural Network, Conundrum and Path/Resonance definitions. |
| `G14-P1-B5` | Lower shared and mode-owned content, services, Adventure outcomes, encounter groups/waves/slots, mechanic rules and all cross-catalog references; publish catalog coverage. |

### Phase 2 — Entry, topology and Cognition

| Batch | Deliverable |
|---|---|
| `G14-P2-B1` | Compile entry, participant lock, difficulty, Path, Custom Dice/loadout, Neural input, Conundrum tracks, Trailblaze Bonuses and initial resources into one Activity profile. |
| `G14-P2-B2` | Generate bounded three-plane chessboard graphs with canonical node/edge ordering, legal routes, entry/terminal reachability and visit budgets. |
| `G14-P2-B3` | Execute room/domain/beacon creation, replacement, copy, blanking and map-event ordering through typed graph operations. |
| `G14-P2-B4` | Execute Cognition adjustment, clamp, carry/reset, plane-boss evaluation, Intra-Cognition ranges and Secret threshold/frontier behavior. |
| `G14-P2-B5` | Execute plane transitions, boss choices, final termination and all topology/Cognition rollback, hash and RNG-isolation fixtures. |

### Phase 3 — Custom Dice and Knowledge

| Batch | Deliverable |
|---|---|
| `G14-P3-B1` | Implement six-slot loadout validation, rarity/color constraints, unlocks, recommendations and Neural slot upgrades. |
| `G14-P3-B2` | Implement Custom Dice initial/passive effects, Path values, roll, reroll, cheat and no-candidate behavior over labeled streams. |
| `G14-P3-B3` | Compile all dice-face selectors, parameters, durations, target/filter policies and ordinary Activity/battle contributions. |
| `G14-P3-B4` | Implement Knowledge placement, propagation, query, consumption, preservation, movement override, countdown and collapse behavior. |
| `G14-P3-B5` | Prove simultaneous dice/Knowledge/movement/map-mutation ordering, rollback, event causality and semantic fixture parity. |

### Phase 4 — Progression, content and battle contributions

| Batch | Deliverable |
|---|---|
| `G14-P4-B1` | Implement all 40 Neural Network nodes, prerequisites, costs, run/service/dice effects and immutable battle contributions. |
| `G14-P4-B2` | Implement independent Stats/Auxiliary Conundrum composition, caps, replacement/cumulative behavior, Berserk and approved numeric policies. |
| `G14-P4-B3` | Implement Trailblaze Bonuses, Path boosts, Resonance additions, Interplays and Third Plane Resonance Extrapolation selection/contributions. |
| `G14-P4-B4` | Reuse validated shared Blessing/level/Resonance definitions and implement Gold-owned Curio copies, states, charges, repair, replacement and candidate pools. |
| `G14-P4-B5` | Implement Occurrence variants/choices, currency, shops/services and offered Adventure outcomes with atomic costs, rewards and lifecycle effects. |

### Phase 5 — Complete mechanic partitions

Each partition exits only when every assigned stable rule has a terminal
runtime disposition and production execution evidence.

| Batch | Partition |
|---|---|
| `G14-P5-M01` | Profile-entry rules. |
| `G14-P5-M02` | Stats Conundrum rules. |
| `G14-P5-M03` | Auxiliary Conundrum rules. |
| `G14-P5-M04` | Neural Network effect rules. |
| `G14-P5-M05` | Curio lifecycle rules. |
| `G14-P5-M06` | Occurrence-choice rules. |
| `G14-P5-M07` | Service and Adventure rules. |
| `G14-P5-M08` | Path-boost rules. |
| `G14-P5-M09` | Resonance Extrapolation rules. |
| `G14-P5-B1` | Execute all 18 Goal 08 semantic fixture families against production domain values and compare ordered operations, events and hashes. |
| `G14-P5-B2` | Prove exact-once 7,913/1,224/18 runtime coverage; reject enabled unimplemented rows, orphan rules, unowned handlers and scattered stable-ID branches. |

### Phase 6 — Encounters and full-run integration

| Batch | Deliverable |
|---|---|
| `G14-P6-B1` | Implement encounter-group selection, room/domain joins, effective difficulty, all waves/enemy slots and elite/boss alternatives with explicit policies. |
| `G14-P6-B2` | Materialize real immutable BattleSpecs from current run inventory, participant builds, difficulty, Conundrum, Path/Dice/Neural/Curio contributions and exact enemy definitions. |
| `G14-P6-B3` | Execute real nested battles, boss choices, Resonance Extrapolation, result verification, HP/Energy/presence carry, defeat/revival and post-battle settlement. |
| `G14-P6-B4` | Complete the frozen seeded matrix across every required difficulty, Path, Custom Dice, Conundrum boundary and policy family with fresh replay verification. |

### Phase 7 — Replay, controllers and external surfaces

| Batch | Deliverable |
|---|---|
| `G14-P7-B1` | Extend component-addressed Activity replay for Gold and Gears commands, graph mutations, policy identities and real nested battle divergence diagnostics. |
| `G14-P7-B2` | Add deterministic baseline-controller scoring for routes, dice/loadouts, Cognition, Knowledge, Conundrum, rewards, services and Adventure outcomes using only offered commands. |
| `G14-P7-B3` | Add `starclock universe run --mode gold-and-gears` plus config/coverage/replay diagnostics in human and JSON modes. |
| `G14-P7-B4` | Extend agent Activity sessions with Gold and Gears observations/actions while preserving Battle and Standard Universe compatibility, authority and replay export. |
| `G14-P7-B5` | Extend authorized MCP Activity tools/resources over the same session facade and preserve quota, tenant, idempotency and transport conformance. |

### Phase 8 — Hardening and release

| Batch | Deliverable |
|---|---|
| `G14-P8-B1` | Run cross-platform command/event/hash goldens, stream perturbation, property, malformed replay, rejection-byte-identity and deterministic fault tests. |
| `G14-P8-B2` | Enforce stable-runner and broad-CI performance/allocation budgets for catalog sharing, incremental sessions, complete runs, replay and concurrent verification. |
| `G14-P8-B3` | Run dependency/license, architecture, native-handler, source/provenance, generated-drift, prior-release and clean-checkout audits. |
| `G14-P8-B4` | Freeze public documentation, compatibility/coverage/golden evidence and the Goal 14 release contract; register the immutable completion snapshot. |

## Acceptance

### Data and catalogs

- the exact frozen Goal 08 Sora bundle and all 52 tables load privately;
- every generated row/reference lowers or receives a typed rejection;
- no runtime path reads Goal 08 normalized JSON, Excel or debug JSON;
- all shared identities resolve to released definitions without duplicating or
  broadening their membership;
- 7,913 source obligations, 1,224 rules and 18 fixture families close exactly
  once in generated runtime coverage.

### Activity and mode behavior

- every compiled graph is bounded and has valid entry/terminal reachability;
- every legal decision is offered canonically and unavailable IDs are rejected
  without state/RNG change;
- Cognition, dice, Knowledge, Neural, Conundrum, content and topology state
  obey declared scopes, bounds, carry and reset;
- all 16 inherited policy boundaries retain explicit accuracy and replacement
  records, and no released legal run depends on an unresolved behavior;
- Adventure accepts only offered abstract outcomes and applies the selected
  result atomically.

### Battle integration

- every reachable encounter resolves concrete validated waves/enemy slots or
  fails catalog construction;
- every battle consumes an immutable current-Activity snapshot and uses an
  independently derived battle seed;
- Gold and Gears battle-visible effects enter only through ordinary Rule IR,
  modifiers, effects, actions and events;
- rejected results preserve Activity state and valid results project only
  declared carry/metrics;
- at least one retained golden per battle-visible family proves a causal
  command/event/hash difference.

### Determinism and clients

- replay detects the first component, registry, policy, command, RNG, graph,
  nested battle, result or Activity-state divergence;
- perturbing graph, dice, Knowledge, reward, shop, occurrence or encounter RNG
  does not shift unrelated streams or battle draws;
- baseline AI, CLI, agent API and MCP consume the same ordered offered-command
  set and complete real seeded runs;
- player observations remain bounded and omit hidden RNG/controller state.

### Release

- the P0-frozen coverage matrix exercises every required difficulty, Path,
  Custom Dice, Conundrum boundary, mechanic fixture and policy family;
- Windows x86-64, Linux x86-64 and macOS ARM64 native gates reproduce the
  frozen goldens before cross-platform compatibility is claimed;
- immutable catalogs are shared and no command clones a full catalog or
  rebuilds replay prefixes;
- formatting, Clippy, tests, generated drift, file-size/visibility,
  dependency, security, Goals 01–08 compatibility and full repository gates
  pass;
- `G14-P8-B4` is committed and the final clean-worktree verifier passes.

Progress is authoritative only in
[the Goal 14 status ledger](14-gold-and-gears-runtime-status.md).
