# Goal 21 — Complete Currency Wars Runtime

## Objective

Release a complete, deterministic and headless-playable Version 4.4 Currency
Wars runtime over the frozen Goal 12 reference package and Starclock's shared
Activity, build, combat, replay and adapter boundaries.

The immutable release identity is `currency-wars-runtime-v1`. Completion means
that source records and configuration programs are privately lowered from the
production Sora bundle into executable typed behavior, complete Standard and
Overclock runs cross real nested battles, and CLI, Agent and MCP surfaces drive
the same authoritative runtime. Loading an ID, exposing a catalog row or
recording an investment identity does not count as implementing its behavior.

## Starting point

The direct implementation baseline is commit
`a139bfc76e4bd7b260ee934e30e6c12ad5a62a31`. It provides:

- the `starclock-mode-currency-wars` crate;
- private production Sora loading through `starclock-data`;
- route progression and battle handoff;
- Gold, Experience, team level and Squad HP state;
- shop refresh, purchase, sale, star combination, deployment and Bond
  recomputation;
- ID-only investment selection and debug/CLI inspection; and
- generic atomic replacement operations for Activity counter maps and ordered
  ID sets.

This is a vertical runtime skeleton, not the Goal 21 release. It does not yet
lower the complete configuration-program closure, assemble exact Currency Wars
battles, execute investment effects, resolve role builds/equipment, implement
all battle overrides or provide complete-run replay and adapter parity.

Goal 12 remains the factual prerequisite. Its current-tree denominators are:

| Dimension | Frozen current input |
|---|---:|
| Source obligations | 19,250 |
| Currency Wars obligations | 18,524 |
| Evidence-only obligations | 726 |
| Normalized families | 102 |
| Production Sora tables | 102 |
| Authored/exported rows | 74,850 |
| Mechanic programs | 2,367 |
| Battle-visible or battle-boundary programs | 1,846 |
| Cross-battle Activity programs | 521 |
| Semantic fixture families | 28 |
| Explicit policy gaps | 12 |
| Routes / nodes | 26 / 493 |
| Difficulty records / roster roles / Bonds | 97 / 77 / 49 |
| Investment identities | 834 |

`G21-P0-B2` must regenerate these values from current authoritative inputs and
fail if they drift. This document does not authorize shrinking a denominator
to match the implementation.

### Sora 0.6.1 prerequisite

Goal 21 validates, generates and exports with exactly `sora 0.6.1`. The host
binary reports that version, but the starting repository still pins Sora 0.3.0
and its Currency Wars project manifest uses the older format. A direct 0.6.1
check rejects that manifest because the required root `project` declaration is
absent.

`G21-P0-B1` therefore owns an intentional repository toolchain migration. It
must update the current tool policy, checksum/install path, capability lock,
project manifests, generators and applicable verifiers; regenerate affected
current outputs; compile generated readers; and prove deterministic drift
under 0.6.1 before any runtime implementation batch begins. Historical Goal
receipts may describe their original 0.3.0 execution, but no Goal 21 gate may
fall back to 0.3.0.

## Terminal outcome

Goal 21 is complete only when all of the following are true:

- every one of the 19,250 source obligations has exactly one reviewed runtime
  disposition;
- all 2,367 mechanic programs are either executable through typed Activity or
  Rule IR, proven metadata-only/excluded, or assigned to an explicitly audited
  bounded static handler;
- every executable program has production-backed construction and execution
  evidence, not only parser or catalog coverage;
- the 28 semantic fixture families execute against production-lowered data;
- all 12 policy gaps have a terminal `ExactEvidence` or
  `VersionedProjectPolicy` implementation with a replacement condition;
- Standard and Overclock runs complete through real nested battles for the
  generated legal matrix;
- every route, difficulty, role, Bond threshold, investment family, encounter
  family and terminal boundary is covered exactly as assigned by the generated
  matrix and disposition manifests;
- rejected commands preserve authoritative Activity/Battle bytes, hashes and
  RNG counters;
- fresh replay reconstruction and CLI/Agent/MCP surface parity pass; and
- generated-data drift, dependency, native CI, performance and clean-checkout
  release gates pass.

