# Documentation Coverage Matrix

This matrix reports the current documentation milestone. It does not claim that Rust code, Excel workbooks, Sora bundles, or complete Version 4.4 numerical content already exist.

## Status vocabulary

- **Normative**: sufficient to implement without choosing a different architectural policy.
- **Research baseline**: formula/behavior is documented, but observed edge cases or exact live-version values still require fixtures.
- **Behavioral catalog**: public identities and high-level mechanics are described; executable rows are not complete.
- **Specified, data pending**: schema, boundary, and acceptance criteria are defined; the frozen row manifest/data import remains outstanding.
- **Out of scope**: intentionally excluded from the project target.

## Core and architecture

| Area | Document | Current status | Exit gate |
|---|---|---|---|
| Battle state, actions, targets, events | `01`, `10` | Normative | Implementation/state-machine golden tests. |
| Timeline, AV, turns, interrupts | `02`, `10` | Research baseline | Equal-AV and multi-interrupt observation fixtures. |
| Damage, healing, shields | `03`, `12` | Research baseline | Fixed-point golden vectors and kit-specific snapshot fixtures. |
| Toughness, Break, Super Break, layers | `04`, `12` | Research baseline | Provider-specific Exo/multi-layer rows and observations. |
| Effects, resources, aggro | `05`, `12` | Research baseline | Full content validation plus targeting/effect fixtures. |
| Workspace and APIs | `06`, `08` | Normative | Compiling workspace obeying dependency/visibility/file-size gates. |
| Excel/Sora pipeline | `07` | Normative design | Pinned Sora project, templates, workbooks, generated readers, and bundle. |
| Fixed point, RNG, hashing | `09`, `16` | Normative | Cross-platform codec/RNG/hash golden vectors. |
| Rule IR/native handlers | `11` | Normative | Schema plus interpreter/registry tests. |
| Enemy AI/encounters | `13` | Specified, data pending | Frozen enemy manifest and all released entries `DataReady`. |
| Activity core/mode extension | `19` | Normative | Generic graph/scope/roster/persistence/clock/metric/BattleSpec tests plus future-mode fixtures. |
| Universe profiles | `14`, `19` | Specified, data pending | One complete manifest and seeded golden activity per supported family. |
| Standard/challenge profiles | `18`, `19` | Normative profile design; seasonal data pending | Standard golden activities plus active 4.4 manifests/golden stages for all three challenge families. |
| Content/provenance/coverage | `15` | Normative | Generated digest-addressed coverage report. |
| Replay/CLI/engine adapters | `16` | Normative design | Replay verifier, CLI smoke tests, adapter determinism test. |

## Content data

| Category | Identity/behavior coverage today | Exact executable data today | Required terminal gate |
|---|---|---|---|
| Released character combat forms | 88 compact E0 behavioral profiles | Not imported | Complete stats, abilities, levels, Traces, Techniques, hit plans, Eidolons, rules, provenance; all `DataReady`. |
| Officially announced forms | 2 disabled behavioral placeholders | Intentionally absent | Stay `Disabled` until public release and full validation; no guessed values. |
| Light Cones | Schema/boundary only | Not imported | Frozen 4.4 manifest, stats/promotions, S1-S5 rules, provenance; all released entries `DataReady`. |
| Relic and planar sets | Schema/boundary only | Not imported | Frozen 4.4 manifest, piece effects, main/sub-affix curves and provenance. |
| Enemies and bosses | Behavior/AI/phase model only | Not imported | Frozen variant manifest, exact stats/skills/AI/phases/summons and provenance. |
| Standard Simulated Universe | System boundary documented | Not imported | Full active permanent content manifest and seeded golden run. |
| Swarm Disaster | System boundary documented | Not imported | Full active content, dice/map/progression mechanics and seeded golden run. |
| Gold and Gears | System boundary documented | Not imported | Full active content, custom dice/Cognition/Secrets/Conundrum and seeded golden run. |
| Unknowable Domain | System boundary documented | Not imported | Full Scepter/Component/alignment/stage data and seeded golden run. |
| Divergent Universe 4.4 | Snapshot boundary documented | Not imported | Frozen current-only manifest, full mechanical data and seeded golden run. |
| Standard battle | Default profile documented | Not imported | Representative standard encounter archetypes and golden battles. |
| Memory of Chaos 4.4 | Family, clock, node, and objective contracts documented | Not imported | Active season stages, encounters, turbulence, clock/objective data and golden stage. |
| Pure Fiction 4.4 | Family, spawn, clock, buff, and score contracts documented | Not imported | Active season stages, Whimsicality/Cacophony, spawn/score data and golden stage. |
| Apocalyptic Shadow 4.4 | Family, boss-progress, AV, buff, and score contracts documented | Not imported | Active season boss variants, Embers/Axiom, AV/score data and golden stage. |
| Future activity archetypes | Generic contracts documented | No profiles imported | Golden survival, defense, boss-rush, trial/borrow, draft/ban, multi-team fork/join, and endless-under-budget fixtures without `combat-core` changes. |

No row in the “Not imported” column may be described as implemented or complete in issues, release notes, README badges, or CLI output.

## Formula confidence work queue

The following require explicit observation/golden work even after table import:

- internal rounding boundaries that differ from the project default;
- exact equal-AV and simultaneous interrupt ordering;
- snapshot behavior for individual DoTs, shields, delayed attacks, fields, summons, and memosprites;
- target invalidation and cross-wave behavior of individual multi-hit actions;
- provider-specific Exo-Toughness and multi-layer routing;
- boss phase carryover and unusual shared-HP components;
- mode-specific formula replacements, caps, and historical-to-current adjustments.

Unverified values use `Researching` status with a conflict note. They do not inherit a convenient default without a documented project-policy label.

## Documentation acceptance

This documentation milestone is complete when:

- all documents agree on lifecycle, fault, wave, Activity/Section/Node/Attempt scopes, numeric, RNG, source, and coverage vocabulary;
- every architecture subsystem has an owner, dependency direction, public boundary, validation rules, and tests;
- every promised content category has a normalized data contract and terminal coverage gate;
- local Markdown links resolve;
- statements about current implementation/data coverage match this matrix.
