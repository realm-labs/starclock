# Goal 04 — Standard Simulated Universe Runtime

## Objective

Implement the complete deterministic, engine-agnostic Version 4.4 main-world
Standard Simulated Universe runtime over the generic `starclock-activity` and
existing `starclock-combat` libraries. Promote the Goal 03 Excel/Sora dataset
through validated private lowering, execute all included run and battle
mechanics, support full seeded headless runs, and expose the same offered-command
surface to the CLI, baseline AI, in-process agent API and MCP adapters.

No 3D scene is implemented. Domains and monster encounters compile to finite
Activity subgraphs and explicit decisions as defined by
[Standard Simulated Universe runtime design](../25-standard-universe-runtime-design.md).

## Frozen prerequisites

- Goal 01 core combat/content release is complete.
- Goal 02 agent-control and MCP release is complete.
- Goal 03 reference release is complete at Version 4.4 / 2026-07-22.
- Goal 03 universe staging bundle SHA-256 is
  `0d94d25bf93392fb65cca1d2879a36170f70262d3dab5a92d5b634fab19f3b04`.
- The preserved Goal 01 runtime bundle SHA-256 is
  `abd84f70461675337092d12377db53f08b4562114fa90aa0b37ad869e9270440`.
- Goal 03 freezes 2,201 DataReady records, 786 rule bindings, 78 executable
  mechanic-family reference fixtures, 49 Sora tables and 13,793 workbook rows.

Changing any prerequisite requires an explicit compatibility decision and new
evidence; execution must not silently regenerate a different content snapshot.

## Terminal outcome

- `starclock-activity` supports the Standard Universe graph, typed state,
  decisions, transactions, RNG, hashing, battle handoff and carry policies;
- a new `starclock-mode-universe` compiles all nine Worlds, nine Paths and 33
  difficulties without owning a second mutable run engine;
- private generated Sora records lower into immutable validated universe domain
  catalogs; normalized JSON and `.xlsx` are never runtime inputs;
- spatial-free domains express mandatory, optional, sequential and grouped
  monster encounters, services, choices, rewards, interactables and exits;
- Blessings, enhancements, Resonances/Formations, Curios/states, Occurrences,
  Cosmic Fragments, services, Ability Tree effects and encounter pools execute;
- every battle uses an immutable verified `BattleSpec` and declared result
  projection; battle code never mutates run state or grants run rewards;
- deterministic baseline AI can complete seeded runs and external callers can
  control the same offered decisions through the agent API and MCP;
- canonical activity replay detects command, bundle, RNG, nested battle and
  post-command hash divergence;
- all runtime dispositions, fixtures, full-run goldens, performance workloads,
  cross-platform checks and release evidence pass from a clean checkout.

## Non-goals

- 3D scenes, coordinates, movement, collision, patrol, aggro radius, rendering,
  animation, audio or UI;
- Swarm Disaster, Gold and Gears, Unknowable Domain, Divergent Universe,
  historical revisions or temporary variants;
- account rewards, weekly points, achievements, inventory persistence, gacha,
  story/dialogue playback or assets;
- Adventure/minigame physics, aiming or timing simulation; these remain offered
  `ExternalOutcome` decisions;
- generalized network save synchronization, matchmaking or anti-cheat service;
- a universe-specific `apply`, replay, RNG implementation or combat resolver;
- implementing unrelated challenge/event/ForkJoin profiles merely to prove an
  extension seam.

## Architecture invariants

1. `Activity::apply(ActivityCommand)` is the only run mutation boundary.
2. Invalid commands and rejected `BattleResult` values preserve state/hash/RNG.
3. A Standard domain compiles to bounded generic Activity nodes; it is not a
   spatial simulation and does not add a generic `Domain` node kind.
4. A domain hub offers exact stable interaction handles. Mandatory clear state
   gates exit; the caller cannot name an unavailable encounter or reward.
5. Encounter preparation offers validated techniques and normal engagement.
   Initial technique effects enter `BattleSpec` through typed contributions.
