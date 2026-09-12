# Goal 22 — Complete Divergent Universe Runtime

## Objective

Release a complete, deterministic and headless-playable Version 4.4 Divergent
Universe runtime over the frozen Goal 11 reference package and Starclock's
shared Activity, build, combat, replay and adapter boundaries.

The immutable release identity is `divergent-universe-runtime-v1`. Completion
means that the Candidate reference package is promoted through current
production Excel/Sora authoring into private typed catalogs and executable
behavior; Ordinary and Cyclical Extrapolation complete real nested battles;
and CLI, Agent and MCP surfaces drive the same authoritative runtime. Loading
an identity, exposing a catalog row, passing a reference-review fixture or
attaching a no-op rule does not implement gameplay.

## Starting point

`G22-P0-B2` maintains current input identities and denominators. Git history
records the planning and execution starting points; the working tree does not
retain launch snapshots or completed-goal receipts as runtime evidence.

The current tree provides:

- the Version 4.4 Candidate reference package, including source-only records
  whose selector proof and executable behavior remain incomplete;
- three isolated `openpyxl` workbooks, 81 Sora tables, generated readers and a
  binary reference bundle;
- 6,762 exact-once source/content obligations across 51 categories;
- 669 typed reference-only mechanic rules, 25 semantic families and 25
  research gaps;
- 54 explicit `ProjectPolicy` source boundaries with replacement conditions;
- the shared Standard Universe, Swarm Disaster and Gold and Gears Activity,
  battle assembly, Rule IR, controller, replay and adapter infrastructure; and
- the current Sora 0.6.1 production toolchain.

The reference package is a factual prerequisite, not a runtime implementation.
Its current bundle contains 28,732 rows, including 547 source-only Persona
obligations (78 Researched and 469 Cataloged), all explicitly Unimplemented.
The reference layer grants no execution credit to its 669 mechanic rules.
Its 6,215 `DataReady` obligations mean
that an obligation ended in an exact fact or an explicit policy/exclusion
boundary. It does not mean that the corresponding behavior is executable.

The current input dimensions are:

| Dimension | Current reference input |
|---|---:|
| Source/content obligations | 6,762 |
| Manifest categories | 51 |
| Normalized/Sora families | 81 |
| Generated rows | 28,732 |
| DataReady / source-only obligations | 6,215 / 547 |
| Mechanic programs | 669 |
| Semantic fixture families | 25 |
| Research gaps | 25 |
| Policy-source boundaries | 54 |
| Profiles/modules/entries/finish conditions | 17 |
| Areas / difficulties / layers | 28 / 22 / 11 |
| Arithmetic Mapping obligations | 258 |
| Equations | 80 |
| Divergent Blessings | 414 |
| Current Curio mode copies | 235 |
| Current Grand Miracle/Hex definitions | 17 |
| Titan types / Boons / talent levels | 12 / 84 / 36 |

`G22-P0-B2` must regenerate these values from current authoritative inputs and
fail on unexplained drift. This document does not authorize shrinking a
denominator to fit the implementation.

### Sora 0.6.1 promotion prerequisite

Goal 11's completed reference package was authored and generated with Sora
0.3.0. The repository's sole current schema, validation, code-generation and
export authority is Sora 0.6.1.

`G22-P0-B1` therefore owns a focused promotion of the Divergent Universe
project, schemas, generator, authoring verifier and all generated artifacts to
Sora 0.6.1. It must regenerate clean targets, compile and load every reader,
prove deterministic drift and update the current reference-package facts and
outputs together. Git history records the superseded 0.3.0 generation; no Goal
22 gate may load or fall back to it.

## Terminal outcome

Goal 22 is complete only when all of the following are true:

- every current source/content obligation has exactly one reviewed runtime
  disposition;
- all 669 mechanic programs are executable through typed Activity or Rule IR,
  proven mechanically inert/excluded, or admitted through a bounded static
  handler audit;
- all executable programs have production construction and execution evidence,
  not merely reader, parser or catalog coverage;
- all 25 semantic fixture families execute against production-lowered data;
- all 25 research gaps and 54 policy sources finish as exact evidence,
  executable `VersionedProjectPolicy` behavior or proven non-runtime scope;
