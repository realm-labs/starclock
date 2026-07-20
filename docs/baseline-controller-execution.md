# Deterministic baseline controller execution

Goal 01 batch `G01-P6-B3` completes the reusable smoke-test controller boundary.
The baseline player scorer and authored enemy controller both select one exact
value from combat's immutable, replay-canonical `DecisionPoint`; neither can
construct a command independently or mutate a `Battle`.

## Baseline player scorer

`starclock_ai::baseline::BaselineController` consumes a `BattleView`, the active
decision and immutable `BaselineHints`. Hints identify an ability as Basic,
Skill, Interrupt or Mandatory and provide bounded integer components for
authored priority, survival, break opportunity, resource reserve and synergy.
Resolved target hints provide the target-value component. Ability and target
rows canonicalize by stable ID and reject duplicates.

The versioned `baseline-battle-controller-v1` policy applies separated integer
tiers in this order:

1. an authored action marked as preventing immediate loss outranks every
   non-survival offer;
2. mandatory and interrupt actions precede Skills, which precede Basics;
3. explicit priority/survival/break/target/resource/synergy components refine
   the tier; and
4. equal totals select the replay-canonical command key, including stable target
   identity.

All components are checked within plus/minus one million and accumulated in
`i64`; tier gaps exceed the maximum possible component sum. A missing actor,
target, ability hint, target hint or incompatible interrupt class is a typed
failure, never an inferred default. Concede ranks below ordinary offers. System
StartBattle remains a mandatory exact offer.

`BaselineDecision` retains the selected command and an ordered score breakdown
for every offer. These diagnostics are replay/audit material only; only the
subsequently submitted command affects authoritative combat state.

This scorer is a deterministic project smoke policy, not an observed optimal
game strategy. Content/profile authoring owns its hints; the controller contains
no character, enemy or mode ID branch.

## Authored enemy execution

`EnemyController` continues to execute validated finite `AiGraphDefinition`
rows: automatic transitions settle under budget, passing candidates use
canonical priority/ability/candidate order, tied authored groups use exactly one
purpose-labelled weighted draw, and no-target behavior follows its explicit
fallback. Selection searches only the supplied legal-command values.

B3 extends its contract test so reversing offered-command storage cannot change
the authored selection, candidate identity, draw, or draw count. Enemy AI does
not use the baseline player scorer.

## Deferred integration

Controller decisions are not yet replay records or CLI output; B4 and B5 own
those surfaces. Production Standard manifests and seeded end-to-end archetype
runs remain B6. Research/coverage state is unchanged.