6. Activity and battle RNG streams are purpose-derived and independent.
7. Catalogs are immutable and shared. Per-run mutation never clones a complete
   catalog or replays every prior prefix.
8. Generated Sora types, workbook schemas and numeric backends remain private.
9. Mode behavior uses typed Activity operations and Rule IR first. Native
   handlers are static, audited and return ordinary validated operations.
10. `starclock-combat` cannot depend on `starclock-mode-universe` or branch on
    universe content IDs.

## Execution and commit rules

- Execute the earliest unblocked batch only; keep at most one batch InProgress.
- Each completed batch updates the status ledger and is committed atomically.
- Commit subjects use
  `<type>(<scope>): <batch-id> <imperative summary>`.
- Mechanic partitions `G04-P4-M01` through `M15` are separate commits. P0-B3
  freezes the exact record/rule/fixture membership for each partition before
  any partition implementation begins.
- Preserve unrelated user changes and never rewrite prior Goal release evidence
  merely to make a new test pass.
- New dependencies require exact pins, license review and deterministic impact
  notes in the batch that introduces them.
- If execution finds a genuine authoring-data defect, correct/regenerate the
  complete workbook through the pinned Python `openpyxl` adapter and revalidate
  it with Sora 0.3.0. Never patch `.xlsx` cells manually or bypass the frozen
  Goal 03 compatibility decision.
- Handwritten Rust files remain below 1,200 physical lines; split by
  responsibility and avoid convenience `pub use` surfaces.
- Every batch runs focused tests, formatting, linting, source policy and the
  relevant prior release contracts. Phase exits run the full repository gate.

## Delivery phases

### Phase 0 — Contract, audit and vertical-slice plan

| Batch | Deliverable |
|---|---|
| `G04-P0-B1` | Freeze this execution package and runtime design; audit current one-battle Activity, bundle, replay, CLI, agent and MCP boundaries against the Goal 04 terminal outcome. |
| `G04-P0-B2` | Freeze public Activity/mode/catalog interfaces, command/error/event families, canonical codec and RNG/configuration revision migration without exposing generated rows. |
| `G04-P0-B3` | Generate the runtime-disposition manifest for all 2,201 content records, 786 rule bindings and 78 fixtures; assign each to generic Activity IR, battle Rule IR, static native handler, data-only metadata or explicit policy. Freeze the exact P4 mechanic partitions. |
| `G04-P0-B4` | Freeze the first World vertical slice, service-verification workloads, allocation counters, CI matrix, dependency/license baseline and Goal 04 release-contract scaffold. |

### Phase 1 — Universe bundle and domain catalog

| Batch | Deliverable |
|---|---|
| `G04-P1-B1` | Add `starclock-mode-universe`, private isolated-bundle readers and composed combat/universe catalog identity; reject wrong bundle/revision/digest without changing the Goal 01 bundle. |
| `G04-P1-B2` | Lower and validate profile, World, difficulty, topology, room, domain and Activity-binding definitions with stable runtime IDs and complete graph references. |
| `G04-P1-B3` | Lower Paths, Blessings/levels/prerequisites, Resonances/Formations and exact parameter vectors into immutable domain definitions. |
| `G04-P1-B4` | Lower Curios, lifecycle states, charges, repairs, replacements and parameter vectors into immutable domain definitions. |
| `G04-P1-B5` | Lower Occurrences/choices/outcomes, services/currency, shops, Ability Tree DAG/effects, external outcomes and authored policies. |
| `G04-P1-B6` | Lower encounter groups/members/waves/enemy slots, room pools, fixed difficulty bindings and mechanic-rule contributions; close every cross-catalog enemy/rule/evidence reference and publish catalog coverage. |

### Phase 2 — Generic Activity runtime