The terminal state is `Released`, not `Candidate`, `CatalogOnly`,
`IdentityOnly`, `FixtureMetadata` or `PolicyPending`.

## Non-goals

- game UI, presentation adapters, localization rendering or ID dereferencing
  for display;
- story dialogue, assets, audio, collection screens, account rewards,
  achievements or rank payouts;
- support for unreleased, preview, beta, leaked or NDA-bound content;
- changing another mode's semantics merely because it uses a similar source
  table;
- a Currency Wars-specific command processor, battle state machine, replay
  format, RNG implementation or formula engine;
- runtime loading of normalized JSON, raw configuration files or Excel;
- global mutable registration or general-purpose runtime scripting; and
- compatibility migrations for superseded intermediate Goal 21 formats.

## Architecture contract

Ownership follows `docs/06-rust-architecture.md`:

| Responsibility | Owner |
|---|---|
| One battle, formulas, timeline, effects, triggers and battle RNG | `starclock-combat` |
| Character progression, trial/owned build and equipment compilation | `starclock-build` |
| Cross-battle graph, state, decisions, inventory, carry and settlement | `starclock-activity` |
| Production Sora readers and private immutable lowering | `starclock-data` |
| Currency Wars definitions, policies, profile and mode operations | `starclock-mode-currency-wars` |
| Exceptional bounded handlers, if any survive the zero-handler audit | `starclock-rules` plus a mode-owned static bundle |
| CLI, Agent, MCP and future UI projection | adapter crates |

The production path is:

```text
Currency Wars .xlsx
        |
        v
Sora 0.6.1 bundle and generated readers
        |
        v
starclock-data private lowering
        |
        v
immutable Currency Wars catalogs and typed programs
        |
        v
Activity command -> current contribution snapshot -> BattleSpec
        |                                            |
        |                                            v
        |                                      combat commands
        |                                            |
        v                                            v
Activity settlement <- verified BattleResult <- battle terminal
```

The mode crate must not query workbooks, account inventory, raw source caches
or presentation catalogs. A future UI may resolve IDs independently without
changing authoritative state or simulation cost.

## Runtime disposition contract

`G21-P0-B3` generates the machine-readable exact-once disposition and partition
manifests. Handwritten completion totals are informative only; generated
manifests are authoritative.

Each source obligation receives exactly one target disposition:

- `Integrated`: mode-owned executable runtime input;
- `SharedIntegrated`: executable through a proven shared Starclock identity;
- `ExternalOutcome`: a typed result supplied at an explicit Activity decision
  boundary;
- `MetadataOnly`: mechanically inert data retained for validation or identity;
- `Excluded`: presentation, account, story or other-mode content with a named
  reason; or
- `Blocked`: temporary execution state only, forbidden at release.

Each mechanic program receives exactly one execution disposition:

- `ExactRuleIr` or `ExactActivityProgram`;
- `PolicyRuleIr` or `PolicyActivityProgram`;
- `StaticHandler`, admitted only by the handler audit;
- `MetadataOnly` or `Excluded`, with evidence that it cannot affect legal run
  state; or
- `Pending`, forbidden at release.

Catalog loading, successful parsing, retained source identity and a no-op
handler are not execution dispositions. Every executable disposition names its
catalog batch, execution batch, fixture, owner, trigger, state lifetime and
accuracy class.

## Evidence and uncertainty policy

Research follows the repository evidence order:

1. pinned released structured rows and configuration programs;
2. official released text;
3. reproducible live observations;
4. independent public cross-checks.

When a field remains uncertain, the owning batch must search the pinned source
closure first and then perform bounded public research. If no released evidence
resolves it, implement a deterministic `VersionedProjectPolicy` and record:

- known facts and the exact unresolved field;
- selected behavior and rejected alternatives;
- ordering, rounding, candidate set and RNG stream;
- affected source IDs, fixtures and matrix entries;
- confidence and non-parity wording; and
- a concrete replacement condition.

Guessing may unblock implementation, but it may never be labeled observed or
exact. New evidence replaces the current policy directly; no legacy branch is
retained.

The twelve inherited policy fields are:

