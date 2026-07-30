# Goal 16 — Galactic Baseballer Reference Data

## Objective

Prepare a complete, auditable and reproducible Version 4.4 reference-data pack
for both released editions of Legend of the Galactic Baseballer before any
runtime implementation:

- Version 2.2 `Departure` (`启程篇`), the original event;
- Version 3.3 `Demon King` (`魔王篇`), including its retained Version 4.4
  permanent content.

The editions are versioned profiles over one shared mechanic system. Demon King
does not overwrite Departure. Shared identities, edition-specific records and
behavioral differences remain explicit and independently reconcilable.

This goal ends with frozen manifests, normalized mechanics, provenance,
Candidate-quality Excel/Sora authoring data, generated readers, deterministic
exports, coverage and executable semantic review fixtures. It does not
implement a playable Activity, battle handler, CLI, agent API or MCP surface.

## Start condition

Goal 16 may run while other Goals use separate worktrees. It is unblocked when:

- Goal 03 remains `Complete`;
- the pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- Goal 15 is already reserved, so this work uses Goal 16 consistently;
- this checkout has its own branch and worktree;
- `origin` authentication accepts a dry-run push of the Goal 16 branch; and
- all artifacts use the isolated roots declared below.

If research discovers a missing shared runtime primitive, record it as a future
runtime prerequisite. Do not implement it here.

## Frozen snapshot and evidence boundary