| Batch | Deliverable |
|---|---|
| `G04-P2-B1` | Replace the one-battle-only flow with validated immutable Activity graphs, bounded nodes/edges/visits, typed terminal outcomes and stable graph identities while retaining the Goal 01 one-battle profile. |
| `G04-P2-B2` | Implement generic Activity/Section/Node/Attempt scopes, typed slots, bounded inventories/counter maps, modifier ownership and explicit reset/carry/snapshot policies. |
| `G04-P2-B3` | Implement typed Activity expressions/operations, ordered option generation, command transactions, events/causes, rollback and explicit `Faulted` settlement. |
| `G04-P2-B4` | Implement labeled Activity RNG streams, stable weighted selection, canonical state encoding/hashing and bounded player/debug read-only views. |
| `G04-P2-B5` | Implement participant/roster locks, domain-attempt state, encounter preparation and immutable pending `BattleSpec` construction without build-field leakage. |
| `G04-P2-B6` | Implement verified `BattleResult` submission, declared metric/state projection, HP/Energy/presence carry, battle defeat and activity transition semantics. |
| `G04-P2-B7` | Extend activity codec/property/replay scaffolding, invalid-command byte-identity tests, RNG perturbation tests and the initial post-vertical-slice performance baseline. |

### Phase 3 — Standard Universe profile systems

| Batch | Deliverable |
|---|---|
| `G04-P3-B1` | Compile run entry, participant lock, World/difficulty selection, Path selection and Ability Tree input into Activity definitions/state. |
| `G04-P3-B2` | Compile World topology into Section/domain micrographs, bounded domain hubs, interaction consumption, route choices and mandatory exit gates. |
| `G04-P3-B3` | Implement encounter-slot resolution, optional/sequential/one-of groups, pre-battle technique decisions, battle start and post-battle reward return without spatial state. |
| `G04-P3-B4` | Implement Blessing offers, rarity/prerequisites, resets, replacement, enhancement, inventory state and typed run/battle contributions. |
| `G04-P3-B5` | Implement Path passive, Resonance energy/action availability and Formation selection/contributions through generic commands and battle rules. |
| `G04-P3-B6` | Implement Curio acquisition, uniqueness, charges, negative/error lifecycle, repair, replacement, teardown and scoped contributions. |
| `G04-P3-B7` | Implement Occurrence decisions/costs/outcomes, Cosmic Fragments, shops, respite, revival, downloader, abstract interactables/external outcomes and typed completion/failure. |

### Phase 4 — Complete mechanic partitions

P0-B3 must replace every row's placeholder membership with frozen stable IDs,
rule IDs and fixture IDs. Each partition exits only when its assigned content is
runtime-executable and covered by end-to-end or semantic fixtures.

| Batch | Partition |
|---|---|
| `G04-P4-M01` | Shared Activity operations, currency, Ability Tree and cross-family policies. |
| `G04-P4-M02` | Preservation Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M03` | Remembrance Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M04` | Nihility Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M05` | Abundance Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M06` | The Hunt Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M07` | Destruction Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M08` | Elation Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M09` | Propagation Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M10` | Erudition Path, Resonance/Formations and all 18 Blessings with enhanced levels. |
| `G04-P4-M11` | Positive/neutral/special Curios and ordinary lifecycle behavior. |
| `G04-P4-M12` | Negative and Error Code Curios, repair/fixed states and replacement behavior. |
| `G04-P4-M13` | Occurrences, conditional choices and all explicitly versioned random-outcome policies. |
| `G04-P4-M14` | Services, shops, enhancement, respite, revival, downloader, roster and abstract interactables. |
| `G04-P4-M15` | Encounter pools, waves, enemy bindings, World/difficulty policies, environment rules and battle-result carry. |
| `G04-P4-B1` | Execute all 78 Goal 03 mechanic-family fixtures against runtime domain values and compare the Sora/reference expectations. |
| `G04-P4-B2` | Prove 2,201/2,201 content and 786/786 rule dispositions are reachable, intentionally metadata-only or explicitly policy-bound; reject unimplemented enabled records and scattered content-ID branches. |

### Phase 5 — Replay, controllers and external interfaces

