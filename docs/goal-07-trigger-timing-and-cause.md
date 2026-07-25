# Goal 07 Trigger Timing and Cause Runtime

`G07-P1-B1` replaces the former `AfterEvent`-only runtime with an explicit
phase/point matrix. The matrix is shared by catalog validation and production
dispatch, so an accepted trigger cannot silently remain unreachable.

## Committed-event phase order

For one observed event, candidates execute by phase and then by authored
priority, side, formation, spawn sequence, source, rule, rule instance and
trigger identity. Lower priority values execute first. The supported
post-commit sequence is:

1. `Before` at a start envelope;
2. `AfterMutation`;
3. `AfterDefeatSettlement` for `UnitDefeated`;
4. `AfterEvent`;
5. `AfterAction` for `ActionResolved`; and
6. `Boundary` at declared lifecycle boundaries.

`Before` is valid only for events that are themselves emitted before their
substantive body: battle, wave, turn, action, phase and hit starts, plus
`ActionDeclared`. `BattleFaulted`, `DamageCalculated` and `TimelineChanged`
currently have no committed event fact and are rejected during catalog
construction. A future consumer must add its fact and production probe before
those points can be authored.

Each trigger's first emitted operation is parented directly to the observed
event. Nested emissions retain root command, action, phase, hit, owner, actor,
applier, source and target attribution.

## Once-scope behavior

Battle Rule IR supports event, hit, target-within-hit, ability, action, turn,
wave and battle scopes. The rule instance and trigger ID are always part of
the key.

Turn keys are cleared atomically before `TurnStarted` dispatch. This makes a
once-per-turn rule work for damage and other nested facts without adding a
second persisted turn identity. The integration probe executes a two-hit
action on two separate turns: one activation per turn. Event keys are removed
only after all phases and candidates for that event finish, bounding ledger
growth without allowing a repeated activation.

## Replacement boundary

`evaluate_replacement_program` returns only typed
`RuleReplacementProposal` values. Ordinary mutation emissions are rejected.
A `Replace` trigger is not accepted into a battle catalog until a typed
operation consumer exists; it cannot masquerade as a post-commit event.

`G07-P1-B5` owns the concrete action, Break and phase replacement consumers.
This dependency is machine-recorded rather than leaving an accepted but dead
trigger in production data.

Verify this batch with:

```text
node tools/goal07/verify-phase1-b1.mjs
cargo test -p starclock-combat --all-features --locked
node tools/repository-check/run.mjs
```
