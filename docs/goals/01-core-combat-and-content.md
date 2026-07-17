# Goal 01 — Complete Core Combat and Released Character Content

## 1. Goal statement

Implement the first production milestone of Starclock as a deterministic,
engine-agnostic Rust library and CLI that can compile complete character builds
and execute ordinary Standard battles from start to finish.

The required content baseline is the frozen public Version 4.4 manifest dated
2026-07-17. The goal includes every enabled released character combat form in
that manifest, all battle-relevant Traces, Techniques and Eidolons, and every
released Light Cone in the frozen Light Cone manifest. Announced but unavailable
content remains disabled and must not contain guessed or leaked values.

This goal deliberately does not implement Simulated Universe families, the
three recurring challenge families, or other activity-specific gameplay. It
must preserve the extension boundaries already specified for those systems,
but no out-of-scope mode may delay completion of the core battle milestone.

## 1.1 Pre-start content prerequisite

Goal 01 starts from the prepared
[Version 4.4 content reference pack](../content-reference/README.md). It must not
start bulk implementation from the compact profiles alone.

Before `G01-P0-B1` begins, the executor verifies:

- reference pack digest
  `0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a`;
- the machine counts and gates in
  [reference coverage](../content-reference/coverage.md);
- the [reference schema](../content-reference/schema.md) and
  [content authoring contract](../content-reference/authoring-contract.md);
- that the pinned released source cache can be reproduced and every generated
  file matches `pack-index.json`.

The pack provides prepared facts and source evidence; Goal 01 lowers them into
reviewed Excel/Sora rows and executable Rule IR. It may refine an approximation
through a stronger public source or observation fixture, but must record the old
fact, decision, and new evidence rather than silently changing it.

## 2. Terminal outcome

Goal 01 is complete only when all of the following are true:

1. A validated build containing a released character form, ability levels,
   unlocked Traces, an Eidolon level and a Light Cone can be compiled into a
   generic `ResolvedCombatantSpec` without exposing build-system types to the
   combat crate.
2. Every enabled released character form in the frozen goal manifest is
   `DataReady`, including exact combat values, ability level curves, hit plans,
   Techniques, battle-relevant Traces, E1-E6 patches, summons, memosprites and
   required exceptional handlers.
3. Every released Light Cone in the frozen goal manifest is `DataReady`,
   including level/promotion statistics, S1-S5 values, applicability rules and
   provenance.
4. The combat kernel implements every shared mechanic required by the frozen
   character and Light Cone manifests and by the normative core specifications.
5. The Standard profile can construct and finish deterministic single-wave,
   multi-wave, elite and multi-phase boss encounters with authored enemy AI.
6. The CLI can validate configuration, report goal coverage, run a Standard
   battle with a fixed seed, write a replay and verify that replay.
7. Accepted commands, controller decisions, RNG draws and authoritative state
   produce stable canonical hashes on supported platforms.
8. Required tests, generated-data drift checks, provenance validation, format
   checks and lints pass from a clean checkout.
9. The status ledger contains evidence for every acceptance gate and contains
   no required `Pending`, `InProgress`, `Researching` or `Blocked` entry.
10. Every required content row maps back to a prepared reference record and the
    bound reference-pack digest.
11. The committed CI matrix proves the supported platform gates, and versioned
    performance workloads remain within their reviewed latency, throughput and
    allocation budgets on the designated benchmark runner.

An identity row, behavioral summary, schema, disabled row, mock implementation,
TODO, or unverified guessed value does not satisfy a content gate.

## 3. Normative document set

Implementation decisions must follow the repository documents in this order of
authority:

1. This goal document and its status ledger define milestone scope and delivery
   gates.
2. [Core implementation design](../20-core-implementation-design.md),
   [Rust architecture](../06-rust-architecture.md),
   [lifecycle and resolution](../10-lifecycle-and-resolution.md), and
   [build, Traces and equipment](../21-build-traces-and-equipment.md) define
   ownership and public boundaries.
3. [Determinism and numerics](../09-determinism-and-numerics.md),
   [rule IR](../11-rule-ir-and-native-handlers.md),
   [modifier and snapshot pipeline](../12-modifier-and-snapshot-pipeline.md),
   and [engineering standards](../08-engineering-standards.md) define mandatory
   implementation policy.
4. Documents `01` through `05` and `13` define battle formulas, timeline,
   effects, resources, Toughness, enemy AI and encounter behavior.
