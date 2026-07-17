# Starclock — Engine-Agnostic Combat Reference

**Starclock** is a deterministic, engine-agnostic Rust combat and activity simulator. This directory is its execution reference for mechanics inspired by *Honkai: Star Rail*. It deliberately excludes rendering, animation, audio, input widgets, networking, account systems, real-time exploration movement, gacha, account inventory, and copyrighted game assets.

The target is a headless battle kernel plus a generic activity orchestrator that can later be driven by Bevy, another engine, the `starclock` CLI, tests, or a simulation service.

**Current repository status:** implementation in progress. The normalized
Version 4.4 content reference pack and the responsibility-separated Rust
workspace exist, but production Excel workbooks, Sora bundle, executable
catalogs, battle runtime and CLI commands are not completed artifacts. See the
[Goal 01 status ledger](goals/01-core-combat-and-content-status.md).

## Research baseline

- Sources were reviewed on **2026-07-17**.
- HoYoverse does not publish a complete combat specification. Exact formulas are therefore based primarily on current community-maintained references, cross-checked where possible against in-game tutorial transcriptions and independent calculations.
- A formula marked **verified** has consistent support in at least two established references or an established reference plus a worked in-game result.
- A rule marked **observed** is widely documented but may contain edge cases.
- A rule marked **project policy** is an explicit implementation choice where the original game's behavior is unpublished, unstable, or unnecessary for the first milestone.

Treat the date, not an assumed game version number, as the baseline. Character kits and newer mode-specific systems change frequently; the underlying engine must express exceptions without hard-coding them into shared formulas.

## Document map

1. [Core battle model](01-core-battle-model.md) — scope, entities, targeting, lifecycle, and event semantics.
2. [Action order and turns](02-action-order-and-turns.md) — Speed, Action Gauge, interrupts, duration timing, and cycles.
3. [Damage and sustain formulas](03-damage-and-sustain.md) — ordinary damage, DEF, RES, CRIT, healing, and shields.
4. [Toughness and Weakness Break](04-toughness-and-break.md) — Toughness reduction, elemental breaks, Break DoT, and Super Break.
5. [Effects and resources](05-effects-and-resources.md) — buffs, debuffs, effect chance, Skill Points, Energy, and aggro.
6. [Rust architecture](06-rust-architecture.md) — pure-core boundaries, data model, command/event API, RNG, and test plan.
7. [Excel and Sora configuration pipeline](07-configuration-pipeline.md) — Excel authoring, Sora schema/code generation/export, runtime bundle loading, and CI policy.
8. [Rust engineering standards](08-engineering-standards.md) — file-size limits, responsibility-based modules, visibility/re-export policy, errors, testing, linting, and review gates.
9. [Cross-platform determinism and numeric policy](09-determinism-and-numerics.md) — fixed-point arithmetic, rounding, RNG, stable ordering, state hashing, command atomicity, and platform verification.
10. [Lifecycle and resolution](10-lifecycle-and-resolution.md) — command atomicity, action phases, defeat, presence, waves, faults, and event causality.
11. [Rule IR and native handlers](11-rule-ir-and-native-handlers.md) — typed conditions, selectors, operations, triggers, state slots, scopes, and the exceptional-code boundary.
12. [Modifier and snapshot pipeline](12-modifier-and-snapshot-pipeline.md) — stat-query stages, stacking groups, applicability, snapshots, and cycle detection.
13. [Enemy AI and encounters](13-enemy-ai-and-encounters.md) — enemy definitions, deterministic behavior graphs, boss phases, variants, waves, and encounter validation.
14. [Universe mode profiles](14-run-core-and-universe-modes.md) — Standard SU, Swarm Disaster, Gold and Gears, Unknowable Domain, and Divergent Universe over the generic activity core.
15. [Content data and coverage](15-content-data-and-coverage.md) — bilingual/provenance fields, character/equipment/enemy/mode schemas, completeness states, and the Version 4.4 manifest policy.
16. [Replay, CLI, and engine integration](16-replay-cli-and-engine-integration.md) — canonical replay format, planned CLI contracts, baseline controllers, and Bevy/engine adapters.
17. [Documentation coverage matrix](17-documentation-coverage-matrix.md) — current normative coverage, research gaps, data-import status, and terminal gates.
18. [Standard and challenge modes](18-standard-and-challenge-modes.md) — ordinary battles plus Memory of Chaos, Pure Fiction, and Apocalyptic Shadow orchestration, clocks, scoring, and seasonal data.
19. [Activity core and mode extension](19-activity-core-and-mode-extension.md) — unified graphs, scopes, rosters, persistence, clocks, metrics, battle handoff, and future-mode extension.
20. [Core implementation design](20-core-implementation-design.md) — concrete Rust ownership, identities, stores, action lowering, operations, events, transactions, modules, and implementation order.
21. [Character builds, Traces, and equipment](21-build-traces-and-equipment.md) — independent `starclock-build`, build selection/compilation, ability levels, Traces, Eidolons, Light Cones, relics, affixes, generic combat output, and validation.
22. [Reference data](reference-data.md) — constants and the attacker-level multiplier table.
23. [Sources and confidence](sources.md) — source provenance and known uncertainties.
24. [Public character mechanics](characters/README.md) — the public Version 4.4 combat-form catalog, behavioral contracts, and an auditable engine-feature matrix.
25. [Combat content reference pack](content-reference/README.md) — prepared Version 4.4 character, Trace, Eidolon, Light Cone, enemy, ability, and ordinary-encounter facts used before Excel/Sora authoring.
26. [Dependency and tool policy](dependency-and-tool-policy.md) — exact active package/tool versions, licenses, deterministic impact, compile-cost records and rejected alternatives.
27. [Sora 0.3.0 capability lock](sora-0.3.0-capability-lock.md) — checksum-bound installation, executed CLI/schema/codegen/export surface and pinned limitations.
28. [CI platform matrix](ci-platform-matrix.md) — pinned hosted runners, native execution, compile-only targets and retained evidence boundaries.
29. [Common configuration schema](common-configuration-schema.md) — stable identity, localization, version, provenance, evidence and canonical-decimal transport contracts.
30. [Character and build configuration schema](character-build-configuration-schema.md) — Sora contracts for abilities, hit plans, characters, Traces, Eidolons, build patches and Light Cones.
31. [Typed Rule IR configuration schema](rule-ir-configuration-schema.md) — transport contracts for rules, slots, triggers, expressions, selectors, operations, effects, modifiers and native-handler metadata.