| Batch | Deliverable |
|---|---|
| `G04-P5-B1` | Implement deterministic baseline Activity controller scoring and automatic nested battle execution using only ordered legal commands. |
| `G04-P5-B2` | Complete versioned Activity replay for full runs, controller diagnostics, nested battle identities, first-divergence reporting and bounded verification. |
| `G04-P5-B3` | Add `starclock universe run`, universe-aware config/coverage diagnostics and replay verification in human/JSON modes. |
| `G04-P5-B4` | Extend `starclock-agent-api` with owned Activity observations, offered action tokens, session settlement and replay export while retaining all battle v1 compatibility. |
| `G04-P5-B5` | Add authorized MCP universe/activity tools and bounded resources over the same session facade; extend stdio/HTTP conformance, idempotency, tenant and quota tests. |

### Phase 6 — Golden runs and release hardening

| Batch | Deliverable |
|---|---|
| `G04-P6-B1` | Run the frozen seeded matrix across all nine Worlds, nine Paths and all 33 constructible difficulties; retain representative complete-run event/decision/hash goldens and failure-path fixtures. |
| `G04-P6-B2` | Complete cross-platform native goldens, stream-isolation perturbations, property/fuzz/malformed replay tests, clean regeneration and prior Goal release compatibility. |
| `G04-P6-B3` | Enforce stable-runner and broad-CI performance/allocation budgets for incremental sessions, full-run replay and concurrent shared-catalog server verification; complete dependency, architecture and security audits. |
| `G04-P6-B4` | Freeze public documentation, runtime coverage, golden/benchmark evidence and the Goal 04 release contract; commit completion only after the clean-worktree verifier passes. |

## Acceptance

### Data and catalog

- the exact Goal 03 bundle and all 49 generated tables load through private
  readers and validated domain conversion;
- 2,201 content rows, 786 rule bindings and 78 fixtures have complete frozen
  runtime dispositions;
- no runtime path reads normalized JSON or Excel;
- all source IDs, canonical decimals, references and cross-catalog enemy keys
  survive lowering without floating-point conversion or fabricated rows.

### Activity and domain behavior

- graph entry/terminal reachability, bounded loops and decision availability
  validate for every compiled profile;
- mandatory/optional/sequential/group encounters and exit gates work without
  coordinates or scene state;
- all accepted commands settle atomically to a decision, battle, terminal or
  typed fault; every rejection preserves state hash and RNG counters;
- every inventory, charge, resource, Path, Formation, Curio state and carry
  field obeys its declared scope and bounds.

### Battle integration

- encounter selection and pre-battle technique preparation produce immutable
  reproducible `BattleSpec` values;
- every returned result verifies complete activity/node/attempt/battle,
  catalogs, spec, seed, projection and final hashes;
- HP, Energy, presence, defeat/revival and mode counters carry only through
  declared projections;
- combat has no reverse dependency or universe-specific branch.

### Determinism and clients

- replay verifies every Activity command and nested battle and detects the
  first command/RNG/bundle/configuration/state divergence;
- perturbing graph/reward/shop/occurrence RNG does not change unrelated battle
  draws or streams;
- baseline controller, CLI, agent API and MCP all consume the same ordered
  offered-command set;
- player-visible observations omit hidden RNG/controller state and remain
  bounded; debug views require the existing explicit authority.

### Performance and release

- immutable catalogs are shared across concurrent runs and no command clones a
  complete catalog or replay prefix;
- versioned stable-runner thresholds are recorded after the vertical slice and
  enforced against regression before release;
- Windows x86-64, Linux x86-64 and macOS ARM64 execute the frozen native goldens
  identically;
- formatting, linting, tests, generated drift, file-size/visibility, dependency,
  security and Goals 01–03 release contracts pass;
- `G04-P6-B4` is committed and the final release verifier passes on a clean
  worktree.

Progress is authoritative only in
[the Goal 04 status ledger](04-standard-universe-runtime-status.md).