- Ordinary and Cyclical Extrapolation complete through real nested battles;
- Arithmetic Mapping, Equations, Blessings, Curios, Weighted Curios, Grand
  Miracles/Hex, Golden Blood's Boons, Titan talents, Threshold Protocol,
  Astronomical Division, workbench, services and run progression execute at
  their truthful Activity or battle boundary;
- the generated legal matrix covers every assigned content family, mechanic
  partition, policy boundary and terminal path;
- rejected commands preserve authoritative Activity/Battle bytes, hashes and
  RNG counters;
- fresh replay reconstruction and CLI/Agent/MCP surface parity pass; and
- generated-data drift, dependency, native CI, performance and clean-checkout
  release gates pass.

The terminal state is `Released`, not `Candidate`, `ReferenceOnly`,
`CatalogOnly`, `IdentityOnly`, `FixtureMetadata` or `PolicyPending`.

## Non-goals

- game UI, map rendering, localization rendering or presentation asset
  integration;
- story dialogue, audio, cinematics, collection screens, account rewards,
  weekly payouts, achievements or calendar scheduling;
- historical Divergent Universe modules or compatibility with superseded
  intermediate Goal 22 inputs;
- support for unreleased, preview, beta, leaked or NDA-bound content;
- inventing membership from `Tourn` prefixes, names, adjacent IDs or a shared
  source table;
- changing Standard Universe, Swarm Disaster or Gold and Gears semantics to
  make a Divergent Universe row convenient;
- a Divergent Universe-specific command processor, combat state machine,
  replay format, RNG implementation or numeric engine;
- runtime loading of normalized JSON, source caches or Excel; and
- global mutable registration or general-purpose runtime scripting.

## Architecture contract

Ownership follows `docs/06-rust-architecture.md`:

| Responsibility | Owner |
|---|---|
| One battle, formulas, timeline, effects, triggers and battle RNG | `starclock-combat` |
| Character progression, temporary build mapping and immutable combatant compilation | `starclock-build` |
| Cross-battle graph, state, decisions, inventory, carry and settlement | `starclock-activity` |
| Production Sora readers and private immutable lowering | `starclock-data` |
| Divergent Universe definitions, policies, profile and mode operations | `starclock-mode-universe` |
| Exceptional bounded handlers, if any survive audit | `starclock-rules` plus a mode-owned static bundle |
| Baseline decisions over offered commands | `starclock-ai` |
| CLI, Agent, MCP and future presentation projection | adapter crates |

The production path is:

```text
Divergent Universe .xlsx
        |
        v
Sora 0.6.1 bundle and generated readers
        |
        v
starclock-data private lowering
        |
        v
immutable Divergent Universe catalogs and typed programs
        |
        v
Activity command -> contribution snapshot -> BattleSpec
        |                                    |
        |                                    v
        |                              combat commands
        |                                    |
        v                                    v
Activity settlement <- verified BattleResult <- battle terminal
```

The mode must extend the existing Universe facade and generic Activity graph.
It must not add `DivergentUniverseActivity::apply`, query account state from
combat, mutate a live battle from Activity, or hide mode IDs in shared combat
resolver branches.

## Runtime disposition contract

`G22-P0-B3` generates machine-readable exact-once disposition and partition
manifests. Generated manifests are authoritative; handwritten counts are
informative checks only.

Each source/content obligation receives exactly one runtime disposition:

- `ExactIntegrated`: executable production input backed by exact released
  evidence;
- `PolicyIntegrated`: executable production input governed by a named
  `VersionedProjectPolicy`;
- `SharedIntegrated`: executable through a proven shared Starclock identity;
- `ExternalOutcome`: a typed result supplied at an explicit Activity decision
  boundary;
- `MetadataOnly`: mechanically inert identity or validation data;
- `Excluded`: presentation, account, historical or other-mode data with a
  named reason; or
- `Blocked`: temporary execution state, forbidden at release.

Each mechanic program receives exactly one execution disposition:

- `ExactRuleIr` or `ExactActivityProgram`;
- `PolicyRuleIr` or `PolicyActivityProgram`;
- `StaticHandler`, admitted only by the bounded-handler audit;
- `MetadataOnly` or `Excluded`, with evidence that it cannot affect a legal
  run; or
- `Pending`, forbidden at release.

