# Goal 12 — Currency Wars Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Currency Wars reference-data pack
and an isolated Excel/Sora authoring surface before any Currency Wars runtime
implementation.

The goal covers the released three-Plane and Node flow, Standard Gambit and
Overclock Gambit, difficulties and rank-relevant mechanical boundaries, Squad
HP and action-value battle limits, recruitment and Store economy, roster size,
positioning, Character Empowerment, Bonds, character costs, star upgrades,
owned/trial build mapping, equipment, Investment Environments and Investment
Strategies, reachable Blessings, Curios, events, services, encounters and every
battle-visible or cross-battle rule contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated
readers, coverage and semantic review fixtures. It does not implement or expose
a playable Currency Wars profile.

## Start condition

Goal 12 may run while Goal 07 is completing Standard Simulated Universe
mechanics and Goals 08 through 11 are collecting mode reference data. It is
unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses a separate branch and git worktree from every other active
  Goal checkout; and
- Currency Wars artifacts use the isolated paths declared below.

Goal 07, 08, 09, 10 or 11 completion is not a prerequisite for source
collection, manifest freezing, normalization, evidence work, isolated schemas
or workbook authoring. Goal 12 must not modify their plans, ledgers, manifests,
normalized rows, workbooks, generated output, runtime lowering or shared
combat/activity behavior.

If research discovers a missing shared runtime primitive, record it as a future
Currency Wars runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and source-reproduction prerequisite

- game/content snapshot: Version 4.4;
- planning audit date: 2026-07-29;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- existing audit baseline:
  `content-manifests/standard-universe-v1/source-inventory.json`;
- cross-mode ownership inputs: committed Goal 08, 09, 10 and 11 manifests at
  the revisions inspected by `G12-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 12 rather than
  silently inheriting an earlier research date.

The planning audit found clean local caches at both required commits, proved
the commit objects readable, verified each configured origin and ran Git
connectivity checks. `G12-P0-B1` must reproduce those checks in its own
worktree before the first Goal data mutation. Planning-time availability is not
batch-owned release evidence.

## Starting source oracle

### Currency Wars selectors

The published `G12-P0-B1` through `G12-P1-B2` history began from a
`TournRogue` / `Tourn3` hypothesis. `G12-P1-B3` source discovery found the
stronger direct selector below and triggered the `G12-R01` replacement
condition:

```text
GuideRogueTab:
  ID = 1003
  GuideType = GridFight
  Name = Currency Wars / 货币战争

GuideRogueData:
  ID = 301
  TabID = 1003
  Name = Currency Wars / 货币战争