1. `bond.simultaneous_recompute`;
2. `encounter.boss_identity`;
3. `mechanic.configuration_program`;
4. `flow.carry_reset`;
5. `route.gambit_membership`;
6. `economy.gold_coin_id`;
7. `investment.operation_order`;
8. `star.maximum_overflow`;
9. `economy.offer_sampling_order`;
10. `position.automatic_technique_rescue`;
11. `build.role_to_shared_build`; and
12. `squad_hp.same_boundary_order`.

No policy may finish as merely inherited or assigned. Its owner batch must
produce executable behavior and replacement-trigger tests.

## Configuration authoring contract

- Excel `.xlsx` remains the editable production surface.
- Use the documented Python `openpyxl` authoring path for workbook changes.
- Sora 0.6.1 remains the sole schema, validation, code-generation and export
  authority.
- Never hand-edit generated Rust, schema locks, templates, debug exports or the
  binary bundle.
- Generate complete clean workbook targets; do not patch `.xlsx` ZIP members
  or silently overwrite designer-edited files.
- Preserve canonical decimal strings and semantic order.
- Runtime code reads only the production Sora bundle through generated readers.
- Raw configuration programs may be transformed into typed authored rows by a
  deterministic generator, but raw proprietary program dumps are not committed.
- Schema, workbook, generated reader, debug export, bundle, lowering and drift
  tests travel in the same batch.

## Partition rules

The exact batch denominator is generated in `G21-P0-B3`. Partition generation
must obey these limits:

- at most 64 configuration programs per generated execution partition;
- keep one source program and all of its dependent operations in one partition;
- group by truthful runtime owner and required generic capability, not source
  filename ranges;
- do not mix cross-battle Activity and battle-owned programs;
- keep one boss/phase family whole even when it needs a smaller dedicated
  partition;
- every partition owns production lowering, execution fixtures, coverage and
  policy updates together;
- order partitions by dependency closure, then stable source identity; and
- generated ledgers and progress files are regenerated, never hand-edited.

The initial expected lower bound is 38 partitions for 2,367 programs at the
64-program cap. Dependency and boss-family constraints may increase that
number. `G21-P0-B3` freezes the real count before broad implementation begins.

## Delivery phases

### Phase 0 — Foundation, denominator and release contract

| Batch | Deliverable |
|---|---|
| `G21-P0-B1` | Migrate the current repository toolchain and all applicable Currency Wars project/generator/verifier inputs to pinned Sora 0.6.1; regenerate and compile outputs, prove deterministic drift and forbid Goal 21 fallback to 0.3.0. |
| `G21-P0-B2` | Verify commit `a139bfc7…a31`, Goal 12 inputs, source revisions, clean starting tree and all current denominators. Record the runtime skeleton honestly as partial. |
| `G21-P0-B3` | Generate exact source dispositions, mechanic-program partitions and ordered batch ledger for all 19,250 obligations, 2,367 programs, 28 fixtures and 12 policies. |
| `G21-P0-B4` | Freeze the public runtime/API boundary, component identities, Activity slots/scopes, command decisions, BattleSpec/Result contract, handler admission policy and failure semantics. |
| `G21-P0-B5` | Generate the legal seeded matrix, first vertical slice, policy owners, replay identities, performance workloads and native CI expectations. |
| `G21-P0-B6` | Add verification/status scaffolding and prove every later batch has an owner, prerequisite, focused gate and terminal evidence target. |

### Phase 1 — Complete private catalog lowering

| Batch | Deliverable |
|---|---|
| `G21-P1-B1` | Load and validate all 102 production tables and 74,850 rows through private generated readers; bind schema/config/component digests. |
| `G21-P1-B2` | Lower profile, Gambits, entries, finish conditions, 26 routes, 493 nodes, layers, rooms, domain compositions, carry/reset and rank progression. |
| `G21-P1-B3` | Lower currencies, economy, offers, prices, Experience, team size, positions, roster transactions, star states and lifecycle rules. |
| `G21-P1-B4` | Lower 77 role/build mappings, equipment, off-field conversions, Character Empowerment, 49 Bonds, levels and contributions. |
| `G21-P1-B5` | Lower all 834 investments, formulas, occurrences, services, workbenches and their candidate, eligibility and lifecycle relationships. |
| `G21-P1-B6` | Lower encounter groups, source obligations, waves, enemy slots, affixes, boss pools, battle overrides and configuration-program identities with complete reference closure. |