Every executable disposition names its catalog batch, execution partition,
fixture, runtime owner, trigger, state lifetime, snapshot policy, accuracy and
replacement condition where applicable. Reader loading, source identity and a
successful no-op are not execution dispositions.

## Evidence and uncertainty policy

Research follows the repository evidence order:

1. pinned released structured rows and configuration programs;
2. official released text;
3. reproducible public observations; and
4. independent public cross-checks.

The reference package deliberately preserves missing selectors, weights,
timing and lifecycle facts. An implementation batch must first search the
pinned source closure and then perform bounded released/public research. When
the evidence remains unavailable, the batch may implement a deterministic
`VersionedProjectPolicy`, but it must record:

- the exact unavailable fact and known constraints;
- selected behavior and rejected alternatives;
- stable candidate ordering, rounding, limits and labeled RNG stream;
- affected stable IDs, fixtures, matrix entries and authoritative state;
- confidence and explicit non-parity wording; and
- a concrete released-evidence replacement condition.

Missing membership is stricter than a missing numeric value. The 848 Tourn2
room candidates, 618 weekly display-pool bindings, 286 Curio groups, 126
Gamble groups, 97 absent Occurrence graphs and missing service graphs must not
be relabeled exact. A playable baseline may select a bounded reviewed subset
only through explicit `PolicyIntegrated` rows. Those rows must preserve their
candidate provenance, must not alter Goal 11 ownership facts, and must be
replaceable independently when a released selector appears.

The inherited policy work is grouped into these 25 semantic families:

1. module entry, area/layer flow, room selection, carry and finish;
2. Ordinary versus Cyclical selection and reset behavior;
3. temporary character mapping eligibility and refresh;
4. Trace, Light Cone and Relic field-wise preservation;
5. Equation candidate generation and reroll;
6. Equation acquisition, replacement, discard and no-candidate behavior;
7. Equation progress, expansion and ownership refresh;
8. Blessing offer membership and weighting;
9. Blessing enhancement, rewrite and simultaneous contribution ordering;
10. Curio offer membership and weighting;
11. Curio charge, destruction, repair and replacement lifecycle;
12. Weighted Curio eligibility and selection;
13. Grand Miracle/Hex effect and teardown behavior;
14. Titan type and Golden Blood's Boon offers;
15. Titan talent activation, stacking and probability boundaries;
16. Threshold Protocol and Astronomical Division progression;
17. Star-Pioneer, Practice and Cognoculus retention boundaries;
18. permanent-talent prerequisite direction and unlock consumers;
19. weekly modifier, room-mark and cyclical refresh behavior;
20. workbench prices, input/output selection and failure behavior;
21. gamble and Curse Chest membership, weights and fallback;
22. Occurrence graph, choice and settlement behavior;
23. service NPC and Adventure outcome boundaries;
24. encounter, wave, boss and difficulty reachability; and
25. hidden target, simultaneous ordering, cap, rounding and empty-pool
    fallbacks shared by the preceding families.

`G22-P0-B3` must replace this descriptive grouping with generated exact owners
for all current gap and policy rows. No policy may finish merely inherited or
assigned.

## Configuration authoring contract

- `config/divergent-universe/data/*.xlsx` is the editable production authoring
  surface after the Phase 0 promotion.
- Use the documented Python `openpyxl` authoring path and generate complete
  clean targets without overwriting designer changes implicitly.
- Sora 0.6.1 is the sole schema, validation, code-generation and export
  authority.
- Never hand-edit generated Rust, schema locks, templates, debug exports or
  the binary `.sora` bundle.
- Preserve canonical decimal strings, stable semantic order and row-level
  source attribution.
- Runtime code reads only the production Sora bundle through generated
  readers; normalized JSON and source programs remain authoring/review inputs.
- Raw configuration programs may be transformed into typed authored rows by a
  deterministic generator, but bulk proprietary program dumps are not
  committed.
- Schema, workbook, reader, debug export, bundle, private lowering and drift
  tests travel together in the owning batch.

## Partition rules

The exact partition ledger is generated by `G22-P0-B3`. It must:

- cap each generated mechanic execution partition at 64 programs;
- keep one source program and all dependent operations in one partition;
- group by truthful Activity, build or battle ownership rather than filename
  ranges;
