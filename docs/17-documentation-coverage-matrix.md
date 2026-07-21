# Documentation Coverage Matrix

This matrix reports the Goal 01 release boundary. Rust code, Excel workbooks,
Sora bundles and the frozen Version 4.4 Goal 01 content scope are implemented;
future universe, relic/planar and rotating-challenge datasets remain explicitly
outside this release.

## Status vocabulary

- **Normative**: sufficient to implement without choosing a different architectural policy.
- **Research baseline**: formula/behavior is documented, but observed edge cases or exact live-version values still require fixtures.
- **Behavioral catalog**: public identities and high-level mechanics are described; executable rows are not complete.
- **Specified, data pending**: schema, boundary, and acceptance criteria are defined; the frozen row manifest/data import remains outstanding.
- **Out of scope**: intentionally excluded from the project target.

## Core and architecture

| Area | Document | Current status | Exit gate |
|---|---|---|---|
| Battle state, actions, targets, events | `01`, `10` | Implemented and golden-verified | Repository combat and replay suites. |
| Timeline, AV, turns, interrupts | `02`, `10` | Research baseline | Equal-AV and multi-interrupt observation fixtures. |
| Damage, healing, shields | `03`, `12` | Research baseline | Fixed-point golden vectors and kit-specific snapshot fixtures. |
| Toughness, Break, Super Break, layers | `04`, `12` | Research baseline | Provider-specific Exo/multi-layer rows and observations. |
| Effects, resources, aggro | `05`, `12` | Research baseline | Full content validation plus targeting/effect fixtures. |
| Workspace and APIs | `06`, `08` | Implemented and frozen | Dependency, visibility, source-size and public-API audits. |
| Core implementation ownership and resolver design | `20` | Normative | Compiling aggregate, stores, lowering, transaction, operation/event pipeline, and command-to-hash vertical slice. |
| Character build, Trace, Light Cone, relic compilation | `21` | Normative design | Compiling independent `starclock-build` catalog/compiler plus E0/E6, S1-S5, affix/set, preset, generic combat-output, lock, source-attribution, and cross-platform digest tests. |
| Excel/Sora pipeline | `07` | Implemented and frozen | Pinned Sora project, templates, workbooks, generated readers, and bundle. |
| Fixed point, RNG, hashing | `09`, `16` | Implemented and golden-verified | Native matrix execution plus alternate-CPU compile evidence. |
| Rule IR/native handlers | `11` | Implemented and audited | Typed interpreter tests and zero production native handlers. |
| Enemy AI/encounters | `13` | Goal 01 subset implemented | 17 frozen variants, six encounters and six scenarios execute. |
| Activity core/mode extension | `19` | Normative | Generic graph/scope/roster/persistence/clock/metric/BattleSpec tests plus future-mode fixtures. |
| Universe profiles | `14`, `19` | Specified, data pending | One complete manifest and seeded golden activity per supported family. |
| Standard/challenge profiles | `18`, `19` | Normative profile design; seasonal data pending | Standard golden activities plus active 4.4 manifests/golden stages for all three challenge families. |
| Content/provenance/coverage | `15` | Implemented and frozen | Generated digest-addressed 283/283 report. |
| Replay/CLI/engine adapters | `16` | CLI/replay implemented; engine adapter future | Replay verifier and CLI contracts pass. |

## Content data

| Category | Identity/behavior coverage today | Prepared data today | Required terminal gate |
|---|---|---|---|
| Released character combat forms | 88 frozen forms | Imported through Excel/Sora with statistics, abilities, Traces, Techniques, Eidolons, rules and goldens | 88/88 `GoldenVerified`. |
| Officially announced forms | 2 disabled behavioral placeholders | Intentionally absent | Stay `Disabled` until public release and full validation; no guessed values. |
| Light Cones | 165 frozen definitions | Imported through Excel/Sora with stat curves, promotions, S1-S5 selections, rules and goldens | 165/165 `GoldenVerified`. |
| Relic and planar sets | Detailed set/affix/piece schema | Not imported | Frozen 4.4 manifest, piece effects, main/sub-affix curves and provenance. |
| Enemies and bosses | Behavior/AI/phase model plus prepared source evidence | Goal 01 imports the exact 17-variant Standard subset and 63 source-bound abilities | 17/17 frozen variants verified; broader catalogs remain future scope. |
| Standard Simulated Universe | System boundary documented | Not imported | Full active permanent content manifest and seeded golden run. |
| Swarm Disaster | System boundary documented | Not imported | Full active content, dice/map/progression mechanics and seeded golden run. |
| Gold and Gears | System boundary documented | Not imported | Full active content, custom dice/Cognition/Secrets/Conundrum and seeded golden run. |
| Unknowable Domain | System boundary documented | Not imported | Full Scepter/Component/alignment/stage data and seeded golden run. |
| Divergent Universe 4.4 | Snapshot boundary documented | Not imported | Frozen current-only manifest, full mechanical data and seeded golden run. |
| Standard battle | Default profile implemented | Six frozen encounters and scenarios imported and seeded to `Won` | Goal 01 Standard gate complete. |
| Memory of Chaos 4.4 | Family, clock, node, and objective contracts documented | Not imported | Active season stages, encounters, turbulence, clock/objective data and golden stage. |
| Pure Fiction 4.4 | Family, spawn, clock, buff, and score contracts documented | Not imported | Active season stages, Whimsicality/Cacophony, spawn/score data and golden stage. |
| Apocalyptic Shadow 4.4 | Family, boss-progress, AV, buff, and score contracts documented | Not imported | Active season boss variants, Embers/Axiom, AV/score data and golden stage. |
| Future activity archetypes | Generic contracts documented | No profiles imported | Golden survival, defense, boss-rush, trial/borrow, draft/ban, multi-team fork/join, and endless-under-budget fixtures without `starclock-combat` changes. |

Rows that remain “Not imported” are outside Goal 01 and may not be described as
implemented or complete in release notes, README badges or CLI output.

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