### Phase 2 — Shared capability closure

| Batch | Deliverable |
|---|---|
| `G21-P2-B1` | Inventory every configuration opcode, expression, selector, trigger, state and lifecycle shape; map each to existing Rule IR/Activity support or a named missing capability. |
| `G21-P2-B2` | Add only the generic Activity operations, conditions, decision types, inventories and lifecycle semantics required by multiple source programs. |
| `G21-P2-B3` | Add only generic combat selectors, expressions, operations, trigger points, effects and settlement projections required by the program inventory. |
| `G21-P2-B4` | Extend `starclock-build` only where exact role, trial build, off-field conversion or equipment compilation cannot be expressed by the current generic compiler. |
| `G21-P2-B5` | Execute shared capability probes, audit content-ID branches and handler metadata, and freeze the remaining generated program partitions. Default admitted native-handler count is zero. |

### Phase 3 — Entry, route, economy and roster execution

| Batch | Deliverable |
|---|---|
| `G21-P3-B1` | Execute Standard/Overclock entry, route membership, three-Plane flow, node decisions, finish conditions and exact carry/reset behavior. |
| `G21-P3-B2` | Execute Squad HP, action-value limits, timeout/victory/loss ordering, checkpoint continuation, recovery and run failure. |
| `G21-P3-B3` | Execute Gold and Experience income/spend, deterministic offer generation, refresh, purchase, sale, refund and empty-candidate behavior. |
| `G21-P3-B4` | Execute roster/bench/field caps, three-copy combination, all star states, maximum-star overflow and teardown. |
| `G21-P3-B5` | Execute team-size leveling, front/back positioning, deployment legality and all same-boundary roster reconciliation. |
| `G21-P3-B6` | Upgrade the first vertical slice to complete one real Standard run through economy, roster mutation, multiple battles and terminal settlement. |

### Phase 4 — Builds, equipment, Bonds and battle overrides

| Batch | Deliverable |
|---|---|
| `G21-P4-B1` | Compile each role to its exact owned/trial resolved build without querying account inventory from combat or Activity. |
| `G21-P4-B2` | Execute equipment eligibility, three-slot replacement/teardown and off-field Eidolon/signature Light Cone conversion. |
| `G21-P4-B3` | Execute field/bench position changes and Character Empowerment activation, refresh and teardown. |
| `G21-P4-B4` | Execute all Bond memberships, thresholds, levels, simultaneous recomputation and Activity/battle contributions. |
| `G21-P4-B5` | Execute automatic Techniques, defeat-Energy scaling, lethal rescue, countdown reduction and remaining battle overrides. |
| `G21-P4-B6` | Materialize one immutable contribution snapshot that binds deployment, builds, equipment, stars, Bonds, investments, difficulty and node state into battle identity. |

### Phase 5 — Investments and cross-battle content

| Batch | Deliverable |
|---|---|
| `G21-P5-B1` | Execute Augment definitions, season membership, selected enhancements, remarks and their offer/replace lifecycle. |
| `G21-P5-B2` | Execute Portal buffs, Orbs, Projections, Talents and associated maze-buff or display contributions. |
| `G21-P5-B3` | Execute investment ordering, stacking, replacement, reroll, eligibility, cross-family interaction and battle contribution snapshots. |
| `G21-P5-B4` | Execute formula identity/recipe/progress/randomization/contribution families; preserve source-proven zero families as zero rather than inventing content. |
| `G21-P5-B5` | Execute Occurrence variants/choices/costs/outcomes and explicit external-result boundaries. |
| `G21-P5-B6` | Execute shops, service offers, workbenches, gamble/curse/Hex/Curio families and proven empty-pool fallbacks without importing unrelated Universe content. |
| `G21-P5-Axx` | Generated ordered partitions for all 521 cross-battle Activity programs. Each partition lowers and executes its assigned source programs with exact-once receipts. |

### Phase 6 — Encounters, battle assembly and battle-visible programs

