# Goal 18 — Apocalyptic Shadow Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Apocalyptic Shadow reference-data
pack and an isolated Excel/Sora authoring surface before any Apocalyptic Shadow
runtime implementation.

The goal covers the stable Apocalyptic Shadow challenge family and the active
released Version 4.4 period selected by schedule `203019` and group `3019`:
entry and unlock locators, four ordinary difficulties plus the released Tierce
record `30195`, two-node and Tierce flow, participant/loadout locks, attempts,
boss progress, remaining-Action-Value scoring, targets/stars, Steadfast
Safeguard, Finality's Axiom/Embers contributions, exact encounter selectors,
enemies, AI, abilities and battle-visible lifecycle rules.

This is a reference-data goal. It ends with frozen exact-once manifests,
normalized Candidate mechanics, provenance, reconciliation receipts, three
complete authoring workbooks, Sora 0.3.0 schemas/readers/bundle and executable
semantic review fixtures. It does not implement or expose a playable profile.

## Start condition and snapshot

Goal 18 may run concurrently with Goals 15, 17 and 19 only in its dedicated
`codex/goal18-apocalyptic-shadow-reference` branch and worktree. It is unblocked
when Goal 03 remains complete and these public released snapshots are locally
readable at their exact revisions:

- game/content snapshot: Version 4.4;
- structured baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93`;
- challenge architecture: `docs/18-standard-and-challenge-modes.md`;
- evidence/content contract: `docs/15-content-data-and-coverage.md` and
  `docs/content-reference/authoring-contract.md`.

The release selector seed is schedule `203019` (2026-07-20 04:00 through
2026-08-31 04:00 in the source timezone) to group `3019`. Group `3020` is
scheduled after the active period and is exclusion evidence only. Rows
`30191`–`30194`, selected Tierce `30195`, group MazeBuff `3031001`, floor
MazeBuff `3110006`, group option lists and ChallengeBoss targets are starting
oracles, not frozen denominators until Phase 0 completes.

## Included content

Completeness is defined by frozen manifest obligations for:

1. stable family, active-period, schedule, entry, unlock and terminal outcomes;
2. ordinary difficulties, two ordered nodes, predecessor relations and the
   exact released Tierce role;
3. participant/team slots, character/combat-form uniqueness, Light Cone and
   Relic-instance snapshot/lock scope, substitutions and retry boundaries;
4. attempt creation, accepted/rejected starts, abandonment, retry, node
   transition, completion and record replacement;
5. per-node Action Value budget, first-window semantics, wave transitions,
   expiry and result projection;
6. boss effective-health/progress accounting, partial progress on timeout,
   defeat completion and deterministic stage aggregation;
7. node score, total score, target `3001`–`3003` evaluation and star mapping;
8. Steadfast Safeguard/toughness protection, activation/deactivation,
   vulnerability/break contribution and phase/reset behavior;
9. Finality's Axiom, Embers and every active group/floor/Tierce buff option,
   selection scope, parameters, trigger ordering and teardown;
10. selected `ChallengeBoss*`, MazeBuff, BattleEvent, map-event, StageConfig,
    level/config and ability-program relationships;
11. exact encounters, waves, slots, concrete enemy variants, difficulty
    inputs, skills, statuses, AI, abilities, summons, linked actors and phases;
12. Blessing, Curio, Occurrence, service, currency, shop and random-choice
    families, with generated exact-zero selector proofs where absent;
13. bilingual identities, concise independent summaries, row-level provenance,
    field-level quality and explicit replacement conditions; and
14. source-path/row-locator/digest reconciliation receipts for every shared row
    overlapping Goals 13, 15 or 17.

Shared rows remain references after reachability is proven. Goal 18 never
modifies another goal's normalized row, workbook or generated output.

## Excluded content

- runtime lowering, evaluators, handlers, controllers, CLI, Agent, MCP, replay
  or playable challenge flow;
- changes under `crates/`, shared challenge/combat/activity semantics or another
  goal's artifact roots;
- historical periods and group `3020`, except bounded selector/exclusion proof;
- Memory of Chaos, Pure Fiction, Anomaly Arbitration, Simulated Universe and
  event-mode content except reconciliation evidence;
- rewards, badges, achievements, quick-clear/account history, calendar service
  behavior and item payloads;
- story dialogue, cutscenes, presentation, assets, audio and UI;
- leaks, beta/test-server rows, previews and unavailable content; and
- any production parity, runtime completeness or Released-mode claim.

## Isolated artifacts

Only these Goal-owned roots may be changed:

```text
docs/goals/18-apocalyptic-shadow-reference-data*.md
docs/goals/README.md
content-manifests/apocalyptic-shadow-v1/
content-reference/apocalyptic-shadow-v1/
config/apocalyptic-shadow/
config/apocalyptic-shadow-generated/
tools/apocalyptic-shadow-reference/
evidence/apocalyptic-shadow-reference-v1/
```

The three clean-generated `openpyxl` authoring surfaces are:

```text
ApocalypticShadow.xlsx
ApocalypticShadowBindings.xlsx
ApocalypticShadowReview.xlsx
```

Sora 0.3.0 is the only schema/check/codegen/export authority. JSON is committed
only as normalized reference/debug evidence and is never a runtime input.

## Evidence, ownership and approximation

Every factual row records exact repository/URL, revision/access date, game
version, source path/page, stable row locator, evidence SHA-256, evidence
quality, mechanism quality and note. Allowed evidence labels are
`ExactStructured`, `ExactPublicText`, `Observed`,
`ApproximateFromReleasedText` and `ProjectPolicy`.

Do not infer semantics from table names, numeric adjacency, display names or
parameter similarity. Any unknown timing, rounding, candidate ordering or
cross-node scope is a field-level gap with known facts, deterministic policy,
alternatives, rationale, affected fixtures and replacement condition. Candidate
release requires zero blocking gaps and zero runtime-executable rows.

## Normalized families

The Phase 0 schema freeze must at minimum cover:

- profile, active period, schedule, entry/unlock and terminal outcome;
- stage, node, Tierce, predecessor, attempt and transition;
- participant policy, team slot and loadout record;
- AV clock/window, wave boundary, boss progress and aggregation;
- target, objective, score, star and settlement;
- Steadfast Safeguard, Axiom, Ember, MazeBuff, BattleEvent and contribution;
- encounter, wave, slot, enemy, skill, status, AI, ability, summon and phase;
- audited content pools and exact-zero proofs;
- mechanic rule, source, reconciliation, gap, coverage, fixture and pack index.

Canonical documents use UTF-8, LF, stable key ordering and SHA-256. Stable IDs
are Starclock identities; upstream integers remain locators unless explicitly
declared otherwise.

## Phases and atomic batches

### Phase 0 — Foundation and frozen denominator

- `G18-P0-B1`: reproduce snapshots/prerequisites, prove isolated scope and
  record the branch base;
- `G18-P0-B2`: inventory dedicated/adjacent tables, config programs, TextMaps,
  encounters, enemies and exclusions;
- `G18-P0-B3`: freeze active selector, exact obligations, ownership,
  reachability, Tierce role and exact-zero pool proofs;
- `G18-P0-B4`: freeze normalized, provenance, workbook, reconciliation and
  semantic-fixture contracts.

### Phase 1 — Mode systems

- `G18-P1-B1`: import family/period, stages/nodes/Tierce, entry and outcomes;
- `G18-P1-B2`: import participants, loadout records, attempts and transitions;
- `G18-P1-B3`: import AV clocks, boss progress, scores, objectives and stars;
- `G18-P1-B4`: import Safeguard, Axiom, Embers, buffs and rule contributions.

### Phase 2 — Encounters and closure

- `G18-P2-B1`: import exact-zero/nonzero pool audits and selected shared
  challenge/config relationships;
- `G18-P2-B2`: import encounters, waves, enemy slots, variants and levels;
- `G18-P2-B3`: import skills/statuses/AI/abilities, summons, phases and generate
  mechanics, coverage, gaps, fixtures and pack index.

### Phase 3 — Excel/Sora authoring

- `G18-P3-B1`: generate isolated Sora schemas/templates/readers/debug export and
  all three workbooks through the owning generators;
- `G18-P3-B2`: run Sora check/build/codegen/export drift, workbook structural
  checks and rendered visual review.

### Phase 4 — Candidate freeze

- `G18-P4-B1`: audit exact-once coverage, ownership, shared-row reconciliation,
  semantic fixtures and zero blocking/runtime rows;
- `G18-P4-B2`: run deterministic regeneration, repository quick/full gates and
  clean-checkout release verification;
- `G18-P4-B3`: register immutable Candidate completion evidence and final
  status snapshot.

Only one batch is `InProgress` at a time. Every completed batch updates the
ledger, uses a Conventional Commit containing its exact batch ID, pushes to
`origin` and verifies the remote commit before the next batch starts.

## Terminal gates

Goal 18 is complete only when:

- every frozen obligation is accounted exactly once and all included rows are
  `DataReady`;
- active selector/Tierce semantics and all transitive encounter/ability closure
  are evidenced; all excluded pools have generated exact-zero proofs;
- three workbooks, Sora schemas, readers, debug export and binary bundle
  regenerate byte-identically;
- semantic fixtures have zero failures and zero blocking gaps;
- Goals 13/15/17 shared rows reconcile with zero content conflict and no shared
  file mutation;
- runtime-executable row count is zero and release state is
  `CandidateReferenceData` / runtime `Unreleased`;
- the change-aware gate, full repository gate and clean-checkout release audit
  pass; and
- completion evidence is committed, pushed and remotely verified.