- game/content snapshot: Version 4.4;
- planning and launch audit date: 2026-07-30;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93`;
- original-edition official release boundary: Version 2.2 update and the
  publisher's 2024-05-28 event notice;
- Demon King official release boundary: Version 3.3 update published
  2025-05-20;
- retained Version 4.4 boundary: the pinned structured snapshot and released
  permanent-event text present in that snapshot;
- public cross-check access date: recorded per URL during Goal 16.

Only formally released public material is admissible. Leaks, beta dumps,
preview-only values, announced-but-unavailable content and private/NDA
observations are rejected. A Version 4.4 file, name, prefix or adjacent ID is a
discovery seed, not membership evidence.

## Profile and version contract

The pack contains at least these immutable profile identities:

| Profile | Release version | Role |
|---|---|---|
| `galactic-baseballer.departure.v2_2` | 2.2 | Original six-stage rules and content as released. |
| `galactic-baseballer.demon-king.v3_3` | 3.3 | Demon King six-stage rules, advanced synthesis and retained progression. |

Both profiles may reference shared definitions through stable Starclock IDs.
Every shared row records its source path, row locator and evidence digest.
Profile membership is explicit; neither a common display name nor the presence
of a later copy proves replacement, inheritance or reachability.

The Version 4.4 pack must preserve:

- permanent mechanical entry, unlock and profile-selection relationships;
- distinct stage, weapon, accessory, synthesis, strategy, shop and progression
  records where the editions differ;
- shared records without duplication when exact identity is proven;
- permanent reputation/progression mechanics separately from time-limited
  account rewards; and
- edition-specific corrections documented by released patch notes.

## Included content

Completeness is defined by frozen manifests for:

1. both profiles, release versions, entry points, unlock conditions, permanent
   availability and terminal outcomes;
2. all planets/stages, difficulties, phases, waves, elite encounters, bosses,
   initial weapons, team bonuses, recommendations and reachable enemies;
3. defeat-to-experience flow, team levels, exact thresholds and stage-buff
   scaling;
4. upgrade offer generation, stable candidate order, labeled RNG streams,
   weapon/accessory/strategy choices, skips, refreshes and exclusions;
5. duplicate acquisition, maximum levels, slot limits, slot expansion,
   replacement and no-legal-candidate behavior;
6. every weapon, accessory, level, parameter, target rule, trigger, cooldown,
   counter, action source and state owner;
7. resonant accessory relationships and ordinary-to-Legendary synthesis;
8. Demon King Twin and Esteemed/Ultimate synthesis relationships and every
   other released advanced synthesis edge;
9. an explicit acyclic synthesis graph with prerequisites, consumption,
   candidate selection, deterministic ordering and failure behavior;
10. stage phase/wave progression, cycle limits, despawn boundaries, elite
    objectives, boss phases and final settlement;
11. enemy escalation, stage/team bonuses, kill score, boss-damage score,
    remaining-cycle score, final score, rating thresholds and clear conditions;
12. weapon interactions with Basic ATK, Skill, Ultimate, Follow-Up Attack, DoT,
    Weakness Break, summon/memosprite and other released trigger sources;
13. Cosmic Reputation, Raccoon Tokens/Gold, Cosmic Store, mechanically
    relevant upgrades, Adventure Index and Adventure Strategies;
14. inherited character/enemy identities and formula references without
    copying or modifying their owning partitions;
15. bilingual names, independent short summaries, provenance, confidence,
    approximation records, review fixtures and exact-once coverage.

## Excluded content

- runtime lowering, Activity/battle handlers, controllers, CLI, agent or MCP;
- changes to shared combat, build, activity, RNG, replay or formula behavior;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, materials, selectable characters, avatars, achievements and
  account-reward payloads, except bounded locators needed to prove entry,
  unlock or permanent-versus-limited separation;
- live calendar services and wall-clock behavior beyond immutable release
  metadata;
- inferred synthesis edges based on names, pictures or ID ranges;
- preview, beta, leaked, private or announced-but-unreleased content;
- a Released runtime profile or production compatibility claim.

## Architecture and artifact isolation

Goal 16 starts in the `Experimental` lane and may finish with a complete
`Candidate` reference bundle. It owns only:

```text
content-manifests/galactic-baseballer-v1/
content-reference/galactic-baseballer-v1/
config/galactic-baseballer/
config/galactic-baseballer-generated/
tools/galactic-baseballer-reference/
evidence/galactic-baseballer-reference-v1/
```

It must not modify or regenerate:

- `config/generated/`;
- `config/universe-generated/`;
- `config/gold-and-gears-generated/`;
- `config/swarm-disaster-generated/`;
- `config/unknowable-domain-generated/`;
- `config/divergent-universe-generated/`;
- `config/currency-wars-generated/`;
- `config/anomaly-arbitration-generated/`;
- another mode's manifests, normalized data, workbooks, tools or evidence; or
- Goals 01–15 historical ledgers, completion evidence and immutable snapshots.

Reference rows may describe future generic Activity or battle contributions.
They do not become executable programs in this goal.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`. Priority is:

1. pinned released structured tables and configuration/ability programs;
2. official released publisher text and patch notes;
3. reproducible released-version observations;
4. independent public cross-checks.

Every factual row records repository or URL, exact revision or access date,
game version, path/page, stable row locator, evidence digest, quality and note.
Allowed quality labels are:

- `ExactStructured`;
- `ExactPublicText`;
- `Observed`;
- `ApproximateFromReleasedText`;
- `ProjectPolicy`.

Approximation is field-level. Every unavailable weight, candidate order, RNG
draw boundary, trigger timing, snapshot rule, rounding rule, replacement rule
or failure behavior records:

- the unavailable fact;
- the selected deterministic behavior;
- at least two rejected alternatives;
- rationale;
- affected fixtures;
- field confidence; and
- a concrete replacement condition.

## Normalized reference families

Phase 0 freezes the exact machine schema. At minimum it must account for:

- profile, release, entry, unlock and retention;
- stage, difficulty, team bonus, recommendation and initial weapon;
- phase, wave, spawn slot, elite objective, boss phase and escalation;
- team level, experience threshold and stage-buff level;
- offer rule, candidate member, skip, refresh, exclusion and RNG stream;
- inventory slot, expansion, replacement and duplicate-level operation;
- weapon, weapon level, trigger, target, action, counter and cooldown;
- accessory, accessory level, contribution and resonance;
- synthesis recipe, ingredient, result and graph audit;
- Adventure Strategy, offer, level and contribution;
- currency, store upgrade, price, rank, unlock and persistent modifier;
- score term, rating, objective, clear and settlement;
- source, reconciliation, approximation, mechanic rule, review fixture,
  coverage and pack index.