- keep one Equation, Curio, Titan or boss family whole when its lifecycle
  requires a smaller dedicated partition;
- keep cross-battle and battle-visible programs in separate partitions;
- bind production lowering, execution fixtures, coverage and policy updates in
  the same partition;
- order partitions by dependency closure and then stable source identity; and
- regenerate ledgers and progress files instead of hand-editing them.

The arithmetic lower bound is eleven partitions for 669 programs at the
64-program cap. Runtime ownership and lifecycle grouping may increase the
count; Phase 0 freezes the actual value before broad implementation.

## Delivery phases

### Phase 0 — Foundation, production promotion and release contract

| Batch | Deliverable |
|---|---|
| `G22-P0-B1` | Promote the isolated Divergent Universe project, schemas, authoring generator, verifiers and generated artifacts from historical Sora 0.3.0 output to current Sora 0.6.1; compile/load every reader and prove deterministic drift. |
| `G22-P0-B2` | Verify current reference inputs, pinned source revisions, schema/bundle identities and all current denominators; distinguish source-closure and catalog checks from independently proven executable behavior. Preserve user work and keep project history solely in Git. |
| `G22-P0-B3` | Generate exact runtime dispositions, mechanic-program partitions and an ordered batch ledger for every obligation, program, semantic family, gap and policy source. |
| `G22-P0-B4` | Freeze public runtime/API boundaries, component identities, Activity slots/scopes, decisions, BattleSpec/Result handoff, save/load snapshot semantics, handler admission and deterministic fault behavior. |
| `G22-P0-B5` | Generate the legal seeded matrix, first vertical slice, policy owners, replay component set, performance workloads and native CI expectations. |
| `G22-P0-B6` | Add status and verification scaffolding and prove that every later batch has prerequisites, focused gates, owned files and terminal evidence targets. |

### Phase 1 — Complete private catalog lowering

| Batch | Deliverable |
|---|---|
| `G22-P1-B1` | Privately load and validate all production Sora tables and rows; bind schema, configuration, content, source and component digests. |
| `G22-P1-B2` | Lower profiles, module, entries, finish conditions, 28 areas, 22 difficulties, 11 layers, Ordinary/Cyclical identities, flow and carry/reset records. |
| `G22-P1-B3` | Lower Arithmetic Mapping eligibility, role/build joins, temporary build patches and mapping lifecycles without querying live account data from Activity or combat. |
| `G22-P1-B4` | Lower all Equation, recipe, progress, expansion, keyword, Blessing, enhancement, rewrite, group and Equation-contribution definitions. |
| `G22-P1-B5` | Lower Curio, Weighted Curio, Grand Miracle/Hex, Titan, Boon, talent, Protocol, Division and progression definitions. |
| `G22-P1-B6` | Lower currencies, workbench/gamble/service/Occurrence families, candidate encounter closure, enemy slots and all 669 mechanic-program identities with complete source attribution. |

### Phase 2 — Shared capability closure

| Batch | Deliverable |
|---|---|
| `G22-P2-B1` | Inventory every mechanic expression, selector, trigger, operation, state and lifecycle shape; map each to existing shared support or a named missing capability. |
| `G22-P2-B2` | Add only generic Activity operations, conditions, inventories, offers, scoped state and lifecycle semantics required by multiple Divergent programs. |
| `G22-P2-B3` | Add only generic combat selectors, expressions, effects, triggers, operations and result projections required by the program inventory. |
| `G22-P2-B4` | Extend `starclock-build` only where field-wise temporary mapping cannot be represented by the current generic contribution compiler. |
| `G22-P2-B5` | Execute shared capability probes, audit mode/content-ID branches and handler metadata, and freeze remaining program partitions; the default admitted native-handler count is zero. |

### Phase 3 — Entry, flow, progression and Arithmetic Mapping

