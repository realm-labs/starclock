# Goal 09 — Swarm Disaster Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Simulated Universe: Swarm Disaster
reference-data pack and an isolated Excel/Sora authoring surface before any
Swarm Disaster runtime implementation.

The goal covers the mode-specific three-plane map, domains and beacons,
countdown and Planar Disarray, Path-specific Audience Dice and their faces,
reroll and cheat behavior, the Communing Device, mechanically relevant
Communing Trail and Pathstrider progression, boss-choice consequences,
Resonance Interplays, the Propagation additions, mode content pools, encounters
and every battle-visible or cross-battle rule contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated
readers, coverage and semantic review fixtures. It does not implement or expose
a playable Swarm Disaster profile.

## Start condition

Goal 09 may run while Goal 07 is completing Standard Simulated Universe
mechanics and Goal 08 is collecting Gold and Gears reference data. It is
unblocked when:

- Goal 03 remains `Complete`;
- the Version 4.4 structured source revision and existing source inventory can
  be reproduced;
- the executor uses a separate branch and git worktree from every active Goal
  07 or Goal 08 checkout; and
- Swarm Disaster artifacts use isolated paths and generated outputs.

Goal 07 or Goal 08 completion is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 09 must not modify their plans, ledgers, manifests, normalized
rows, workbooks, generated output, runtime lowering or shared combat/activity
behavior.

If research discovers a missing shared runtime primitive, record it as a future
Swarm Disaster runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and starting oracle

- game/content snapshot: Version 4.4;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- existing audit baseline:
  `content-manifests/standard-universe-v1/source-inventory.json`;