Definitions remain separate from mutable inventory, level, offer, wave,
counter, currency and score state. Exact decimals are canonical strings.
Semantic sequences preserve declared order; set-like collections sort by
stable fixed-width identity.

## Semantic review contract

Every independent mechanism family has at least one `ReferenceOnly` mechanic
rule and one executable semantic review fixture. Each rule/fixture pair names:

- trigger point and owner domain;
- owned state and snapshot boundary;
- preconditions;
- ordered typed review operations;
- concrete input records;
- expected facts and rejection behavior;
- source and mechanism quality; and
- linked approximation/replacement records.

The minimum families are:

1. profile/version selection;
2. stage/difficulty selection;
3. wave/phase progression;
4. experience/team upgrade;
5. random upgrade candidates;
6. weapon acquire/duplicate upgrade;
7. accessory acquire/duplicate upgrade;
8. slot cap/expansion/replacement;
9. autonomous weapon action;
10. character-action-triggered weapon;
11. resonant accessory binding;
12. Legendary synthesis;
13. Twin synthesis;
14. Esteemed/Ultimate synthesis;
15. Adventure Strategy;
16. team bonus;
17. Cosmic Store progression;
18. score/rating/clear;
19. boss phase/final settlement;
20. no-legal-candidate and rejection invariance.

## Excel and Sora authoring contract

- `.xlsx` is the only editable production authoring surface.
- Complete new workbooks are generated with the documented Python path and
  `openpyxl==3.1.5`; an `.xlsx` is never patched as a ZIP.
- Sora CLI 0.3.0 is the only schema validation, template, Rust code-generation
  and production export authority.
- The isolated project owns its schema lock, templates, generated Rust readers,
  binary `.sora` bundle and debug export.
- JSON is research staging or debug output only; runtime loading is forbidden.
- Generation runs twice in clean isolated targets and proves byte identity.
- Every sheet and every schema-field column is rendered and visually reviewed.
- A standalone reader loads every table and every row from the binary bundle.
- Template generation never overwrites a designer-edited workbook.

## Delivery phases

### Phase 0 — Scope, versions, sources, denominator and contracts

| Batch | Deliverable |
|---|---|
| `G16-P0-B1` | Audit prerequisites, Goal numbering, branch/worktree/remote isolation and source caches; freeze plan, ledger, prompt and foundation policy. |
| `G16-P0-B2` | Generate the focused Version 4.4 inventory for both editions, official notices, TextMaps, stage/config/ability programs and shared enemy identities. |
| `G16-P0-B3` | Freeze explicit profile membership, per-category denominators, shared reachability, limited-reward exclusions and reconciliation receipts. |
| `G16-P0-B4` | Freeze normalized schemas, canonical encoding, workbook/table families, approximation records and the 20-family semantic fixture contract. |

### Phase 1 — Departure profile and shared base system

| Batch | Deliverable |
|---|---|
| `G16-P1-B1` | Import Departure profile, entry/unlock, six stages, difficulties, initial weapons, team bonuses and recommendations. |
| `G16-P1-B2` | Import every Departure weapon/accessory, levels, parameters, triggers, targets, resonances and Legendary recipes. |
| `G16-P1-B3` | Import experience, team levels, offers, duplicate upgrades, slots, refresh/skip/exclusion and deterministic failure boundaries. |
| `G16-P1-B4` | Import Departure phases, waves, enemies, elites, bosses, escalation, score/rating/clear rules and review fixtures. |

### Phase 2 — Demon King differences and complete progression