5. [Configuration pipeline](../07-configuration-pipeline.md),
   [content and coverage](../15-content-data-and-coverage.md),
   [replay and CLI](../16-replay-cli-and-engine-integration.md),
   [Standard and challenge profiles](../18-standard-and-challenge-modes.md), and
   [activity core](../19-activity-core-and-mode-extension.md) define integration
   contracts. Only the Standard profile in document `18` is in this goal.
6. [Sources](../sources.md), [reference data](../reference-data.md), and the
   [character catalog](../characters/README.md) define research and content
   evidence requirements.
7. [Content reference schema](../content-reference/schema.md) and
   [authoring contract](../content-reference/authoring-contract.md) define how
   prepared facts are promoted into Excel/Sora and executable definitions.

If two normative documents conflict, do not silently choose one. Add a narrow
decision record, update the affected documents and add a regression fixture in
the same batch. This goal document may narrow delivery scope, but it does not
replace combat semantics defined by the core specifications.

## 4. Included scope

### 4.1 Runtime crates

The following responsibility-separated workspace crates are required:

- `starclock-combat`: battle definitions, authoritative state, commands,
  decisions, formula stages, rule execution, timeline and resolver;
- `starclock-build`: build selections, Trace/Eidolon/ability-level patches,
  Light Cone application and compilation to combat-domain input;
- `starclock-data`: generated Sora readers, catalog construction, validation,
  provenance and coverage;
- `starclock-rules`: deterministic static native-handler registry for mechanics
  that the typed rule IR cannot reasonably express;
- `starclock-replay`: canonical command/event encoding, state hashing and replay
  verification;
- `starclock-ai`: deterministic baseline player controller and authored enemy
  controller support;
- `starclock-activity`: the minimum generic activity handoff needed to run one
  Battle node and reach a terminal outcome;
- `starclock-mode-standard`: ordinary battle profile construction with no
  challenge clock, score or seasonal modifier layer;
- `starclock-cli`: configuration, coverage, battle-run and replay commands.

Dependency direction remains one-way. In particular:

- `starclock-combat` must not depend on build, data, activity, mode or engine
  crates;
- `starclock-build` consumes combat-domain input types but its types never enter
  authoritative battle state;
- `starclock-activity` treats battle participant specifications and digests as
  opaque handoff values;
- generated Sora row types and the selected fixed-point implementation remain
  private behind validated domain types;
- no public API may depend on Bevy, an ECS, rendering or account state.

### 4.2 Core battle behavior

The implementation must cover at least:

- authoritative battle phases, command atomicity, rollback and explicit faults;
- teams, combatants, summons, memosprites, linked entities and battlefield
  presence independent from life state;
- action gauge, Speed, turns, actions, hits, interrupts, Ultimates, extra turns,
  follow-ups, counters and deterministic tie-breaking;
- Basic ATK, Skill, Ultimate, Talent, Technique and content-defined actions;
- single-target, Blast, AoE, Bounce, random and content-defined target plans;
- multi-hit damage, CRIT, DEF, RES, vulnerability, damage reduction, healing,
  shields, HP consumption and overflow policies;
- buffs, debuffs, crowd control, DoT, dispel, cleanse, fields, marks, stacks,
  counters, duration scopes and chance checks;
- Energy, Skill Points, character resources and source-owned state slots;
- Toughness, Weakness Break, elemental break effects, Break DoT, Super Break,
  Exo-Toughness and multiple Toughness layers when required by included content;
- derived-stat modifiers, stacking groups, formula stages, caps, query-cycle
  detection and explicit snapshot policies;
- defeat, downed state, revival, departure, transformation, replacement, phase
  change, target invalidation and post-action wave advancement;
- complete cause chains and deterministic event ordering;
- typed rule IR plus narrowly justified native handlers.

The content manifest is also a completeness oracle: a shared mechanic required
by any included character form or Light Cone is part of Goal 01 even if it is
not enumerated above.

### 4.3 Character and Light Cone content

For each enabled released character combat form, required rows include:

- stable identity and bilingual name/summary fields;
- path, element, rarity and level/promotion base statistics;
- all ability definitions, exact level curves, costs, target plans and hit plans;
- Talent and Technique behavior;
- all battle-relevant minor and major Traces and unlock conditions;
- Eidolon patches E1 through E6;
- required state slots, rules, modifiers, summons, memosprites and transformations;
- native-handler reference and justification where IR is insufficient;
- source records, access date, confidence, version note and evidence hash;
- validation fixtures proving E0 and E6 compilation and representative behavior.

For each released Light Cone, required rows include:

- stable identity and bilingual name/summary fields;
- path, rarity, level/promotion statistics and exact scaling curves;
- passive rules and S1 through S5 values;
- wearer/applicability filters, state slots, modifiers and triggers;
- source records and evidence hashes;
- validation fixtures for S1, S5, valid wearer and invalid wearer behavior.

### 4.4 Standard battle content

Phase 0 freezes a `standard-v1` manifest. It must contain enough exact enemy,
encounter and scenario data to prove the complete battle kernel without claiming
the full public enemy catalog. At minimum it includes:

- a basic single-wave encounter;
- a deterministic multi-wave encounter;
- an elite encounter with adds;
- a summoner or replaceable battlefield entity;
- a crowd-control and DoT encounter;
- a multi-phase boss with a phase transition;
- a Toughness-layer or equivalent routing fixture;
- encounters exercising defeat, revival/return when supported by included
  content, target invalidation and post-action wave advancement;
- scenario presets covering representative player builds and all target shapes.

Every row is subject to the same bilingual, provenance and `DataReady` rules as
other content. Synthetic fixtures may supplement this manifest for isolated
tests, but may not replace public-data acceptance scenarios.

### 4.5 Tooling and reproducibility

Required tooling includes:

- `.xlsx` editable sources and `sora-cli = 0.3.0` as the authoritative
  validation/export path;
- deterministic `.sora` production output and JSON diagnostic output;
- a deterministic workbook bootstrap/import tool using the pinned
  `rust_xlsxwriter` policy from document `07`;
- locally cached, ignored third-party evidence with committed revision and hash
  metadata;
- generated coverage reports tied to catalog and source digests;
- canonical replay and state-hash fixtures;
- committed CI workflows covering supported Windows, Linux and macOS runners,
  plus repository scripts for formatting, linting, tests, generated-file drift,
  source-file-size enforcement and golden-vector artifacts;
- a versioned combat benchmark harness that reports command latency, battle
  throughput, allocations, state-copy cost and journal growth.

## 5. Explicitly excluded scope

The following are not Goal 01 deliverables:

- Standard Simulated Universe, Swarm Disaster, Gold and Gears, Unknowable Domain,
  Divergent Universe, blessings, curios, equations, resonances or run progression;
- Memory of Chaos, Pure Fiction, Apocalyptic Shadow, their clocks, scoring,
  seasonal rules and active-stage data;
- a complete public enemy and encounter catalog beyond `standard-v1`;
- complete relic, planar ornament, main-affix and sub-affix datasets;
- Adventure/minigame nodes, shops, reward selection or activity currencies;
- UI, rendering, animation, audio, input widgets or a Bevy adapter implementation;
- account progression, inventory ownership, acquisition, gacha, rewards, story,
  exploration, networking, persistence services or game assets.

`starclock-build` may retain the documented future boundary for relics and
planar ornaments, but placeholder relic data is not an acceptance item and must
not be reported as implemented. Generic combat modifiers must remain capable of
receiving future equipment output without changing `starclock-combat`.

Out-of-scope mode crates must not be created merely as empty scaffolding. The
generic extension seams described in documents `14`, `18` and `19` must remain
intact and be protected by architecture tests where practical.

## 6. Delivery rules

### 6.1 Batch and commit policy

Each batch below is one reviewable, atomic commit unless its row explicitly says
that it expands into a deterministic series of content commits. A batch commit:

1. changes only one responsibility or one bounded content partition;
2. includes its tests, schema changes, migration, generated output and relevant
   documentation in the same commit;
3. updates the [Goal 01 status ledger](01-core-combat-and-content-status.md) and
   generated coverage evidence;
4. passes the batch gates before commit;
5. leaves no unrelated working-tree changes and does not overwrite user changes;
6. does not combine opportunistic refactors from later phases;
7. is not pushed or published unless separately authorized.

Preferred commit subjects use the batch identifier, for example:

```text
feat(combat): G01-P3-B1 add battle aggregate and command boundary
data(characters): G01-P7-C03 import character partition 03
test(replay): G01-P8-B2 add cross-platform golden vectors
```

The executor may split a batch if it becomes too large for review, but must add
the child IDs to the ledger before implementing them. It may not merge batches
only to reduce commit count.

### 6.2 Universal batch gates