- Gold and Gears ownership cross-check: the committed Goal 08 manifest at the
  revision inspected by `G09-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 09 rather than
  silently inheriting the Goal 03 research date.

The existing Standard inventory already hashes these thirty-two focused
`RogueDLC` tables:

```text
RogueDLCAdventureRoom.json
RogueDLCAeon.json
RogueDLCAeonCabinet.json
RogueDLCAeonCross.json
RogueDLCAeonDice.json
RogueDLCAeonDiceSurface.json
RogueDLCAeonDimension.json
RogueDLCAeonTalent.json
RogueDLCArea.json
RogueDLCBlockIntro.json
RogueDLCBlockType.json
RogueDLCBossDecay.json
RogueDLCChessBoard.json
RogueDLCChessBoardAnimation.json
RogueDLCChessBoardEvent.json
RogueDLCConstValueClient.json
RogueDLCConstValueCommon.json
RogueDLCDiceSurfaceRarity.json
RogueDLCDifficulty.json
RogueDLCEndGameReward.json
RogueDLCEntrance.json
RogueDLCFinishWay.json
RogueDLCJoyHelp.json
RogueDLCLayer.json
RogueDLCMainStory.json
RogueDLCMainStoryBranch.json
RogueDLCMainStoryReward.json
RogueDLCMarkType.json
RogueDLCRoom.json
RogueDLCSubStory.json
RogueDLCSubStoryGroup.json
RogueDLCUnlock.json
```

These tables are a shared DLC framework used by both `ChessRogue` and
`ChessRogueNous`; their prefix is not proof of Swarm Disaster ownership.
`ChessRogue` selectors, transitive references or inherited shared stable IDs
are starting reachability evidence, not a substitute for the frozen manifest.

Goal 09 must also resolve the non-`MapRepo160` Swarm topology configurations
under `Config/Gameplays/RogueDLC`, `RogueBonus` IDs `101`–`106`, reachable
shared Rogue definitions, battle/level ability programs, TextMap records,
enemy stages and mode-specific copies in base tables. File or table presence,
an ID range or a matching display name is never sufficient pool-membership
evidence.

## Included content

Completeness is defined by frozen manifests for:

1. mode entry, unlocks, initial resources, Path choice and Swarm Disaster
   Trailblaze Bonuses `101`–`106`;
2. all five difficulties, their enemy/difficulty inputs, unlock relationships
   and Planar Disarray boundaries;
3. three planes, generated columns, nodes, edges, legal movement, entry and
   terminal rules;
4. all reachable room definitions, domain kinds, beacons, node replacements,
   copies, blanking and boss-choice consequences;
5. countdown initialization, movement cost, increments/decrements, scope,
   plane carry/reset and the exact transition into Planar Disarray;
6. every Planar Disarray and boss-decay level, stacking rule, exact modifier,
   cap, timing and battle contribution;
7. every selectable Path and Audience Die definition, initial effect, passive
   effect, unlock and Path-specific movement/graph rule;
8. every Audience Die face, rarity, target selector, effect, duration, legal
   target policy, roll, reroll, cheat and no-legal-target behavior;
9. Communing Device choices, Aeon cabinet/dimension definitions, point changes,
   eligibility, ordering and carry rules;
10. every Communing Trail node that changes run decisions, content
    availability, services, dice behavior or battle state, including its
    prerequisites, thresholds and exact effect;
11. every mechanically relevant Pathstrider objective, finish condition,
    progress rule and unlock consequence;
12. chapter/story prerequisites only where they unlock a Path, node, choice or
    other simulation-visible mechanic; story prose remains excluded;
13. Swarm Disaster Path and Resonance additions, all Resonance Interplays,
    Propagation unlocks and final-boss behavior;
14. Swarm-owned and reachable shared Blessings, enhanced levels,
    Curios/states, Occurrences/choices, currencies/services and candidate pools;
15. Adventure outcome tiers and rewards as abstract external outcomes, without
    reproducing movement, aiming, physics or timing input;
16. mode encounters, exact enemy variants, waves, elite/boss choices, final
    boss pools and difficulty bindings;
17. every battle-visible rule contribution, cross-battle state slot, RNG
    candidate set and lifecycle boundary;
18. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

Shared Standard content is referenced by stable Starclock identity only after
the frozen Swarm Disaster manifest proves reachability. A source copy with a
mode-specific effect, state, pool rule or display binding remains a distinct
Swarm-owned record.

## Excluded content

- Swarm Disaster runtime lowering, controllers, CLI, Agent, MCP or full playable
  runs;
- changes to `starclock-activity`, `starclock-combat` or existing Standard
  Universe runtime behavior;
- Gold and Gears, Unknowable Domain, Divergent Universe, Currency Wars and
  historical temporary modes except as explicit exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, index rewards, first-clear/account payouts and
  collection-completion rewards;
- reproduction of Adventure/action-minigame movement, aiming, physics or timing
  input;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Story and reward tables may be retained as provenance locators when necessary
to prove a mechanical unlock, prerequisite or choice. Their prose and
account-reward payloads do not become normalized runtime content.

## Architecture and artifact isolation

Goal 09 starts in the `Experimental` content lane and may finish with a complete
`Candidate` reference bundle. It does not promote a `Released` production mode.

Use isolated paths:

```text
content-manifests/swarm-disaster-v1/
content-reference/swarm-disaster-v1/
config/swarm-disaster/
config/swarm-disaster-generated/
tools/swarm-disaster-reference/
evidence/swarm-disaster-reference-v1/
```

The authoring workbooks should be mode-owned, for example:

```text
SwarmDisaster.xlsx
SwarmDisasterBindings.xlsx
SwarmDisasterReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G09-P0-B4`. They must not share mutable sheets or generated directories with
the Standard Universe or Gold and Gears workbooks,
`config/universe-generated/`, `config/gold-and-gears-generated/` or
`config/generated/`.

Swarm Disaster authoring may reference generic Activity concepts and stable
Standard/shared content IDs. It may not redefine Activity graph execution,
scopes, command atomicity, BattleSpec/Result, combat formulas, RNG, hashing or
replay. Reference rows may declare a future activity or battle rule
contribution without implementing its evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released ability programs;
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

Approximation is field-level. Hidden weights, target ordering, timing, caps,
rounding, fallback behavior or map-generation probabilities must never be
silently inferred. A deterministic project policy requires the known facts,
selected behavior, rejected alternatives, rationale, affected fixtures and a
concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G09-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, entry, difficulty and Trailblaze Bonus;
- plane, map generation, node, edge, room, domain and beacon;
- countdown, Planar Disarray and boss-decay definitions/levels;
- Path, Audience Die, die face, rarity, target rule and roll control;
- Communing Device choice, Aeon cabinet/dimension and point adjustment;
- Communing Trail node, prerequisite, threshold, cost and effect;
- Pathstrider objective, finish condition, progress and unlock;
- mechanical chapter/unlock locator;
- Path boosts, Resonance additions, Resonance Interplays and Propagation
  additions;
- Blessings/levels, Curios/states, Occurrences/variants/choices and services;
- Adventure outcomes, encounter pools, groups, waves, enemy slots and bosses;
- mechanic rules, sources, coverage, review fixtures and pack index.

Definitions remain separate from levels, mutable states, variants, objectives
and unlock conditions. Exact decimals use canonical strings. Arrays that are
sets are sorted by stable ID; map paths, effect programs and other semantic
sequences preserve declared order.

## Parallel execution rules

When Goals 07, 08 and 09 execute concurrently:

- use separate git worktrees and branches;
- Goal 09 owns only the isolated paths declared above plus its three goal
  documents and index row;
- do not edit Goal 07 or Goal 08 plans, ledgers, policies, evidence or content
  partitions;
- do not edit existing Standard Universe or Gold and Gears manifests,
  normalized rows, workbooks or generated output;
- do not regenerate `config/universe-generated/`,
  `config/gold-and-gears-generated/` or `config/generated/`;
- do not reclassify a shared `RogueDLC` record in place; record Swarm ownership
  and reachability in the Goal 09 manifest;
- compare any overlapping Goal 08 and Goal 09 source row by source path,
  stable row locator and evidence digest at the named reconciliation
  checkpoint;
- stop and reconcile incompatible ownership or semantic classifications rather
  than overwriting another goal;
- preserve Goal 03 and current production bundle digests.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed fully
in parallel. Runtime lowering and changes to shared schemas or operations
belong to a later goal after the relevant reference packs are frozen.

## Delivery phases

### Phase 0 — Freeze scope, sources and contracts

| Batch | Deliverable |
|---|---|
| `G09-P0-B1` | Verify the Goal 03 snapshot, freeze Goal 09 scope/exclusions, inspect the active Goal 08 ownership boundary and prove parallel path isolation. |
| `G09-P0-B2` | Regenerate a Swarm-focused source inventory from the pinned cache, including `RogueDLC`, `ChessRogue` topology, reachable shared Rogue, TextMap, StageConfig and mechanic-evidence files. |
| `G09-P0-B3` | Freeze exact per-category manifest IDs/counts, ownership, shared reachability and the Swarm versus Gold/other-mode exclusion boundary. |
| `G09-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families, reconciliation receipts and semantic review-fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G09-P1-B1` | Import profile entry, five difficulties, planes, columns, nodes, edges and terminal topology. |
| `G09-P1-B2` | Import rooms, domains, beacons, node replacements, copies and boss-choice topology/consequences. |
| `G09-P1-B3` | Import countdown initialization and movement rules, Planar Disarray levels, boss decay, caps, timing and battle contributions. |
| `G09-P1-B4` | Import selectable Paths, Audience Dice definitions, initial/passive effects, unlocks and Path-specific graph rules. |
| `G09-P1-B5` | Import every Audience Die face, rarity, target/effect parameters, roll/reroll/cheat behavior and no-legal-target policies. |
| `G09-P1-B6` | Import Communing Device choices, Aeon cabinet/dimension values, point changes, eligibility and carry/order rules. |
| `G09-P1-B7` | Import the mechanically relevant Communing Trail prerequisite graph, thresholds, run/service/dice effects and battle contributions. |
| `G09-P1-B8` | Import Pathstrider objectives, exact finish/progress conditions, unlocks and mechanical chapter locators. |
| `G09-P1-B9` | Import bonuses `101`–`106`, Path/Resonance additions, Propagation unlocks, all Resonance Interplays and final-boss contributions. |

### Phase 2 — Content pools, outcomes and encounters

| Batch | Deliverable |
|---|---|
| `G09-P2-B1` | Freeze reachable shared versus Swarm-owned Blessing, enhanced-level, Path and Resonance pool membership. |
| `G09-P2-B2` | Import all obtainable Curios, mode-specific copies, lifecycle states, charges, repair, replacement and pool rules. |
| `G09-P2-B3` | Import Occurrences, Swarm variants, conditional choice graphs, costs and mechanical outcomes without presentation prose. |
| `G09-P2-B4` | Import currencies, shops/services, beacons and abstract Adventure outcome tiers, prices, rewards and eligibility. |
| `G09-P2-B5` | Import encounter pools, exact enemy variants, StageConfig waves, elite/boss alternatives, final bosses and difficulty bindings. |
| `G09-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking evidence gap. |