| Batch | Deliverable |
|---|---|
| `G21-P6-B1` | Resolve every encounter group, wave, enemy slot, elite/boss choice and node/difficulty binding to concrete immutable combat inputs. |
| `G21-P6-B2` | Compile enemy affixes, scaling, stage limits and boss phases through shared combat definitions without mode-ID branches in the resolver. |
| `G21-P6-B3` | Implement the production Currency Wars BattleSpec assembler and bounded cache over the current Activity contribution snapshot. |
| `G21-P6-B4` | Execute battle-result projection, Squad HP/action-value settlement, rewards, carry and the next-node transition atomically. |
| `G21-P6-B5` | Prove rejected/stale assembly and settlement preserve Activity/Battle state, RNG and cache semantics; verify replay reconstruction of transition battles. |
| `G21-P6-Mxx` | Generated ordered partitions for all 1,846 battle-visible or battle-boundary programs, including production execution fixtures and exact-once receipts. |

### Phase 7 — Complete runs, replay and adapters

| Batch | Deliverable |
|---|---|
| `G21-P7-B1` | Implement a deterministic baseline controller that selects only currently offered Activity and battle commands and completes legal runs. |
| `G21-P7-B2` | Add `currency-wars run`, configuration coverage and replay export/verify to the CLI while preserving existing validate/inspect behavior. |
| `G21-P7-B3` | Expose bounded Currency Wars manifests, sessions, observations and actions through Agent API without generated rows or private state leakage. |
| `G21-P7-B4` | Expose the same authoritative session through MCP; verify authorization, idempotency, cancellation and bounded event pagination. |
| `G21-P7-B5` | Implement component-addressed fresh replay reconstruction and first-divergence reporting across catalog, Activity, battle assembly, battle commands and settlement. |
| `G21-P7-B6` | Execute the generated legal matrix, including all assigned routes, difficulties, Gambits, roles, Bonds, investment families, encounters and policy boundaries. |

### Phase 8 — Hardening and release

| Batch | Deliverable |
|---|---|
| `G21-P8-B1` | Add malformed-input, stale-command, property, RNG-isolation, empty-pool, overflow, recursion-budget and replay-corruption suites. |
| `G21-P8-B2` | Freeze catalog-load, cold/warm assembly, full-run, replay, trigger-heavy, concurrent-session and invalid-command performance/allocation workloads. |
| `G21-P8-B3` | Run dependency/license, architecture, unsafe, generated drift, workbook/Sora, provenance, native-handler and prior-release isolation audits. |
| `G21-P8-B4` | Close exact-once runtime coverage for 19,250 obligations, 2,367 programs, 28 fixtures and 12 policies; no pending or identity-only executable item may remain. |
| `G21-P8-B5` | Run native CI and clean-checkout acceptance, update current state/docs, freeze release evidence and register the completion snapshot only after every gate passes. |

## First vertical slice

`G21-P0-B5` selects the exact released route, difficulty, Gambit, roster and
seed after validating their joins. The slice must include:

- entry and initial state;
- at least one shop refresh and purchase;
- a roster combination or explicit proof the selected seed cannot offer one;
- deployment and Bond recomputation;
- one investment activation whose effect changes authoritative state;
- one real assembled nested battle whose contribution differs from a control;
- victory and loss/checkpoint settlement paths;
- progression into a later Plane and final completion; and
- fresh replay verification from immutable production inputs.

The slice proves the end-to-end architecture only. It cannot close content,
program, policy or matrix coverage by itself.

## Matrix and coverage requirements

The matrix generator must produce a bounded legal axis-covering set rather
than a Cartesian product. At minimum it assigns:

- every one of the 26 routes and 97 difficulty records to a valid entry;
- both Gambits and all rank/Overclock boundaries;
- every role, rarity, position kind, team-size boundary and star transition;
- every Bond at each authored threshold and contribution level;
- every investment identity to an execution fixture and every investment
  family to one or more complete runs;
- every encounter group, wave, enemy slot/affix and boss pool;
- every terminal, timeout, Squad HP and action-value boundary;
- every mechanic-program partition and semantic fixture family; and
- every `VersionedProjectPolicy` plus its replacement-trigger test.