Every implementation batch must run the applicable subset of:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
starclock config validate
starclock catalog coverage --goal core-combat-v1
```

Data batches additionally run Sora validation/export, generated-file drift,
reference integrity, bilingual-field, provenance and evidence-hash checks.
Determinism batches additionally run canonical vector and replay checks. If the
full workspace is temporarily unavailable during initial bootstrap, the ledger
must record the exact narrower command and the later batch that closes the gap.

Warnings, ignored failures, undocumented exclusions and locally patched
generated files do not count as passing gates.

### 6.3 Research policy

- Freeze manifests before bulk import and report completeness by manifest ID,
  not estimated row count.
- Use public sources only. Do not use leaks, extracted proprietary assets, copied
  long descriptions or unpublished values.
- Exact claims require provenance under document `15`. Conflicts remain
  `Researching` with evidence; they are not filled using a convenient default.
- Project-policy behavior must be labeled, documented and covered by a fixture.
- Independently summarize descriptions in English and Simplified Chinese.
- A content partition may commit only when every included entry reaches
  `DataReady`; partial entries remain in staging outside the authoritative bundle.
- Phase 4 real-content probes are the sole exception to the last rule: they use
  a dedicated Sora probe scope, remain excluded from production catalogs and
  coverage, and may contain only the minimum reviewed rows needed to exercise a
  shared mechanic. They never count as `DataReady` content.

## 7. Execution phases and commit batches

Phases are ordered. A later batch may start early only when its dependencies are
complete and doing so does not conceal a failed earlier gate. Phase 4 is
deliberately interleaved with reviewed real-content mechanism probes; complete
content import remains in Phase 7. The status ledger is the source of truth for
the next unblocked batch.

### Phase 0 — Freeze scope and evidence

**Exit gate:** immutable goal manifests and evidence policy exist; every required
ID is known; unresolved normative questions have owners and fixtures.

| Batch | Atomic deliverable |
|---|---|
| `G01-P0-B1` | Verify and bind the prepared content-reference pack, then freeze `core-combat-v1` manifests for released combat forms, released Light Cones and `standard-v1` enemies, encounters and scenarios. Record both digests and inclusion state, deterministically generate every concrete Phase 7 character/Light Cone partition, and add its ledger row immediately. |
| `G01-P0-B2` | Map every goal-manifest entry to prepared reference records and verify source-cache revisions, source-file hashes, approximation labels, and evidence hashes. Explicitly audit the pinned 4.3 fallback and 4.4 identity mapping for Saber and Archer. Do not create a competing staging model. |
| `G01-P0-B3` | Convert remaining formula/timing ambiguities that block implementation into named research cases, decision records and reproducible observation/golden fixture specifications. The initial register must explicitly cover shared Elation semantics and every unresolved V1a mechanism probe. |
| `G01-P0-B4` | Generate the initial machine-readable goal coverage report and verify that documentation counts match the frozen manifests. |

### Phase 1 — Workspace and reproducible data foundation

**Exit gate:** the responsibility-separated workspace builds from a clean
checkout; pinned tools and empty validated catalogs reproduce without drift.

| Batch | Atomic deliverable |
|---|---|
| `G01-P1-B1` | Replace the placeholder package layout with the required workspace crates and enforce dependency-direction tests. |
| `G01-P1-B2` | Add pinned dependency/tool policy, private fixed-point backend boundary, stable ID foundation and toolchain metadata. Record purpose, version, license, deterministic impact, compile-time cost and rejected alternatives for every new dependency or binary. |
| `G01-P1-B3` | Add local format, clippy, test, file-size, visibility, dependency-direction and generated-drift repository checks. |
| `G01-P1-B4` | Pin and install `sora-cli 0.3.0` by checksum, then prove its exact CLI/schema capabilities with a minimal golden project: `check`, `build`, `excel-template`, preview/write `excel-sync`, references, indexes, unions/child tables, Rust codegen and reader, `.sora` export and diagnostic JSON. Record actual spellings and unsupported assumptions before production schema work. |
| `G01-P1-B5` | Add committed CI workflows with a pinned toolchain and Windows/Linux/macOS matrix. Run repository checks, the proven Sora capability/drift path, golden artifacts and source-size enforcement through the same scripts used locally; document the supported native and compile-only CPU targets. |
| `G01-P1-B6` | Add shared schema modules for stable identity, localization, versions, provenance, confidence, evidence hashes and canonical decimal fields. |
| `G01-P1-B7` | Add the Character/Ability/HitPlan/Trace/Eidolon/LightCone schema family and representative cross-reference fixtures. |
| `G01-P1-B8` | Add the Effect/Modifier/RuleDefinition/Slot/Trigger/Expression/Selector/Operation schema family and representative typed-program fixtures. |
| `G01-P1-B9` | Add the Enemy/Variant/Ability/AI/Encounter/Standard scenario schema family plus only the minimum generic Activity/BattleBinding/ResultProjection rows required by Goal 01. Do not add challenge or universe tables. |
| `G01-P1-B10` | Add the deterministic reference-to-workbook bootstrap/import command, authoritative `.xlsx` layout, `.sora` production output, diagnostic JSON, generated Rust readers and conversion boundary. Regeneration never overwrites a designer-edited workbook. |
| `G01-P1-B11` | Add validated empty-domain and representative-domain catalog construction, real bundle-load tests and committed clean-regeneration fixtures proving the complete data pipeline has no drift. |

### Phase 2 — Deterministic primitives

**Exit gate:** numeric, RNG, canonical codec, catalog identity and replay headers
have cross-platform golden vectors and contain no authoritative floating point.

| Batch | Atomic deliverable |
|---|---|
| `G01-P2-B1` | Implement private fixed-point-backed domain newtypes, checked arithmetic, explicit rounding and overflow faults. |
| `G01-P2-B2` | Implement stable typed IDs, ordered collections, immutable catalog construction and reference validation. |
| `G01-P2-B3` | Implement SHA-256 stream derivation, ChaCha8 RNG wrapper, stable range/weighted selection and draw counters. |
| `G01-P2-B4` | Define and implement the first-class canonical replay contract: version policy, header, record envelope, config/numeric/RNG/controller identities, digest types, unknown-record policy and state-hash vector tests. The canonical encoder writes to a shared sink contract and the production hash path streams directly into SHA-256 without a full-state byte allocation. Later phases add domain record payloads without redesigning this envelope. |
| `G01-P2-B5` | Add numeric boundary, rounding, overflow and floating-point-oracle tests for documented formulas. |
| `G01-P2-B6` | Establish the reusable property-test harness for fixed-point arithmetic, RNG mapping, canonical encoding, malformed input and stable collection ordering. Pin and license-review its dependencies; Phase 8 expands this harness rather than introducing it. |

### Phase 3 — Executable combat vertical slice

**Exit gate:** a synthetic but fully deterministic Standard single-wave battle
runs through commands to victory and replay verification.

| Batch | Atomic deliverable |
|---|---|
| `G01-P3-B1` | Implement `Battle`, authoritative stores, phases, `Command`, ordered legal decisions, `Resolution` and immutable query views. |
| `G01-P3-B2` | Implement mutation journal/rollback, explicit fault transition, event journal and complete cause-chain identities. The owned working-state transaction must reuse battle-local scratch through `clone_from`/state swapping, prepare it only after legality validation, retain bounded capacity, and preserve byte-identical fault semantics. |
| `G01-P3-B3` | Implement action gauge, turn selection, command validation, action lowering, phases, hits and deterministic interrupt queue. |
| `G01-P3-B4` | Implement target selectors/plans, target locks, invalidation policy, Basic ATK/Skill/Ultimate resources and multi-hit execution. |
| `G01-P3-B5` | Implement initial damage, healing, defeat, victory, wave advancement and formula golden tests. |
| `G01-P3-B6` | Add a synthetic Standard profile plus CLI battle-run/replay smoke path proving command-to-hash determinism. |
| `G01-P3-B7` | Add versioned small/medium/large and server-verification benchmark workloads: ordinary/trigger-heavy apply, invalid-command rejection, streaming hash, one-shot 100/500-command replay and many isolated jobs sharing one catalog. Record command latency, commands/second/core, full-replay throughput, peak bytes/job, allocation, state-copy and journal/event/operation growth. Establish a generous order-of-magnitude CI smoke ceiling and designate the stable runner used for strict budgets. |
| `G01-P3-B8` | Add command-sequence property tests proving invalid-command non-mutation, rollback byte identity, deterministic command-to-hash behavior and replay-envelope corruption detection. |

### Phase 4 — Complete shared combat kernel

**Exit gate:** every generic mechanic required by the goal manifests is
implemented or has a reviewed native-handler boundary; core lifecycle and
formula suites pass. Every named V1a probe compiles from the dedicated Excel/Sora
probe scope and passes through the production data-to-domain boundary. Probe
rows remain non-production and do not count toward Phase 7 coverage.

The V1a probes are design feedback, not miniature synthetic stand-ins. Use Asta
for modifier/buff stacks, Kafka for DoT and authored detonation, Clara for
counter/follow-up scheduling, Firefly for HP consumption, transformation and
Super Break, Aglaea for memosprite ownership/presence/transformation, and at
least two released Elation forms to constrain the shared Elation subsystem from
more than one kit. A probe contains only the ability, slot, rule and fixture
rows required by its owning batch; `G01-P7-V1B` later replaces it with the
complete through-E6 production content.

| Batch | Atomic deliverable |
|---|---|
| `G01-P4-B1` | Implement typed rule definitions, expressions, conditions, selectors, operations, trigger phases, priorities and once-scopes, plus the deterministic static native-handler registry contract and audit scaffold. This foundation must exist before a real probe is accepted; no content handler is admitted without an IR-insufficiency decision. |
| `G01-P4-B2` | Implement staged stat queries, modifier registry, stacking/caps, filters, snapshot policies and cycle faults. Compile the Asta modifier/stack probe through Rule IR and prove its query, stacking and snapshot behavior. |
| `G01-P4-B3` | Complete damage, CRIT, DEF, RES, vulnerability, mitigation, HP consumption, healing, shields and overflow policies. Compile the Firefly HP-consumption and damage-stage probe through Rule IR. |
| `G01-P4-B4` | Implement Toughness routing, Weakness Break, seven base break effects, Break DoT, Super Break, Exo-Toughness and layered Toughness. Extend the Firefly probe through Weakness and Super Break integration. |
| `G01-P4-B5` | Implement buffs, debuffs, crowd control, DoT, dispel, cleanse, durations, chance checks, aggro, Energy, Skill Points and state-slot resources. Compile the Kafka DoT/detonation probe and the Asta buff-duration probe through Rule IR. |
| `G01-P4-B6` | Implement Ultimate interrupts, follow-ups, counters, extra actions/turns, delayed actions and reaction scheduling. Compile the Clara counter/follow-up probe and prove cause ownership and reaction ordering. |
| `G01-P4-B7` | Implement summon/memosprite ownership, presence states, link/transform/replace, downed/defeated/revive/departure and cross-wave policies. Compile the Aglaea memosprite probe and Firefly transformation probe. |
| `G01-P4-B8` | Implement shared Elation semantics explicitly required by the frozen manifest, including distinct Elation damage/ability tags, Punchline, Certified Banger, forced Elation Skills and shared actor/resource boundaries. Compile probes from at least two released Elation forms and reject one-kit-specific core APIs. |
| `G01-P4-B9` | Implement enemy AI graphs, waves, summons, boss phases, phase transitions and encounter validation. |
| `G01-P4-B10` | Complete native-handler contract tests and the audit that rejects scattered content-ID branches. Review every handler admitted by V1a, require its written IR-insufficiency decision and remove any handler now expressible through the completed shared IR. |
| `G01-P4-B11` | Run the complete core formula/lifecycle/rule/probe golden suite, review benchmark growth against the Phase 3 baseline, set provisional full-kernel budgets, and close or explicitly resolve every Phase 0 blocking research case. |

### Phase 5 — Build compiler, Traces, Eidolons and Light Cones

**Exit gate:** complete E0/E6 and S1/S5 representative builds compile
deterministically into combat-only definitions with stable source digests.

| Batch | Atomic deliverable |
|---|---|
| `G01-P5-B1` | Implement independent `BuildCatalog`, `BuildSpec`, validation report and compilation boundary to `ResolvedCombatantSpec`. |
| `G01-P5-B2` | Implement ability-level selection/curves and Trace unlock/stat/rule patches with deterministic patch ordering. |
| `G01-P5-B3` | Implement Eidolon E1-E6 patches, replacement/conflict validation and E0/E6 compilation fixtures. |
| `G01-P5-B4` | Implement Light Cone levels/promotions, path applicability, S1-S5 passive patches and valid/invalid wearer fixtures. |
| `G01-P5-B5` | Implement source attribution, definition/build/catalog digests, named build presets and build-lock verification. |
| `G01-P5-B6` | Protect the future relic/planar boundary with a narrow compatibility test; do not import or claim the deferred full dataset. |

### Phase 6 — Standard orchestration, AI, CLI and replay

**Exit gate:** public-data Standard scenarios run autonomously or from replay
through the production crate boundaries and finish with stable hashes.

| Batch | Atomic deliverable |
|---|---|
| `G01-P6-B1` | Implement the minimum generic Activity aggregate for one Battle node, battle handoff, result return and terminal outcome. Architecture tests must protect the `Activity -> Section -> Node -> Attempt -> Battle` scopes, typed scoped-slot/reset boundary, participant/build lock digest, opaque `BattleSpec` handoff, declared `BattleResultProjection`, result identity/hash validation and the absence of a combat-to-activity dependency. Do not add empty challenge, universe, fork/join or shop semantics. |
| `G01-P6-B2` | Implement `starclock-mode-standard` profile construction with no challenge or universe policies. |
| `G01-P6-B3` | Implement deterministic baseline player scoring and authored enemy-controller execution over ordered legal commands. |
| `G01-P6-B4` | Fill the Phase 2 canonical replay envelope with Activity records and controller-decision logs, then complete divergence diagnostics and replay verification. Do not redesign the version/header/digest contract here. |
| `G01-P6-B5` | Complete CLI `config validate`, goal-aware `catalog coverage`, `battle run` and `replay verify` contracts. |
| `G01-P6-B6` | Import and validate the frozen `standard-v1` enemy, encounter and scenario manifest and add seeded golden battles for every required archetype. |

### Phase 7 — Complete released content import

**Exit gate:** character and Light Cone goal manifests report 100% `DataReady`,
and every imported mechanic has executable behavior evidence.

| Batch family | Atomic deliverable |
|---|---|
| `G01-P7-V1B` | Promote the representative forms used by V1a into complete production content: Asta, Clara, Kafka, Firefly, Aglaea and one released Elation-focused form selected from the frozen manifest. Replace probe rows with full statistics, abilities, Techniques, Traces, E1-E6, provenance and fixtures; no partial probe row enters the production bundle. |
| `G01-P7-Cnn` | Import remaining released combat forms in stable manifest-ID partitions of at most 8 forms per commit. Each partition includes all statistics, abilities, level curves, hit plans, Technique, Traces, E1-E6, rules/handlers, bilingual fields, provenance and tests. |
| `G01-P7-Lnn` | Import released Light Cones in stable manifest-ID partitions of at most 16 cones per commit. Each partition includes levels/promotions, S1-S5, rules, bilingual fields, provenance and tests. |
| `G01-P7-Mnn` | Add a missing shared primitive or reviewed native handler identified by the frozen manifest, Phase 0 research or a content partition. Known Mnn batches must be registered before the first bulk import; newly discovered ones must precede the dependent content batch and include focused generic tests. |
| `G01-P7-R1` | Regenerate catalogs and coverage from clean sources; reject all incomplete, orphaned, unprovenanced or disabled-as-released records. |
| `G01-P7-R2` | Run manifest-wide compilation and scenario generation for every E0/S1 and E6/S5 character/compatible-Light-Cone fixture. |

The prepared 88-form/165-Light-Cone baseline implies one V1B batch, at least 11
remaining character partitions and at least 11 Light Cone partitions before any
Mnn or review batch. This is a capacity forecast, not a substitute for the exact
manifest-derived rows generated by `G01-P0-B1`.

Partition membership is generated once by `G01-P0-B1` and every concrete row is
recorded in the ledger before Phase 1 begins. Do not leave `Cnn`/`Lnn` as the
only progress representation after the manifests exist. Do not partition
alphabetically by display name if stable IDs provide a different order. A character partition may
not omit an exceptional mechanic to make the commit fit; split the partition or
land a preceding `Mnn` mechanic batch.

### Phase 8 — Hardening and documentation freeze

**Exit gate:** every terminal condition in section 2 has evidence, a clean
checkout reproduces it and the goal is marked `Complete`.

| Batch | Atomic deliverable |
|---|---|
| `G01-P8-B1` | Add manifest-wide reference, rule-reachability, native-handler, once-scope, modifier-conflict and source-provenance audits. |
| `G01-P8-B2` | Verify numeric, RNG, codec, battle, build and replay golden vectors on the CI matrix established by `G01-P1-B5`, and record native versus compile-only CPU evidence without claiming an unexecuted target. |
| `G01-P8-B3` | Expand the Phase 2/3 property harness into long-sequence and coverage-guided fuzz tests for invalid commands, rollback, selector validity, effect timing, content compilation and replay corruption; preserve reproducible seeds/corpora for every failure. |
| `G01-P8-B4` | Run source-file-size/public-API/dependency audits and split or document every justified exception. |
| `G01-P8-B5` | Run the versioned representative Standard and server-verification workloads on the designated stable benchmark runner. Compare incremental command latency, one-shot replay throughput, commands/second/core, concurrent isolated-job scaling, peak bytes/job, allocations, state-copy/hash cost and journal growth with reviewed budgets, and record or resolve every material regression. Shared CI retains only the generous smoke ceiling. |
| `G01-P8-B6` | Freeze CLI/library contracts, regenerate documentation/coverage, run clean-checkout acceptance and record release evidence. |
| `G01-P8-B7` | Mark the ledger complete only after all gates pass; commit the final Goal 01 completion record. |

## 8. Acceptance suites

### 8.1 Determinism and numeric acceptance

- no authoritative `f32` or `f64` enters state, rules, generated records or
  formula evaluation;
- checked overflow produces a deterministic fault and rejected commands leave
  state and RNG counters unchanged;
- every formula boundary has an explicit rounding policy;
- stable candidate ordering and integer range/weighted selection are tested;
- canonical serialization never hashes ordinary `serde` output;
- state hashes match after every accepted command across supported platforms;
- replay verification detects command, catalog, numeric revision, RNG revision,
  controller-decision and state-hash divergence.

### 8.2 Combat acceptance

- formula goldens cover damage, Break, Super Break, DoT, healing, shields,
  effect chance, aggro and action order;
- lifecycle goldens cover death, revival, replacement, transformation, summons,
  memosprites, phase changes, multi-wave attacks, target invalidation and faults;
- rule goldens cover selectors, trigger phases, priorities, once-scopes, cause
  chains, snapshotting, source ownership and modifier conflicts;
- legal-decision ordering and baseline AI tie-breaking are stable;
- every `standard-v1` scenario reaches the expected terminal result with the
  expected event and final state hashes.

### 8.3 Build and content acceptance

- every enabled character form compiles at E0 and E6 with minimum/maximum valid
  ability levels and its required summons/memosprites;
- every Light Cone validates at S1 and S5 and enforces path applicability;
- representative combined builds run in Standard battles rather than only unit
  tests;
- all content references resolve and no unregistered native handler is used;
- all required bilingual fields, source facts and evidence hashes are present;
- generated production bundles are reproducible and contain no `Researching`
  entry in a required manifest;
- coverage is computed from the frozen manifest and reports exactly 100% for
  required character forms, Light Cones and `standard-v1` entries.

### 8.4 Engineering acceptance

- source files remain below 1,200 physical lines unless a reviewed exception is
  documented;
- modules are split by responsibility and broad convenience `pub use` exports
  are absent;
- public APIs expose domain types instead of numeric-backend or Sora row types;
- formatting, clippy with warnings denied and all workspace tests pass;
- committed Windows/Linux/macOS CI workflows invoke the same pinned repository
  checks as local execution and publish enough evidence to distinguish native
  execution from compile-only target coverage;
- generated output has no drift from Excel/Sora sources;
- the combat crate has no dependency on data, build, activity, mode, CLI, Bevy
  or other engine crates;
- no universe/challenge implementation or deferred full relic/enemy dataset is
  represented as Goal 01 work.

### 8.5 Performance acceptance

- benchmark workloads are versioned inputs with fixed configuration, seed,
  controller and terminal command count;
- the initial synthetic and final representative Standard reports include
  ordinary/trigger-heavy command latency, invalid-command cost, full-battle and
  one-shot 100/500-command replay throughput, commands/second/core, concurrent
  isolated-job scaling, allocation count/bytes, peak bytes/job, authoritative
  state-copy/hash cost and journal/event/operation growth;
- stateful incremental execution and one-shot audit replay produce identical
  events and hashes, while no supported live-verification workflow repeatedly
  replays growing command prefixes;
- canonical state hashing uses the shared streaming encoder, and the baseline
  transaction reuses bounded scratch capacity without changing canonical state;
- strict regression budgets run on one documented stable runner, while the
  cross-platform CI matrix uses only a generous order-of-magnitude smoke ceiling;
- a budget failure is resolved, intentionally re-baselined with a decision
  record, or remains a blocking gate; it is never hidden by deleting samples;
- benchmark code and instrumentation do not change authoritative battle state,
  RNG consumption, canonical events or hashes.

## 9. Progress accounting

The [status ledger](01-core-combat-and-content-status.md) is updated in every
batch commit. It records:

- active phase and next unblocked batch;
- commit hash and validation commands for each completed batch;
- frozen manifest digests and exact coverage totals;
- content partition membership and states;
- open research cases, blockers and architectural decisions;
- cross-platform acceptance evidence;
- performance baseline, runner identity and final budget evidence;
- the final checklist from section 2.

The reusable [Goal 01 launch prompt](01-core-combat-and-content-prompt.md)
instructs an executor to read the normative documents, select the next ledger
batch, implement and verify it, commit atomically, update the ledger and continue
the loop until the terminal outcome is proven.