## Execution goals

- [Goal packages](goals/README.md) translate these specifications into
  resumable, commit-sized implementation work.
- [Goal 01 — Complete Core Combat and Released Character Content](goals/01-core-combat-and-content.md)
  implements the deterministic combat kernel, Standard battles, all released
  character forms with Traces/Techniques/Eidolons, and all released Light Cones.
- [Goal 01 status ledger](goals/01-core-combat-and-content-status.md) records the
  active batch, manifests, evidence and terminal gates.
- [Goal 01 launch prompt](goals/01-core-combat-and-content-prompt.md) starts or
  resumes the commit-by-commit execution loop until the goal is complete.

## Delivery boundary

The documentation target covers the complete core model even when implementation is staged. A playable headless vertical slice should exercise:

- two teams, waves, defeat, victory, and deterministic target selection;
- Basic ATK, Skill, Ultimate, passive triggers, follow-up actions, and enemy actions;
- HP, ATK, DEF, SPD, CRIT, DMG Boost, RES, Effect Hit Rate/RES, Break Effect, Energy, and Skill Points;
- the Action Gauge timeline and action advance/delay;
- single-target, Blast, AoE, and Bounce target patterns;
- multi-hit attacks, per-hit CRIT, damage, healing, shields, buffs, debuffs, and DoT;
- elemental weaknesses, Toughness, Weakness Break, and the seven base break effects;
- replayable seeded RNG and a serializable battle log;
- at least one summon or memosprite, transformation, special resource, Exo-Toughness interaction, multi-phase boss, and mode-supplied rule bundle;
- a Standard encounter and representative Memory of Chaos, Pure Fiction, and Apocalyptic Shadow challenge stages using the same combat core.

Multi-layer Toughness, Exo-Toughness, summons/memosprites, special resources, Super Break providers, boss phases, and universe-mode rules are no longer deferred from the architecture. They may be delivered after the first vertical slice, but their data and lifecycle contracts are normative now. A feature is not considered implemented merely because an ID or short profile exists; use the coverage states in [Content data and coverage](15-content-data-and-coverage.md).

## Intellectual-property boundary

Use these documents as a mechanics study and implementation reference. Do not copy proprietary assets, story text, voice, extracted client code, or redistributed game data. Prefer original identifiers and test fixtures in the implementation.
