# Content Data and Coverage

This document defines the Version 4.4 data snapshot, common schemas, provenance, and the only valid meaning of “complete.” It deliberately separates public identity coverage from executable, numerically verified content.

## Snapshot boundary

The target snapshot is Version **4.4**, reviewed on **2026-07-17**. It includes public content available in that version plus officially announced characters represented as disabled placeholders. The current data-import target excludes unannounced/leaked Version 4.5 content, Currency Wars, historical Divergent Universe revisions, and past rotating challenge seasons except when an archived digest is required to verify an existing replay. These are data-scope exclusions, not reasons to hard-code architectural limits into `starclock-activity`.

The frozen coverage manifest is a versioned list of stable IDs and expected release states. Import work may revise an ID or split a mechanically distinct variant only through a reviewed manifest migration with provenance. Row counts are derived from the manifest; documentation must not invent totals for categories whose source catalog has not been frozen.

## Coverage states

Every manifest entry has exactly one state:

| State | Meaning |
|---|---|
| `Cataloged` | Stable identity, category, release state, and source exist. No executable-mechanics claim. |
| `Documented` | Original bilingual mechanism summary and dependency tags exist. Exact data may be missing. |
| `Researching` | Conflicting/missing coefficients, hit plan, timing, AI, or formula evidence blocks execution. |
| `DataReady` | All required rows and references validate and the entry is enabled. |
| `GoldenVerified` | `DataReady` plus representative observation/replay fixtures pass. |
| `Disabled` | Officially public placeholder or intentionally unsupported entry that normal simulations cannot load. |

States are monotonic only within one data revision. A live-version change can move an entry from `GoldenVerified` back to `Researching` in the next revision. “Released,” “profile written,” and “DataReady” are independent facts.

## Delivery lanes

Coverage state and publication maturity are separate. New gameplay uses:

| Lane | Requirements | Availability |
|---|---|---|
| `Experimental` | Isolated `.xlsx` workbook, partial local manifest, original/synthetic fixtures, valid deterministic domain data | Tests and explicit developer entry points only |
| `Candidate` | Production Sora schema/export, provenance, bounded promised manifest, semantic fixtures | Integration and compatibility testing |
| `Released` | Complete bilingual/provenance fields, 100% promised-manifest accounting, replay and platform gates | Production library/CLI/MCP profiles |

Experimental content does not bypass Excel/Sora authoring, numeric policy,
typed operations, reference validation, or deterministic simulation. It
defers publication, full-source research, localization, and complete-catalog
claims while a mechanic is being proved. Debug JSON never becomes a parallel
production input.

Promotion creates a new digest-bound revision. Production catalog construction
rejects Experimental rows, and coverage reports never count them as released.
The detailed evolution contract is
[Mode extension and evolution](26-mode-extension-and-evolution.md).

## Common identity and localization

All content uses a neutral stable key and these project-authored metadata fields:

```text
id
name_en
name_zh_cn
summary_en
summary_zh_cn
game_version_introduced
game_version_snapshot
release_state
enabled
coverage_state
source_record_ids[]
```

Names may preserve official public naming. Summaries are concise original descriptions of mechanics, not copied ability/story prose. IDs never depend on localized names.

## Provenance

A `SourceRecord` contains stable source ID, publisher/site, direct URL, access date, applicable game version, source category, confidence, license/usage note, evidence digest, and an optional short conflict note.

Source categories are:

1. official release notes, tutorials, and public descriptions;
2. direct reproducible observation recorded by this project;
3. maintained community formula/index source;
4. structured transcription aid;
5. explicit project policy or synthetic test fixture.

Official descriptions are authoritative for disclosed behavior, not hidden arithmetic. Community indexes may bootstrap factual numbers, but high-impact or surprising values require a second source or observation. Local source caches, extracted assets, and long raw descriptions remain ignored and uncommitted.

## Character and ability data

The normalized progression/loadout model and deterministic compilation order are defined in [Character builds, Traces, and equipment](21-build-traces-and-equipment.md).

A complete released combat form requires:

- identity, rarity, element, path, base resource caps, level/promotion stat rows, and stat-curve provenance;
- Basic, Skill, Ultimate, Talent, battle-relevant Technique entry effect, enhanced/replacement abilities, summon/memosprite actions, and passives where applicable;
- every ability level's coefficients, costs, gains, chances, caps, durations, target program, hit split, Toughness split, retarget and snapshot policies;
- every battle-relevant Trace and minor-stat node;
- six ordered Eidolon patches, including level increases and rule/ability changes;
- rule definitions, state slots, effects, modifiers, native-handler references, bilingual summaries, and provenance.

Eidolon patches apply in ascending order over the validated E0 definition. A patch cannot mutate an unknown field or rely on application order within the same Eidolon unless an explicit operation order is authored.

The character directory currently provides behavioral profiles for the public combat-form catalog. Its implementation matrix is a feature-dependency audit, not coefficient coverage. Rin Tohsaka, Gilgamesh, and any similar officially announced but unavailable entry remain `Disabled` until complete post-release data passes validation; no preview/leak values silently enable them.

## Light Cone data

Light Cone definitions, stat curves, S1-S5 parameters, wearer applicability, and build bindings follow [Character builds, Traces, and equipment](21-build-traces-and-equipment.md).

A complete Light Cone requires identity, rarity/path restriction, level/promotion base-stat rows, Superimposition S1-S5 values, wearer applicability, rule definitions, state/effect teardown, bilingual summary, and provenance.

Superimposition is a level selector over one definition, not five unrelated items. Values that do not scale still appear as explicit equal entries or a validated constant policy. Inventory, ascension materials, acquisition, and gacha are excluded.

## Relic and planar data

Relic/planar definitions, concrete virtual pieces, main-affix curves, sub-affix rolls, set thresholds, and validation policies follow [Character builds, Traces, and equipment](21-build-traces-and-equipment.md).

A complete set requires identity, slot family, piece-count thresholds, each set rule, stack/refresh/snapshot behavior, bilingual summary, and provenance. A complete affix catalog requires slot legality, rarity/tier, enhancement-level main-stat curves, sub-stat roll tiers, caps, units, and provenance.

Combat loadouts may refer to concrete relic instances containing validated set IDs and affix values. Farming, inventory capacity, rerolling, crafting, and drop probability are outside scope.

## Enemy and encounter data

Enemy completeness follows [Enemy AI and encounters](13-enemy-ai-and-encounters.md): exact variant identity, stat curves, defenses/resistances/weaknesses, Toughness layers, abilities, effects, AI graph, phases, summons/links, bilingual summaries, and provenance.

The enemy manifest covers every public enemy and mechanically distinct variant in the snapshot. Encounter coverage is required for universe pools and golden/mechanic fixtures. Reproducing every main-story script is not required unless it supplies the only public executable form of an included enemy mechanic.

## Activity data

Every production-launched battle belongs to an `ActivityDefinition`; Standard content normally uses a one-Battle-node profile. Complete activity data requires:

- graph entry, Sections, Nodes, Edges, conditions, stable priorities, and bounded visits/loops;
- typed Activity/Section/Node/Attempt slots with defaults, bounds, reset, carry, and projection policies;
- participant pools, team slots, eligibility, uniqueness scope, trial/borrow/draft/ban rules, deployment constraints, and loadout lock boundaries;
- decisions/options, modifier inventories, resources, shops/rewards, external outcomes, and checkpoints where used;
- clocks, metrics, score programs, objectives, spawn programs, and terminal outcomes where used;
- each Battle node's encounter/participants/rules/seed bindings and declared `BattleResultProjection`;
- profile/mode metadata, bilingual summaries, provenance, config/manifest digests, and coverage state.

Mode tables may add focused data, aliases, and validation, but reference generic Activity IDs. They cannot redefine command atomicity, graph execution, scopes, clocks, metrics, participant identity, BattleSpec/Result, RNG, replay, or hashing.

## Universe-mode data