The generated matrix records why each entry exists and which obligations it
covers. Removing an entry is legal only when regenerated coverage proves that
another entry closes the same obligations.

## Verification ladder

Every batch runs the narrowest applicable checks and records exact commands.
The minimum ladder is:

1. generator/manifest/schema checks owned by the changed inputs;
2. workbook semantic and visual QA when `.xlsx` changes;
3. Sora check/build/export/load and generated drift checks when configuration
   changes;
4. focused `cargo test -p <affected-package>`;
5. direct Cargo format and Clippy checks for affected packages;
6. cross-crate integration tests when a shared boundary changes;
7. generated partition, policy and matrix coverage checks; and
8. `cargo test --workspace` at shared boundaries and release checkpoints.

Before release, run the explicit exhaustive suites, stable-runner performance
workloads, native hosted matrix and a fresh clean-checkout acceptance. No test
may depend on wall clock, filesystem order, unseeded randomness or hash-map
iteration.

## Execution and commit rules

- Start with the earliest unblocked batch and keep only one Goal 21 batch
  `InProgress` per worktree.
- Do not begin broad implementation before `G21-P0-B3` freezes the generated
  denominator and partitions.
- Each batch owns its production data, lowering, runtime behavior, tests,
  coverage and documentation as one responsibility-bounded change.
- Inspect the relevant design document, current implementation and assigned
  source rows before editing.
- Use a separate branch/worktree for concurrent Goal work and isolated
  generated/temp paths.
- Commit, stage, push or create branches only when the user explicitly
  authorizes those Git actions. When authorized, use
  `<type>(currency-wars): <batch-id> <imperative summary>`.
- Do not preserve completed batch receipts as ad hoc prose snapshots. Current
  generated manifests and the active Goal tracker are the progress authority;
  Git history is the historical record.
- Never lower a coverage denominator, convert an executable record to metadata
  or weaken a fixture merely to make a gate pass.
- A blocked policy triggers bounded research and then an explicit versioned
  policy implementation. It does not justify a silent no-op.
- Update current code, workbooks, generated outputs, tests, coverage and state
  documentation together when their facts change.

## Acceptance

- Goal 12 inputs and all current denominators regenerate without drift.
- Production runtime loads no JSON, workbook or raw source program.
- All 102 Sora tables and 74,850 current rows are privately validated and
  lowered according to their generated dispositions.
- All 19,250 source obligations and 2,367 mechanic programs have exact-once
  terminal runtime dispositions with no `Blocked`, `Pending`, `CatalogOnly` or
  `IdentityOnly` executable item.
- All 28 semantic fixture families execute against production-lowered data.
- All 12 inherited gaps are exact or executable versioned policies with
  replacement tests and visible non-parity wording.
- Complete Standard and Overclock runs cross real battles and pass the generated
  legal matrix.
- Investment, build, equipment, Empowerment, Bond, encounter, affix and battle
  override behavior changes authoritative state/events in production fixtures.
- Rejections are byte-, hash- and RNG-inert; deterministic faults use the
  documented terminal path.
- CLI, Agent and MCP expose the same offered-command semantics and fresh replay
  verifies independently of the live session.
- No content ID enters shared resolver branches and no native handler is added
  without a reviewed static-registry admission record.
- Performance, dependency/license, provenance, Sora/workbook drift, native CI,
  prior-release isolation and clean-checkout gates pass.
- Current docs and `policy/state.json` claim only the behavior proven by the
  final generated release evidence.

## Goal-mode launch objective

Use the following objective when Goal 21 execution starts:

> Implement and release the complete Version 4.4 Currency Wars runtime defined
> by `docs/goals/21-currency-wars-runtime.md`. Begin at `G21-P0-B1`, proceed in
> dependency order, and continue until every source obligation, mechanic
> program, semantic fixture and policy has a terminal executable disposition
> and the complete-run release gates pass. Prefer pinned released evidence,
> then official released text and reproducible observations; when evidence
> remains unavailable, implement and label a deterministic replaceable
> `VersionedProjectPolicy`. Use openpyxl for production workbook authoring and
> Sora 0.6.1 for validation/codegen/export. Never treat IDs, catalog loading or
> no-op handlers as completed mechanics.