| Batch | Deliverable |
|---|---|
| `G16-P2-B1` | Import Demon King profile, entry/retention, six stages and every explicit difference from the shared base/Departure profile. |
| `G16-P2-B2` | Import Demon King weapons/accessories and complete Legendary, Twin and Esteemed/Ultimate synthesis graphs. |
| `G16-P2-B3` | Import Adventure Strategies, Raccoon Gold, Cosmic Reputation, Cosmic Store and mechanically relevant persistent upgrades. |
| `G16-P2-B4` | Import all Demon King phases, waves, enemies, servants, bosses, score/settlement rules and semantic fixtures. |

### Phase 3 — Evidence closure and isolated authoring

| Batch | Deliverable |
|---|---|
| `G16-P3-B1` | Close every frozen source obligation, reconciliation row and approximation boundary at 100% DataReady without reducing denominators. |
| `G16-P3-B2` | Generate complete no-overwrite openpyxl workbooks twice; validate structure and visually review every sheet/column. |
| `G16-P3-B3` | Add isolated Sora schemas/templates/lock/readers and prove clean byte-identical double generation. |
| `G16-P3-B4` | Export binary/debug bundles, run drift checks and load every table/row through the standalone generated reader. |

### Phase 4 — Audit, semantic execution and Candidate freeze

| Batch | Deliverable |
|---|---|
| `G16-P4-B1` | Execute all mechanism-family fixtures, approximation replacement checks and rejection-without-mutation reviews. |
| `G16-P4-B2` | Audit cross-profile differences, shared identities, synthesis DAGs and isolation from Standard/other modes/runtime bundles. |
| `G16-P4-B3` | Run full source-cache regeneration, workbook/Sora drift, reader, repository and clean-checkout acceptance. |
| `G16-P4-B4` | Freeze Candidate evidence, final counters/digests and ledger; push and verify the terminal commit, then complete the persistent Goal. |

## Execution and publication rules

- Select the earliest unblocked `Pending` batch.
- Keep only one Goal 16 batch `InProgress`.
- Complete each batch's data, evidence, validators, authoring changes and ledger
  update together.
- Never reduce a frozen denominator to make coverage pass.
- Run focused checks plus `node tools/repository-check/run.mjs` for each batch.
- Run the full source-cache, double-generation, Sora, reader and clean-checkout
  gates at phase boundaries and final release.
- If an unrelated historical snapshot blocks a full repository gate, record the
  exact command/error and still pass the complete isolated Goal 16 acceptance.
- Do not stop at a plan, partial inventory, workbook or single export.

Each batch is one Conventional Commit:

```text
data(galactic-baseballer): G16-Px-By <imperative summary>
docs(galactic-baseballer): G16-Px-By <imperative summary>
```

After every commit:

```text
git push origin codex/goal16-galactic-baseballer-reference
```

Local `HEAD`, the tracking ref and
`git ls-remote origin refs/heads/codex/goal16-galactic-baseballer-reference`
must match before the next batch begins. No pull request or history rewrite is
authorized.

## Terminal gates

- Both released editions have explicit, independently reconcilable profiles.
- Every frozen weapon, accessory, level, trigger and synthesis relation is
  DataReady or field-level policy-bound.
- Every stage, difficulty, phase, wave, enemy, boss, bonus and score rule closes.
- Upgrade offers, slots, refresh, skip, replacement and failure boundaries close.
- Mechanical permanent progression closes; account rewards remain excluded.
- Frozen coverage is 100% DataReady with no blocking research row.
- All 20 mechanism families have rules and executable review fixtures.
- Every approximation has alternatives, rationale, affected fixtures,
  confidence and replacement condition.
- Workbooks pass no-overwrite, structural, semantic and visual QA.
- Sora outputs regenerate without drift and the reader loads every row.
- No Goal 16 content enters another mode or a production runtime bundle.
- Full isolated and clean-checkout acceptance pass.
- Candidate evidence is frozen, the final commit is pushed, remote equals local
  `HEAD`, and the persistent Goal is marked `Complete`.