### Phase 3 — Isolated Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G09-P3-B1` | Add isolated profile/topology/domain/countdown/Disarray Sora tables and typed references. |
| `G09-P3-B2` | Add Path/Audience Die/face/roll-control, Communing Device, Communing Trail and Pathstrider tables. |
| `G09-P3-B3` | Add Resonance Interplay, content pool, Curio, Occurrence, service, Adventure, encounter and mechanic-rule binding tables. |
| `G09-P3-B4` | Add provenance, coverage, approximation, reconciliation, review-fixture and pack-index tables; generate isolated schema locks/templates/readers. |
| `G09-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for complete isolated workbooks with validation, filters, panes and semantic QA. |
| `G09-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet. |

### Phase 4 — Review and freeze

| Batch | Deliverable |
|---|---|
| `G09-P4-B1` | Audit every manifest row, reference, ownership, bilingual field, provenance and quality label; reject Gold/Unknowable/Divergent leaks. |
| `G09-P4-B2` | Execute semantic fixtures for every distinct mode mechanic, lifecycle and pool policy; verify every approximation replacement condition. |
| `G09-P4-B3` | Reconcile shared `RogueDLC` rows with the committed Goal 08 manifest, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G09-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution and commit rules

- Select the earliest unblocked batch and keep only one Goal 09 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(swarm-disaster): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after the commit succeeds. Do not begin the next batch while the
  current batch exists only locally.