| Batch | Deliverable |
|---|---|
| `G22-P3-B1` | Execute Ordinary and Cyclical entry, area/layer flow, bounded room selection, finish conditions and explicit carry/reset behavior. |
| `G22-P3-B2` | Execute difficulty, Threshold Protocol, Astronomical Division, Star-Pioneer, Practice, Cognoculus and cyclical refresh state. |
| `G22-P3-B3` | Execute run currencies, persistent mechanical progression and immutable save/loadout snapshots without mutating account state. |
| `G22-P3-B4` | Compile and refresh Arithmetic Mapping field by field, preserving already-sufficient caller builds and tearing temporary contributions down at the declared boundary. |
| `G22-P3-B5` | Prove logical Run/Plane/Node/Battle scopes, party-change refresh, transition ordering and accepted/rejected state identity. |
| `G22-P3-B6` | Execute the first production Ordinary vertical slice through entry, mapping, rewards, one real battle, a later layer and terminal settlement. |

### Phase 4 — Equations and Divergent Blessings

| Batch | Deliverable |
|---|---|
| `G22-P4-B1` | Execute Equation offers, eligibility, reroll, acquisition, replacement, discard and no-legal-candidate behavior. |
| `G22-P4-B2` | Execute Path-count recipes, ownership contribution refresh, progress thresholds and expansion state transitions. |
| `G22-P4-B3` | Compile Equation keyword/effect programs and expansion contributions into immutable battle snapshots. |
| `G22-P4-B4` | Execute Blessing offers, acquisition, enhancement, rewrite, replacement and group closure over the exact mode-owned identities. |
| `G22-P4-B5` | Execute Blessing and Equation interactions, simultaneous ownership changes, battle effects and teardown without importing unproven shared content. |
| `G22-P4-B6` | Prove offer RNG isolation, stable ordering, caps, empty pools, rejected mutations and fresh reconstruction of Equation/Blessing state. |

### Phase 5 — Curios, Titan systems and transformations

| Batch | Deliverable |
|---|---|
| `G22-P5-B1` | Execute Curio acquisition, charges, activation, destruction, repair, replacement and contribution teardown. |
| `G22-P5-B2` | Execute Weighted Curio, Gamble and Grand Miracle/Hex eligibility and lifecycle through explicit exact or policy-backed candidate sets. |
| `G22-P5-B3` | Execute all Titan types, Golden Blood's Boon levels/offers and permanent talent prerequisites, costs, stacking and contributions. |
| `G22-P5-B4` | Execute permanent/weekly mechanical progression, unlock consumers, room marks and declared refresh/carry boundaries. |
| `G22-P5-B5` | Execute workbench transformations, prices, input/output selection, Curse Chest operations and failure-without-mutation behavior. |
| `G22-P5-B6` | Materialize one immutable contribution snapshot binding Mapping, difficulty, Protocol, Equations, Blessings, Curios, Titan and progression state into battle identity. |

### Phase 6 — Occurrences, services, encounters and battle programs

| Batch | Deliverable |
|---|---|
| `G22-P6-B1` | Execute Occurrence variants, choices, costs and outcomes; keep unavailable minigames or dialogue interactions at explicit typed external-result boundaries. |
| `G22-P6-B2` | Execute service NPCs, Adventure settlements, shops, workbench/gamble entry and empty/missing-graph fallbacks. |
| `G22-P6-B3` | Resolve room, stage, weekly display, encounter, wave, enemy and boss reachability through exact or explicitly policy-backed production rows. |
| `G22-P6-B4` | Build concrete immutable `BattleSpec` values from current participants, contribution snapshot, encounter, difficulty and policy identity; validate every assembled battle. |
| `G22-P6-B5` | Execute atomic battle-result settlement, rewards, progression, carry, retry/failure and the next-node transition. |
| `G22-P6-B6` | Prove stale/rejected assembly and settlement preserve state, RNG and cache semantics; reconstruct transition battles from fresh inputs. |
| `G22-P6-Axx` | Generated ordered partitions for every cross-battle mechanic program, each with production lowering, execution fixtures and exact-once receipts. |
| `G22-P6-Mxx` | Generated ordered partitions for every battle-visible or battle-boundary mechanic program, including boss-family fixtures and exact-once receipts. |

### Phase 7 — Complete runs, replay and adapters

