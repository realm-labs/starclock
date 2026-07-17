# Enemy, encounter and Standard configuration schema

Goal 01 batch `G01-P1-B9` freezes the Sora 0.3.0 transport contracts for the
enemy catalog, deterministic AI, ordinary encounters and Standard scenarios.
It also defines only the generic Activity rows needed to hand one battle a
locked participant set and receive a declared result projection. These rows are
authoring input; `starclock-data`, `starclock-activity` and `starclock-mode`
later convert them into domain definitions. They are not combat state.

## Modules and ownership

- `config/schema/enemy.toml` owns enemy templates, mechanically distinct
  variants, level/difficulty stats, weaknesses and resistances, Toughness
  layers, enemy abilities, phases and explicit linked-entity lifecycle policy.
- `config/schema/ai.toml` owns finite state graphs, ordered ability candidates,
  mandatory fallbacks and timed state transitions.
- `config/schema/encounter.toml` owns ordinary encounter policy, ordered rule
  bundles, waves, carry policy and formation slots.
- `config/schema/activity.toml` owns the minimum generic Activity graph,
  Activity-scoped typed slots, participant/loadout-lock policy, BattleBinding,
  opaque resolved-build/BattleSpec digests and declared result projections.
- `config/schema/standard.toml` owns Standard profile and reproducible scenario
  bindings.

Every definition relation is a typed Sora reference. Stable child ordering and
parent uniqueness are explicit indexes. Decimal facts are canonical strings
ending in `_decimal`; no authoritative float or implementation type enters the
schema.

## Enemy definitions and lifecycle

An `EnemyTemplate` holds shared rank, aggro and default AI identity. Each
`EnemyVariant` has a unique mechanically-distinct key and selects an exact AI
graph. Stats are keyed by variant, level and difficulty, while weakness,
elemental resistance and debuff-resistance rows remain independently auditable.

`EnemyToughnessLayer` is ordered and distinguishes ordinary, Exo-Toughness,
sequential and shared layers. It records maximum, recovery ratio and initial
activation but does not hard-code resolver behavior in the loader.

Enemy abilities reuse the generic Ability/phase/hit-plan family. Their wrapper
adds telegraph, cooldown, charge and fallback metadata; ordered variant
bindings decide which actions a concrete form owns. `EnemyPhase` explicitly
selects entry/exit conditions, replacement priority, transition model, AI and
HP/action-gauge/effect/Toughness/summon carry policies. `EnemyLink` declares
summon/part/shared-HP/countdown/timeline relationships, capacity overflow,
owner-defeat handling, wave persistence, victory contribution and formation.
The resolver therefore has no reason to branch on enemy IDs.

## Deterministic AI

An `AiGraph` names one initial state and a finite automatic-transition budget.
Every `AiState` has a mandatory fallback ability. Ordered `AiCandidate` rows
declare condition, target selector, priority, first-legal or weighted selection,
RNG purpose and no-target fallback. Ordered transitions declare target state,
condition, priority and one of automatic-before-decision, after-action,
after-phase or explicit timing.

Sora enforces reference and uniqueness constraints. Catalog construction in
`G01-P1-B11` additionally proves graph reachability, candidate legality,
fallback ownership, deterministic priority ties and absence of unbounded
automatic-transition cycles. Runtime AI consumes the same legal-decision and
purpose-labeled RNG surfaces as any other controller.

## Encounters and waves

An `Encounter` selects level, difficulty, environment, wave-transition point,
initial/maximum Skill Points and explicit victory/loss policy. Ordered rule
bindings provide generic resolver input. Ordered `EncounterWave` rows declare
entry/exit programs and HP, Energy, Skill Point, effect and Action Gauge carry
policy. `WaveSlot` fixes spawn order, formation, exact enemy variant, optional
level/phase override and whether that unit contributes to victory.

The schema carries no global clock, score, season, reward or account state.
AfterAction is a selectable ordinary-wave transition boundary; its later
runtime implementation must still settle the authoritative action and event
queues before advancing.

## Minimum Activity and Battle handoff

Goal 01 needs a generic orchestration seam without implementing future modes.
The Activity graph therefore has only `Battle` and `Terminal` node kinds, with
won/lost/faulted edges and bounded visits/traversals. Sections and nodes have
stable IDs and explicit entry points. Activity slots reuse the typed Rule IR
value/scope/reset contracts and are restricted to Activity, Section, Node or
Attempt ownership.

`ParticipantPolicy` declares team/party bounds, uniqueness and the exact
Activity/Section/Node/Attempt loadout-lock scope. `BattleParticipantSlot` binds
ordered formation positions to compiled or fixed character builds. It carries
only a catalog revision plus opaque build and resolved-spec SHA-256 values;
Trace, Eidolon, Light Cone and equipment concepts do not enter battle state.

`BattleBinding` links one battle node to an encounter, participant policy,
immutable rule bundle and result projection. Its purpose-labeled seed stream,
participant-lock digest and BattleSpec policy revision protect deterministic
handoff. `BattleResultProjectionFieldNode` is closed: outcome, final state hash,
event digest, terminal fault or an explicitly typed metric. The Standard
fixture declares exactly the first four fields, so the Activity layer cannot
silently inspect undeclared battle internals.

## Standard-only profile

`StandardProfile` is one player team of at most four participants and an
ordinary wave-transition default. Its global-clock, score and seasonal-rule
flags must all be false. `StandardScenario` binds a profile, Activity,
BattleBinding, fixed master seed and expected terminal outcome.

No challenge, universe, shop, reward, account, season, score or clock table is
present. Documents 14 and the challenge sections of document 18 remain future
extension boundaries and are not implemented by this schema.

## Golden and production boundaries

`config/schema-fixtures/standard-encounter` composes the B7 character/build and
B8 typed Rule IR fixtures before adding a disabled two-variant enemy family,
linked summon, Exo-Toughness layer, enemy ability/hit plan, deterministic AI,
one-wave encounter and four-build Standard Activity. Its verifier runs Sora
check/build/schema lock, all Excel templates, Rust codegen, binary and
diagnostic export, direct/configured output comparison and a second build.

Negative gates reject missing enemy/projection references and duplicate AI
order through Sora. The semantic verifier rejects Activity cycles/unreachable
terminals, missing AI fallback, battle-scoped Activity slots, duplicate or
unlocked participants, undeclared Standard result fields and challenge flags.
All 40 composed identities remain disabled `ProjectFixture` evidence. The
fixture is not a JSON runtime path and grants no coverage.

Production tables remain `.xlsx`-authored and exported by pinned Sora in
`G01-P1-B10`. `G01-P1-B11` owns complete cross-row domain validation and
conversion into immutable catalogs. Runtime AI, Activity execution and
Standard encounter execution remain later Goal 01 batches.