- Record the remote, branch, pushed commit ID and push command/result in the
  status ledger. A batch is not `Complete` until its commit is reachable from
  the recorded remote branch.
- Use the pinned source first and public pages only for boundary checks,
  meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0 remains
  the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a
  designer-edited `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, research decisions, reconciliation
  receipts and replacement conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator to
  make coverage pass.

## Acceptance

- the pinned Swarm Disaster source inventory regenerates byte-identically;
- concrete category manifests provide 100% exact-once accounting;
- every enabled record has bilingual names/summaries and resolvable provenance;
- every numeric vector, pool membership, unlock and relationship is exact or
  explicitly approximate/policy-bound;
- all Swarm-owned/shared/excluded classifications are explicit and fail closed;
- every shared `RogueDLC` row agrees with the committed Goal 08 ownership
  boundary or has a recorded reconciliation decision;
- all topology, countdown, Planar Disarray, Audience Dice, roll controls,
  Communing Device, Communing Trail, Pathstrider and Resonance Interplay
  families have semantic fixtures;
- all encounters resolve concrete released enemy identities and waves, or carry
  an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Swarm Disaster row enters the Standard, Gold and Gears or production
  runtime bundle;
- Goal 03 release evidence and the current production configuration remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 09 denominator with no
  unresolved blocking research case;
- every batch commit, including `G09-P4-B4`, is reachable from the recorded
  remote branch;
- the clean-checkout release gate passes and `G09-P4-B4` is committed.

Progress is recorded in
[the Goal 09 status ledger](09-swarm-disaster-reference-data-status.md).