| Batch | Deliverable |
|---|---|
| `G22-P7-B1` | Implement a deterministic baseline controller that chooses only offered Activity and battle commands and completes legal Ordinary and Cyclical runs. |
| `G22-P7-B2` | Add `universe run --mode divergent-universe`, configuration validation/coverage and replay export/verify to the CLI. |
| `G22-P7-B3` | Expose bounded Divergent Universe manifests, sessions, observations and offered actions through Agent API without generated rows or private-state leakage. |
| `G22-P7-B4` | Expose the same authoritative sessions through MCP and verify authorization, idempotency, cancellation and bounded event pagination. |
| `G22-P7-B5` | Implement component-addressed fresh replay reconstruction and first-divergence reporting across catalog, Activity, Mapping, contribution snapshot, battle assembly, commands and settlement. |
| `G22-P7-B6` | Execute the generated legal matrix across both run families, assigned difficulties/content systems, mechanic partitions, policies and terminals. |

### Phase 8 — Hardening and release

| Batch | Deliverable |
|---|---|
| `G22-P8-B1` | Add malformed-input, stale-command, property, RNG-isolation, empty-pool, overflow, recursion-budget, save/load and replay-corruption suites. |
| `G22-P8-B2` | Freeze catalog-load, cold/warm assembly, full-run, replay, trigger-heavy, policy-heavy, concurrent-session and invalid-command performance/allocation workloads. |
| `G22-P8-B3` | Run dependency/license, architecture, unsafe, generated drift, workbook/Sora, provenance, handler and prior-release isolation audits. |
| `G22-P8-B4` | Close exact-once runtime coverage for every current obligation, mechanic program, semantic family, gap and policy source; no pending executable item may remain. |
| `G22-P8-B5` | Run native CI and fresh clean-checkout acceptance, update current state summaries and publish `Released` only after every gate passes. |

## First vertical slice

`G22-P0-B5` selects the exact Ordinary entry, area, difficulty, layer sequence,
party snapshot, encounter policy and seed after validating their joins. The
slice must include:

- entry and initial run state;
- Arithmetic Mapping that changes at least one below-threshold build field and
  preserves an already-sufficient field;
- one Equation acquisition and one ownership-driven progress transition;
- one Blessing acquisition or enhancement contributing to that Equation;
- one Curio or Titan contribution whose effect changes authoritative state;
- one real assembled nested battle whose snapshot differs from a control;
- battle-result settlement and movement into a later layer;
- a workbench, service or Occurrence decision; and
- terminal completion plus fresh replay verification from production inputs.

The slice proves the end-to-end architecture only. It cannot close whole-family,
program, policy or matrix coverage.

## Matrix and coverage requirements

The matrix generator produces a bounded axis-covering legal set rather than a
Cartesian product. At minimum it assigns:

- both Ordinary and Cyclical entries and every finish-condition family;
- all 28 areas, 22 difficulties and 11 selected layers;
- every exact or policy-selected room/encounter family and every reachability
  exclusion boundary;
- all Arithmetic Mapping eligibility and lifecycle shapes, including the four
  unresolved source-avatar locators;
- all 80 Equations and every category, recipe, progress and expansion shape;
- all 414 Blessings, enhancement/rewrite shapes and Path contribution types;
- all 235 current Curio copies, every lifecycle state, Weighted Curio consumer
  and all 17 current Grand Miracle/Hex definitions;
- all 12 Titan types, 84 Boons and 36 talent levels;
- all Protocol/Division/progression, workbench, gamble, service, Occurrence,
  Adventure and terminal families;
- every generated mechanic-program partition and semantic fixture family;
- every policy source and replacement-trigger test; and
- every accepted/rejected, empty-pool, fault, save/load and battle settlement
  boundary.

The matrix records why each entry exists and which obligations it covers. An
entry may be removed only when regenerated coverage proves that another entry
closes the same obligations.

## Replay identity

A fresh Divergent Universe replay binds at least:

- production schema/configuration/content/source digests;
- Activity definition and mode profile identity;
- Ordinary/Cyclical entry, area, difficulty, layer and policy selections;
- initial participant and caller-owned build snapshot digests;
- Arithmetic Mapping definition and selected temporary contribution digest;
- Equation, Blessing, Curio, Titan, Protocol and progression catalogs;
- every accepted Activity decision and resulting state/event hash;
- each immutable contribution snapshot and `BattleSpec` digest;
- every battle command, event/state hash and terminal `BattleResult`; and
- settlement, next-node and final-run hashes.

Replay verifies only current-tree inputs. No compatibility decoder or revision
branch is added for intermediate Goal 22 formats.