```

`GridFight` is the authoritative Version 4.4 source selector. The older
`TournRogue` / `Tourn3` / module 6002201 rows identify Divergent Universe and
remain only as immutable historical and exclusion evidence. `G12-P0-B5`
regenerates the source inventory, manifest and contracts from this correction;
`G12-P1-B10` replaces normalized flow rows imported under the superseded
selector.

### Mode-focused tables

The pinned Git tree contains 153 `ExcelOutput/GridFight*.json` tables. They
cover the direct flow, stage, route, node, difficulty, role, cost, shop,
experience, team-size, star, position, Bond/Trait, equipment, Augment,
Projection, event/service, enemy and settlement authoring surfaces. Every row
is retained as an exact disposition obligation; a table prefix still does not
prove that every season, tutorial, expired or presentation row is enabled.

The same tree contains 984 `Config` paths whose case-insensitive path contains
`GridFight`, including character, battle-event, AI, ability, level and preload
programs. The inventory retains every one of the 1,137 combined GridFight
paths before applying row-level reachability.

All eleven `RoguePersona` and sixty-four `RogueTourn` tables remain available
only as Divergent Universe/exclusion reconciliation evidence. They cannot
contribute Currency Wars content without an explicit reference originating in
the GridFight closure.

### Configuration, TextMap, Stage and ability seeds

The direct Version 4.4 program seeds are the complete 984-file GridFight
configuration closure. Layout companions are evidence inputs, not independent
mechanic rows. Version directories such as `3.5`, `4.0`, `4.2` and `4.4`
describe content additions inside the released 4.4 snapshot; they are not by
themselves release or ownership selectors.

The focused inventory must also include:

- `TextMap/TextMapCHS.json` and `TextMap/TextMapEN.json`;
- applicable bilingual `StarRailRes` simulated Blessing, Curio, event and item
  indexes;
- concrete `ExcelOutput/StageConfig.json` rows selected through enabled
  GridFight stage, route, node, formation-wave and monster relationships;
- `RogueUpgradeAvatar`, `RogueUpgradeAvatarConst`,
  `RogueUpgradeAvatarEquipment`, `RogueUpgradeAvatarSubRelic`,
  `RogueUpgradeAvatarSubType` and `RogueUpgradeAvatarSubValue` only when an
  explicit GridFight role/build mapping proves reachability;
- reachable `AvatarConfig`, skill/Trace/Eidolon, Light Cone, relic,
  `RogueMazeBuff`, `RogueBuff`, `RogueMiracle`, occurrence, shop,
  `RogueMonster`, `RogueMonsterGroup`, wave, monster skill/status and enemy
  ability records; and
- every transitively invoked level, battle-event, modifier, maze, group
  template and ability program.

Every shared-source reconciliation record uses source path, stable row locator
and evidence digest. The manifest must classify each discovered row as
`CurrencyWars`, `Shared`, `EvidenceOnly` or an explicitly named excluded
mode/module. File presence, prefix, numerical adjacency and name equality are
never sufficient ownership or reachability evidence.

## Included content

Completeness is defined by frozen manifests for:

1. mode entry, unlocks, initial resources, Standard/Overclock Gambit selection
   and terminal outcomes;
2. all released Version 4.4 difficulties, ranks and enemy-affix inputs that
   change legal entry, encounters, state or battle contributions;
3. all three Planes, Nodes, rooms, Domain/composition types, transitions,
   entry/terminal conditions and carry/reset rules;
4. Squad HP initialization, loss, recovery, bounds, run-failure boundary and
   battle-result projection;
5. battle action-value limits, timeout resolution, victory/defeat ordering and
   Squad HP loss;
6. Gold Coins, team Experience, Store refresh resources, team-size levels,
   prices, income, spend/refund and deterministic candidate policies;
7. every recruitable character, cost tier, offered-copy pool, purchase, bench,
   field, sale and roster-cap lifecycle;
8. one-, two- and three-star states, exact three-copy combination behavior,
   stat/effect scaling, overflow and replacement rules;
9. on-field, off-field and on/off-field positioning, validation, movement,
   deployment and Character Empowerment activation/teardown;
10. every Bond, membership, activation threshold, level, contribution,
    recomputation boundary and tie/order rule;
11. character-owned and trial fallback builds, level/Trace/Light Cone/relic
    mapping, refresh and teardown without mutating account inventory;
12. off-field Eidolon and signature Light Cone conversion, up to three
    equipment slots, equipment eligibility, replacement and lifecycle;
13. automatic Technique use, reduced defeat-energy gain, lethal-damage rescue,
    battle countdown reduction and every other Currency Wars battle override;
14. Investment Environments, Investment Strategies and their GridFight
    Augment/Projection/Talent/equipment equivalents, offers and activated
    lifecycle where the manifest proves reachability;
15. Currency Wars-owned and reachable shared Blessings, enhanced levels,
    formula/Equation-like combinations, Curios/Miracles/Hex states, events,
    choices, currencies, services and candidate pools;
16. mechanically relevant permanent progression only where it changes legal
    entry, starting state, available choices, enemies or battle contributions;
17. exact enemy variants, StageConfig rows, waves, elite/boss choices, final
    bosses and difficulty/Plane bindings;
18. every battle-visible rule contribution, cross-battle state slot, RNG
    candidate set and lifecycle boundary;
19. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

A category may close at zero only when the generated manifest proves that the
released GridFight selector and its complete transitive closure make no such
record reachable. Raw shared-table presence cannot force membership, and an
unresolved join cannot be treated as an empty pool.

Shared content is referenced by stable Starclock identity only after the frozen
Currency Wars manifest proves reachability. A source copy with a mode-specific
effect, state, pool rule, mapped build, star level or display binding remains a
distinct Currency Wars-owned record.

## Excluded content

- Currency Wars runtime lowering, handlers, controllers, CLI, Agent, MCP or
  full playable runs;
- changes to `starclock-activity`, `starclock-combat`, `starclock-build` or
  shared runtime semantics;
- Standard Simulated Universe, Gold and Gears, Swarm Disaster, Unknowable
  Domain, Divergent Universe, historical temporary modes and non-GridFight
  modules except as explicit ownership/exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, passes, achievements, collections, weekly points, rank rewards,
  first-clear/account payouts and Bond-chain rewards;
- social profiles, sharing codes, strategy recommendation presentation and
  automated UI/assist behavior;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Story, collection, display, weekly, rank and reward tables may be retained as
provenance locators when necessary to prove a mechanical unlock, module
boundary, stage relationship or offered choice. Their prose, presentation and
account-reward payloads do not become normalized runtime content.

## Architecture and artifact isolation

Goal 12 starts in the `Experimental` content lane and may finish with a complete
`Candidate` reference bundle. It does not promote a `Released` production mode.

Use isolated paths:

```text
content-manifests/currency-wars-v1/
content-reference/currency-wars-v1/
config/currency-wars/
config/currency-wars-generated/
tools/currency-wars-reference/
evidence/currency-wars-reference-v1/
```

The authoring workbooks are mode-owned:

```text
CurrencyWars.xlsx
CurrencyWarsBindings.xlsx
CurrencyWarsReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G12-P0-B4`. They must not share mutable sheets or generated directories with
any other mode workbook, any other mode-generated directory or
`config/generated/`.

Currency Wars authoring may reference generic Activity/build concepts and
stable shared content IDs. It may not redefine Activity graph execution,
scopes, command atomicity, BattleSpec/Result, build resolution, combat
formulas, RNG, hashing or replay. Reference rows may declare a future activity,
build or battle rule contribution without implementing its evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released ability/configuration programs;
2. official publisher announcements and released in-game text;
3. reproducible live observations;
4. independent public community cross-checks.

Every normalized fact records the repository/URL, exact revision or access
date, game version, relative path/page, row locator, evidence digest, quality,
mechanism quality and note. Allowed labels remain:

- `ExactStructured`;
- `ExactPublicText`;
- `Observed`;
- `ApproximateFromReleasedText`;
- `ProjectPolicy`.

Approximation is field-level. Hidden shop weights, candidate/target ordering,
copy/star timing, simultaneous Bond changes, Squad HP projection, action-value
boundary ordering, rounding, caps, fallbacks and pool probabilities must never
be silently inferred. A deterministic project policy requires the known facts,
selected behavior, rejected alternatives, rationale, affected fixtures and a
concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G12-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, Gambit mode, entry, difficulty, rank, Plane, Node, room, Domain and
  finish condition;
- Squad HP, battle action-value limit, timeout result and battle projection;
- roster character, position, cost, shop offer, purchase/sale, bench/field and
  team-size level;
- star state, copy count, combination, overflow, stat/effect scaling and
  teardown;
- Character Empowerment, Bond, threshold, level, member and rule contribution;
- owned/trial build mapping, equipment slot/item and off-field
  Eidolon/Light-Cone conversion;
- Gold Coin, team Experience, Store refresh and service operation;
- Investment Environment/Strategy and direct GridFight
  Augment/Portal/Orb/Projection/Talent/enhancement families;
- Blessing/level/formula, Curio/Miracle/Hex state, occurrence/choice and
  service;
- encounter pool, group, StageConfig wave, enemy slot and boss;
- mechanic rules, sources, reconciliation receipts, coverage, review fixtures
  and pack index.

Definitions remain separate from levels, mutable states, variants, offers,
mapped builds, roster instances and unlock conditions. Exact decimals use
canonical strings. Arrays that are sets are sorted by stable ID; shop offers,
copy combination, stage paths, choice/effect programs and other semantic
sequences preserve declared order.

## Parallel execution rules

When Goals 07 through 12 execute concurrently:

- use separate git worktrees and branches;
- Goal 12 owns only the isolated paths declared above plus its three Goal
  documents and index row;
- do not edit Goal 07, 08, 09, 10 or 11 plans, ledgers, manifests, policies,
  evidence, workbooks or content partitions;
- do not edit another mode's normalized rows or generated output;
- do not regenerate another mode's generated directory or
  `config/generated/`;
- do not reclassify a shared `RogueTourn`, `RoguePersona` or base source row in
  place; record Goal 12 ownership and reachability in its own manifest;
- compare overlapping rows by source path, stable row locator and evidence
  digest at the named reconciliation checkpoint;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another goal; and
- preserve Goal 03 and current production bundle digests.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed fully
in parallel. Runtime lowering and changes to shared schemas or operations
belong to a later goal after the relevant reference packs are frozen.

## Delivery phases

### Phase 0 — Scope, source files, manifests and contracts

| Batch | Deliverable |
|---|---|
| `G12-P0-B1` | Reproduce both pinned caches, verify Goal 03, freeze Goal 12 scope/exclusions, inspect active Goal 08/09/10/11 ownership boundaries and prove branch/worktree/path isolation. |
| `G12-P0-B2` | Generate a focused source inventory covering all 11 `RoguePersona` tables, every `Tourn3`-selected row across all 64 `RogueTourn` tables, direct/transitive config and ability programs, CHS/EN TextMaps, StageConfig rows, shared build/Rogue/enemy data and named exclusions. |
| `G12-P0-B3` | Freeze exact per-category manifest IDs/counts, enabled Version 4.4 selectors, ownership, shared reachability and the Currency Wars versus Divergent/other-mode/module exclusion boundary. |
| `G12-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families, reconciliation receipts and semantic review-fixture contracts. |
| `G12-P0-B5` | Correct the authoritative selector to `GuideType = GridFight`, retain all 1,137 GridFight table/config paths, regenerate manifests/contracts and preserve the published Tourn3 hypothesis as immutable historical exclusion evidence. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G12-P1-B1` | Import profile, Standard/Overclock entry, difficulties/ranks, three Planes, Nodes, rooms, Domain compositions, legal flow, finish and carry/reset rules. |
| `G12-P1-B2` | Import Squad HP, action-value battle limits, timeout/victory projection, loss/recovery, run-failure and same-boundary ordering. |
| `G12-P1-B10` | Replace the superseded Tourn3/Persona flow import with exact GridFight stage, route, node, difficulty and transition closure before any later Phase 1 import resumes. |
| `G12-P1-B3` | Import the recruitable roster, cost tiers, shop offers, purchase/sale, Gold Coins, refreshes, team Experience, team-size levels, bench/field caps and candidate rules. |
| `G12-P1-B4` | Import positioning, Character Empowerment, automatic Techniques, defeat-energy scaling, lethal rescue/countdown behavior and battle-visible overrides. |
| `G12-P1-B5` | Import every Bond, member set, activation threshold, level, recomputation boundary and run/battle contribution. |
| `G12-P1-B6` | Import one-/two-/three-star states, three-copy combination, stat/effect scaling, overflow, roster replacement and teardown. |
| `G12-P1-B7` | Import owned/trial build mapping, level/Trace/Light Cone/relic substitution, off-field Eidolon/signature conversion and three-slot equipment lifecycle. |
| `G12-P1-B8` | Import Investment Environments/Strategies and the complete direct GridFight Augment, Portal, Orb, Projection, Talent, enhancement, offer and effect closure. |
| `G12-P1-B9` | Import Standard/Overclock rank boundaries, enemy affixes and only the permanent progression that changes legal entry, starting state, offered content or battle contributions. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G12-P2-B1` | Freeze reachable shared versus mode-owned Blessing, enhanced-level, buff/formula and Path-related pool membership, including a proven zero count where a category is absent. |
| `G12-P2-B2` | Import all obtainable Curios/Miracles/Hexes, mode copies, equipment-like states, charges, destruction, repair, replacement and offer-pool rules. |
| `G12-P2-B3` | Import Occurrences/events, variants, conditional choice graphs, chests, costs and mechanical outcomes without presentation prose. |
| `G12-P2-B4` | Import currencies, recruitment/refresh/upgrade shops, workbench/gamble/services, prices, inventories, candidate sets and eligibility. |
| `G12-P2-B5` | Import encounter pools, exact enemy variants, StageConfig waves, elite/boss alternatives, final bosses and difficulty/Plane bindings. |
| `G12-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking evidence gap. |

### Phase 3 — Independent Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G12-P3-B1` | Add isolated profile/Gambit/stage/difficulty/Squad-HP/action-value/economy Sora tables and typed references. |
| `G12-P3-B2` | Add roster/position/star/Bond/Empowerment/build/equipment and GridFight investment-system tables and lifecycle references. |
| `G12-P3-B3` | Add Blessing/Curio/event/service/encounter and mechanic-rule binding tables without duplicating generic semantics. |
| `G12-P3-B4` | Add provenance, coverage, approximation, reconciliation, review-fixture and pack-index tables; generate isolated schema locks/templates/readers. |
| `G12-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete isolated workbooks with validation, filters, panes and semantic QA. |
| `G12-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G12-P4-B1` | Audit every manifest row, reference, enabled-selector/ownership classification, bilingual field, provenance and quality label; reject cross-mode and excluded-module leaks. |
| `G12-P4-B2` | Execute semantic fixtures for every distinct flow, Squad-HP/action-value, roster/shop/star, position/Empowerment, Bond, mapping/equipment, investment-system, content, service and pool policy; verify every approximation replacement condition. |
| `G12-P4-B3` | Reconcile overlapping source rows with committed Goal 08/09/10/11 facts, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G12-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution, commit and publication rules

- Select the earliest unblocked batch and keep only one Goal 12 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(currency-wars): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after the commit succeeds. Do not begin the next batch while the
  current batch exists only locally.
- Verify the remote branch resolves to the same full commit ID. Record the
  remote, branch, pushed commit, exact push/verification commands and result in
  the status ledger. A batch is not `Complete` until it is reachable from the
  recorded remote branch.
- Use the pinned source first and public pages only for boundary checks,
  meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0
  remains the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a
  designer-edited `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, research decisions, reconciliation
  receipts and replacement conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator to
  make coverage pass.

## Acceptance

- both pinned source caches and the focused
  table/config/TextMap/Stage/ability inventory regenerate byte-identically;
- concrete category manifests provide 100% exact-once accounting for the
  released Version 4.4 `GuideType = GridFight` selector and its closure;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, pool membership, unlock and relationship is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/evidence-only/excluded classifications are explicit
  and fail closed;
- every shared source row agrees with committed Goal 08/09/10/11 ownership
  facts or has a recorded reconciliation decision;
- flow, Squad HP/action value, economy, roster/shop/star, positioning,
  Character Empowerment, Bonds, mapping/equipment, investment systems and content-pool
  families have semantic fixtures;
- all encounters resolve concrete released enemy identities and waves, or
  carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Currency Wars row enters another mode or the production runtime bundle;
- Goal 03 release evidence and the current production configuration remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 12 denominator with no
  unresolved blocking research case;
- every batch commit, including `G12-P4-B4`, is reachable from the recorded
  remote branch; and
- the clean-checkout release gate passes and `G12-P4-B4` is committed and
  pushed.

Progress is recorded in
[the Goal 12 status ledger](12-currency-wars-reference-data-status.md).
