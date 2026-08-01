# Goal 19 — Fate/Star Rail Night Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Fate/Star Rail Night reference-data
pack and isolated Excel/Sora authoring surface before any runtime
implementation.

The goal covers the released, permanently retained collaboration gameplay:
entry and unlock locators, the mechanically relevant story/activity graph,
Case Boards, Masters/Servants and participant policies, Treasures, Conceptual
Mystic Codes, decks and recommendations, magical-energy costs, Command Spells,
traits/affixes, progression, story fights, Infinite Trial fights, exact
encounters, enemies and every battle-visible or cross-battle contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality workbooks, generated readers,
coverage, reconciliation receipts and semantic review fixtures. It does not
implement or expose a playable profile.

## Start condition and snapshot

Goal 19 may run beside other Goals when it uses branch
`codex/goal19-fate-star-rail-night-reference`, a separate worktree and only the
isolated roots declared below. It is unblocked when Goal 03 remains Complete
and both pinned caches reproduce at:

- game/content snapshot: Version 4.4;
- released gameplay availability: 2026-07-24;
- execution access date: 2026-08-01;
- `Dimbreath/turnbasedgamedata`:
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- `Mar-7th/StarRailRes`:
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- official boundary: HoYoLAB Version 4.4 update details, accessed 2026-08-01;
- architecture/evidence baselines: `docs/15-content-data-and-coverage.md`,
  `docs/19-activity-core-and-mode-extension.md`,
  `docs/26-mode-extension-and-evolution.md`, `docs/07-configuration-pipeline.md`,
  `docs/content-reference/authoring-contract.md` and `docs/sources.md`.

The official update establishes that the Trailblaze Continuance and event are
released, that Infinite Trial unlocks after the authored continuation boundary,
and that permanent gameplay remains after limited rewards expire. Reward,
banner, voice, presentation and account-entitlement facts do not enter the
mechanical pack.

## Starting source oracle

The pinned source contains these dedicated tables:

```text
FateRinAvatar.json
FateRinCaseBoard.json
FateRinCaseBoardInfo.json
FateRinCaseBoardServant.json
FateRinCaseBoardTeamInfo.json
FateRinChallengeFight.json
FateRinChallengeFightBuff.json
FateRinConstClient.json
FateRinConstCommon.json
FateRinDayProgress.json
FateRinDeck.json
FateRinDeckRecommend.json
FateRinHouguConfig.json
FateRinHouguKeyword.json
FateRinHouguMapFight.json
FateRinHouguMapGroup.json
FateRinHouguRarity.json
FateRinHouguTag.json
FateRinLevelUp.json
FateRinMainMissions.json
FateRinOwner.json
FateRinOwnerInitHougu.json
FateRinResidentReward.json
FateRinStoryFight.json
FateRinSwitchDayTalk.json
```

`G19-P0-B2` expanded the planning oracle with the distinct 26-table
`Fate*.json` gameplay family. These rows are primary selector seeds rather than
Currency Wars copies:

```text
FateAffix.json
FateArea.json
FateAvatarDescription.json
FateBattleZone.json
FateBroadcast.json
FateBuff.json
FateBuffSlot.json
FateClazz.json
FateConstValueClient.json
FateConstValueCommon.json
FateDiffPassProgress.json
FateDifficulty.json
FateExpReward.json
FateHandbookMaster.json
FateHougu.json
FateMaster.json
FateMasterTalk.json
FateMazeBuff.json
FateMiscDisplay.json
FateMonsterPool.json
FatePhase.json
FateReiju.json
FateReijuAffix.json
FateStatusConfig.json
FateTrait.json
FateTraitBuff.json
```

This discovery corrects the narrower planning seed without rewriting the B1
receipt. Membership and exact denominators still wait for P0-B3 selector and
reference closure.

The focused inventory must follow exact selectors into:

```text
Config/Gameplays/Fate/MasterConfig/
Config/Gameplays/Fate/ReijuConfig/
Config/Gameplays/Fate/TraitConfig/
Config/ConfigCharacter/BattleEvent/Activity_FateRin_*.json
Config/ConfigAI/*_FateRin.json
Config/ConfigAbility/Monster/*_FateRin.json
StageConfig.json
MonsterConfig.json
MonsterTemplateConfig.json
MonsterSkillConfig.json
MonsterStatusConfig.json
BattleEventConfig.json
MazeBuff.json
TextMap/TextMapCHS.json
TextMap/TextMapEN.json
```