## Verification ladder

Every batch runs the narrowest applicable checks and records exact commands:

1. generator, manifest, schema and source-attribution checks owned by changed
   inputs;
2. workbook semantic and visual QA when `.xlsx` inputs change;
3. Sora 0.6.1 check/build/export/load and generated drift checks when
   configuration changes;
4. focused `cargo test -p <affected-package>`;
5. `cargo fmt --all -- --check` and package-scoped Clippy with warnings denied;
6. cross-crate integration tests when a shared boundary changes;
7. generated disposition, partition, policy and matrix coverage checks; and
8. `cargo test --workspace` at shared-boundary and release checkpoints.

Before release, run the explicit exhaustive suites, stable-runner performance
workloads, hosted native matrix and fresh clean-checkout acceptance. No test may
depend on wall clock, filesystem order, unseeded randomness or collection
iteration order.

## Execution and commit rules

- Start with the earliest unblocked batch and keep only one Goal 22 batch
  `InProgress` per worktree.
- Do not begin broad implementation before `G22-P0-B3` freezes generated
  denominators and partitions.
- Each batch owns its production data, lowering, behavior, tests, coverage and
  documentation as one responsibility-bounded change.
- Inspect the relevant design, current implementation and assigned source rows
  before editing.
- Use a separate branch/worktree and isolated artifact paths for concurrent
  Goal work.
- Stage, commit, push or create branches only when the user explicitly
  authorizes those Git actions. When authorized, use
  `<type>(divergent-universe): <batch-id> <imperative summary>`.
- Never lower a coverage denominator, promote a candidate identity to exact,
  convert an executable record to metadata or weaken a fixture merely to make
  a gate pass.
- A blocked evidence question triggers bounded released/public research and
  then an explicit replaceable policy when permitted; it never justifies a
  silent no-op.
- Update current code, data, tests, generated artifacts and state summaries
  together when their facts change.

## Acceptance

- Goal 11 inputs and all current denominators regenerate without unexplained
  drift.
- The Divergent Universe project validates, generates, exports and loads only
  through Sora 0.6.1.
- Production runtime loads no JSON, workbook or raw source program.
- Every current source/content obligation and all 669 mechanic programs have
  exact-once terminal runtime dispositions with no `Blocked`, `Pending`,
  `CatalogOnly` or `IdentityOnly` executable item.
- All 25 semantic fixture families execute against production-lowered data.
- All 25 gaps and 54 policy sources are exact, executable versioned policies
  or proven non-runtime scope, with replacement tests and visible non-parity
  wording.
- Complete Ordinary and Cyclical runs cross real battles and pass the generated
  legal matrix.
- Mapping, Equation, Blessing, Curio, Titan, Protocol, transformation,
  service, encounter and battle behavior changes authoritative state/events in
  production fixtures.
- Rejections are byte-, hash- and RNG-inert; deterministic faults use the
  documented terminal path.
- CLI, Agent and MCP expose the same offered-command semantics, and fresh
  replay verifies independently from the live session.
- No content ID enters shared resolver branches and no native handler is added
  without a reviewed static-registry admission record.
- Performance, dependency/license, provenance, Sora/workbook drift, native CI,
  prior-release isolation and clean-checkout gates pass.
- `docs/state.md` and `policy/state.json` claim only behavior proven by final
  generated release evidence.

## Goal-mode launch objective

Use the following objective when Goal 22 execution starts:

> Implement and release the complete Version 4.4 Divergent Universe runtime
> defined by `docs/goals/22-divergent-universe-runtime.md`. Begin at
> `G22-P0-B1`, proceed in dependency order, and continue until every current
> source/content obligation, mechanic program, semantic family, research gap
> and policy source has a terminal runtime disposition and both Ordinary and
> Cyclical complete-run release gates pass. Prefer pinned released evidence,
> then official released text and reproducible public observations; when
> evidence remains unavailable, implement and label a deterministic replaceable
> `VersionedProjectPolicy` without promoting candidate membership to exact.
> Use `openpyxl` for production workbook authoring and Sora 0.6.1 for
> validation, code generation and export. Never treat IDs, catalog loading,
> reference-review fixtures or no-op handlers as completed mechanics.