Each supported universe profile freezes separate manifests for paths/alignments, blessings and upgrades, curios/states, occurrences/choices, currencies/services, graph/domain modifiers, difficulty/protocol rules, meta-progression effects that change play, enemies/encounters, and its unique components. Run/Plane/Domain aliases compile to Activity/Section/Node.

Unique manifests include Audience Dice and Communing mechanics; customizable dice/faces, Cognition, Knowledge, Secrets and Conundrum; Scepters/Components/slots/synthesis; and current Divergent Universe Equations, Weighted Curios, workbench rules, Golden Blood's Boon, Astronomical Division and related active systems.

Collection rewards, achievements, story entries, account payouts, and historical mode snapshots are not completeness requirements.

## Standard and rotating challenge data

The Standard profile and three classic rotating challenge families use the contracts in [Standard battle and challenge modes](18-standard-and-challenge-modes.md).

- Standard completeness requires the stable profile plus representative single-wave, multi-wave, multi-phase boss, and battle-entry encounter archetypes. It does not require every story/farming stage.
- Memory of Chaos completeness requires the active 4.4 season, Section/Node aliases, exact encounters, participant/loadout locks, cycle policies, objectives, Memory Turbulence rules, and initial resource policies.
- Pure Fiction completeness requires the active 4.4 season, Whimsicality/Cacophony rules, continuous spawn programs, clocks, score programs, thresholds, and encounters.
- Apocalyptic Shadow completeness requires the active 4.4 season, boss-mirage variants, Ruinous Embers/Finality's Axiom rules, AV clocks, boss-progress score programs, thresholds, and encounters.

Past rotating seasons, quick-clear history, calendar rotation, stars already earned, and account rewards are not active-data completeness requirements. Replays may retain archived seasons by digest.

## Rule and data separation

Exact values and content composition live in Excel/Sora data. Reusable battle semantics live in the rule IR; reusable cross-battle semantics live in `starclock-activity`. A content row may select a registered battle/activity handler but cannot introduce Rust source, arbitrary formulas, untyped state, or a mode-specific duplicate operation language.

Canonical decimal strings or scaled integers carry authoritative fractional data. No source/import path may round through an Excel floating cell, JSON `f64`, or locale-formatted number before fixed-point parsing.

## Coverage reports

Generated reports must provide, per category and source snapshot:

- expected manifest IDs and terminal state;
- missing, extra, duplicate, disabled, and stale-version IDs;
- unresolved provenance or bilingual fields;
- unresolved references/native handlers;
- counts by `Cataloged`, `Documented`, `Researching`, `DataReady`, and `GoldenVerified`;
- the exact manifest/config digest used to calculate the report.

The release gate requires 100% manifest accounting, zero enabled incomplete
entries, and `DataReady` for every released entry in the explicitly promised
category/revision. It does not require an Experimental or Candidate mode to
freeze the eventual complete live-game catalog before a mechanic probe can
run. Golden verification has a separately declared target so lack of an
observation fixture is not hidden.

## Goal 01 release status

The [Version 4.4 content reference pack](content-reference/README.md) remains
the normalized source input. Goal 01 has imported its frozen scope through the
authoritative Excel/Sora path: all 88 released combat forms, all 165 released
Light Cones, 17 Standard enemy variants, six encounters, six scenarios and the
Standard profile validate and execute. The generated coverage report records
283/283 character and Light Cone entries as `GoldenVerified`, with two announced
forms retained as disabled audit-only entries outside the denominator.

Universe and rotating-challenge datasets remain outside Goal 01 and retain the
data-pending status defined above. Their absence does not reduce Goal 01
coverage and must not be described as implemented by this release.

## Acceptance audit

- every enabled row traces to at least one source or explicit project policy;
- all public released manifest entries are enabled only when complete;
- announced/unavailable entries are disabled and contain no guessed required values;
- bilingual fields are nonempty and independently summarized;
- IDs and cross-references are stable and unique;
- exact numbers preserve source precision and declared units;
- data changes update version, provenance, coverage report, and affected golden fixtures together.