`Config/Activity/RtBattle`, Currency Wars Fate Bonds and any other table or
program using similar collaboration terminology are adjacent candidates only.
They remain excluded unless an exact FateRin selector proves reachability.
Names such as `Noble Phantasm`, `Mystic Code`, `Fate`, `Master` or `Servant`
never prove ownership by themselves.

Story, talk, mission and resident-reward rows are evidence-only unless they
prove graph order, entry/unlock, a participant, a mechanical choice or a fight
binding. Long prose and reward payloads are excluded.

## Included content

Completeness is defined by generated manifests for:

1. stable profile, released/permanent boundary, entry, unlocks and outcomes;
2. day/progress graph, Case Boards, nodes, edges, visit/order and carry/reset;
3. Masters, Servants, avatars, teams, trial/fixed participants, uniqueness and
   loadout/roster constraints;
4. Treasures and every Conceptual Mystic Code identity, rarity, tag, keyword,
   deck membership, recommendation and acquisition/selection boundary;
5. Mystic Code cost, target, effect, upgrade/repetition, ordering, duration,
   battle scope, carry and teardown;
6. magical energy, Command Spells/Reiju, reset attempts, choices, costs and
   state transitions;
7. owner/init loadouts, traits/affixes, progression, levels and unlocks;
8. story, map and Infinite Trial fight definitions, difficulty/affix selection,
   objectives, clear/fail/retry and terminal settlement;
9. exact StageConfig waves, enemy slots, variants, levels, skills, statuses,
   AI, abilities, summons, linked actors and phases;
10. event-specific Archer/Rin/Gilgamesh/trial interactions when selected by
    released gameplay, without duplicating their general character kits;
11. all battle-visible rule contributions, activity slots, decisions,
    projections and deterministic candidate sets;
12. audited Blessing, Curio, Occurrence, shop, service, currency and analogous
    pools, with generated exact-zero proofs when none are reachable;
13. bilingual project summaries, fact-level provenance, field-level confidence,
    approximation boundaries and semantic fixtures; and
14. reconciliation receipts for every overlap with Goals 01, 07–17,
    especially Currency Wars and shared enemies.

## Excluded content

- runtime lowering, handlers, controllers, CLI, Agent, MCP or playable flow;
- changes to combat, build, activity, rules or other mode runtime crates;
- story prose, dialogue, cutscenes, presentation, camera, assets, audio and UI;
- collaboration banners, gacha, login gifts, rewards, achievements and account
  history;
- general character kits except mechanically selected event-specific facts;
- Currency Wars Fate Bond records without independent FateRin reachability;
- `RtBattle`, unrelated minigames and other modes without exact selectors;
- preview, beta, leak, scheduled-but-unavailable or Version 4.5 content;
- a seeded runtime activity or production compatibility claim.

## Artifact isolation

Use only:

```text
content-manifests/fate-star-rail-night-v1/
content-reference/fate-star-rail-night-v1/
config/fate-star-rail-night/
config/fate-star-rail-night-generated/
tools/fate-star-rail-night-reference/
evidence/fate-star-rail-night-reference-v1/
```

Author four complete workbooks:

```text
FateStarRailNight.xlsx
FateStarRailNightCombat.xlsx
FateStarRailNightBindings.xlsx
FateStarRailNightReview.xlsx
```

The isolated Sora 0.3.0 project is review evidence only. No row enters a
production bundle or runtime loader.

## Evidence and data policy

Use pinned released structured rows first, publisher-authored released text
for boundary/meaning, reproducible observations where available and
independent public cross-checks last. Every fact records revision/URL, access
date, game version, path/page, locator, evidence digest, quality, mechanism
quality and note.

Hidden targeting, ordering, selection weights, repeat/upgrade behavior,
resource settlement, retry restoration, timing, caps, rounding and fallbacks
remain bounded research cases. If exact evidence stays unavailable, use a
field-level `ApproximateFromReleasedText` or `ProjectPolicy` with alternatives,
rationale, affected fixtures and a concrete replacement condition. Never
present policy as observed parity.

## Delivery phases

### Phase 0 — scope, sources and contracts

| Batch | Deliverable |
|---|---|
| `G19-P0-B1` | Reproduce caches, verify Goal 03/concurrent boundaries, freeze released scope/base/remote and prove isolation. |
| `G19-P0-B2` | Generate focused FateRin table/config/TextMap/Stage/enemy inventory and named exclusions. |
| `G19-P0-B3` | Freeze exact selector-backed manifests, denominators, ownership, shared closure and exact-zero pools. |
| `G19-P0-B4` | Freeze normalized schema, canonical encoding, four-workbook partition, evidence, reconciliation and fixture contracts. |

### Phase 1 — unique activity systems

| Batch | Deliverable |
|---|---|
| `G19-P1-B1` | Import profile, permanent boundary, entry/unlocks, day/progress graph, Case Boards and outcomes. |
| `G19-P1-B2` | Import Masters/Servants/avatars/teams, fixed/trial participants, uniqueness and loadout policies. |
| `G19-P1-B3` | Import Treasures, Mystic Code identities, rarity/tags/keywords, decks, recommendations and acquisition. |
| `G19-P1-B4` | Import Mystic Code costs, target/effect programs, repeat/upgrades, ordering, duration and teardown. |
| `G19-P1-B5` | Import magical energy, Command Spells/Reiju, reset attempts, choices and resource transitions. |
| `G19-P1-B6` | Import owners/init loadouts, traits/affixes, level/progression and cross-battle carry/reset. |
| `G19-P1-B7` | Import story/map/Infinite Trial fight flow, difficulty, affix selection, objectives, retry and settlement. |

### Phase 2 — encounters and complete reference pack

| Batch | Deliverable |
|---|---|
| `G19-P2-B1` | Audit and freeze reachable or exact-zero generic content pools, services, shops and currencies. |
| `G19-P2-B2` | Import fight/buff/BattleEvent/MazeBuff/stage-template/config relationships. |
| `G19-P2-B3` | Import exact StageConfig encounters, ordered waves, slots, variants, levels and difficulty bindings. |
| `G19-P2-B4` | Import enemy skills/statuses/AI/abilities, summons, links, phases and event-specific participant contributions. |
| `G19-P2-B5` | Generate mechanic rules, sources, coverage, gaps, reconciliation rows, fixtures and canonical pack index. |

### Phase 3 — isolated Excel and Sora

| Batch | Deliverable |
|---|---|
| `G19-P3-B1` | Add profile/graph/participant/progression Sora tables. |
| `G19-P3-B2` | Add Mystic Code/deck/resource/Command Spell/trait tables. |
| `G19-P3-B3` | Add fight/buff/encounter/wave/enemy/mechanic-binding tables. |
| `G19-P3-B4` | Add evidence/coverage/gap/reconciliation/fixture/index tables and isolated generated readers. |
| `G19-P3-B5` | Generate and verify all four complete openpyxl workbooks. |
| `G19-P3-B6` | Prove byte-identical double generation, Sora export/load and every-sheet visual QA. |

### Phase 4 — audit and Candidate freeze

| Batch | Deliverable |
|---|---|
| `G19-P4-B1` | Audit exact-once coverage, released selectors, ownership, bilingual fields, provenance and exclusions. |
| `G19-P4-B2` | Execute semantic fixtures and every approximation replacement condition. |
| `G19-P4-B3` | Reconcile overlaps and run source/pack/workbook/Sora/reader/dependency/clean-checkout gates. |
| `G19-P4-B4` | Freeze final documentation, counters, release evidence and Candidate identity. |

## Execution and acceptance

Select the earliest unblocked batch and keep only one Goal 19 batch
`InProgress`. Each batch is one responsibility-bounded commit using
`data(fate-star-rail-night): <batch-id> <imperative summary>`. Push and verify
each commit before starting the next batch and record commands, full commit,
counts, hashes, decisions and evidence in the ledger.

The goal completes only when exact manifests account for every obligation,
all required rows are `DataReady`, every empty family has selector-closure
proof, all ownership/references/provenance/bilingual fields close, every
distinct mechanic has a semantic fixture, workbooks and Sora regenerate/load
without drift, shared rows reconcile, production artifacts remain unchanged,
the full clean-checkout source-cache gate passes and `G19-P4-B4` is committed,
pushed and remotely verified. Runtime remains unreleased.
