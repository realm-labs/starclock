# Rule event observation runtime boundary

This boundary makes authored rule observations exact instead of reducing them to broad event families. `RuleTrigger` retains the complete `EventPattern` point, and dispatch requires both its indexed family and exact point to match. A trigger authored for `ActionStarted`, for example, cannot run on `ActionResolved`.

The immutable event projection exposes generic source class, cause roles, selector-relative owner/actor/applier/target membership, action kind, ability tag, element, damage class, resource kind, ancestry, and typed event values. Missing facts fail closed. Runtime filtering never infers one cause role from another.

Expressions can read Energy, Skill Points, named character resources, named team resources, event properties, and selector sums. Conditions can query life/presence, effect existence, weakness, broken state, selector cardinality, resource bounds, and event-property comparisons. These reads use immutable snapshots captured before rule proposals execute, so evaluation cannot observe its own uncommitted mutations.

The resolver projects only facts carried by the committed battle event. Unsupported or absent facts remain `None`; they are not synthesized from content identities. Production and fixture catalogs share the same lowering path.

Native `DamageOverflow` derives `max(calculated - absorbed - hp_before, 0)`
from ordinary and Break damage events. It measures finalized post-guard damage
above pre-operation HP after actual shield absorption, not raw formula damage,
bounded HP loss or damage prevented by a nonlethal floor. Later healing, lethal
rescue and phase transitions do not rewrite that fact. Invalid absorption or an
unrepresentable Scalar leaves the fact unavailable; an attempted read produces
a typed missing-value fault, never an invented zero. Other event families do
not carry this property. No new event payload or damage mutation path is needed.

The native
[settlement command corpus](../crates/starclock-test-kit/tests/suites/core/combat/ability_program_execution/damage_overflow.rs)
verifies shields, guards, nonlethal floors, fractional finalization, Break damage,
ordered healing/damage, fresh canonical event reconstruction and rejected commands.
The [read contract](../crates/starclock-test-kit/tests/suites/core/combat/rule_ir_contract/damage_overflow.rs)
distinguishes exact zero, positive and missing facts. General Sora expression
admission and the Weighted Curio consumer remain incomplete.

Verification:

- `cargo test -p starclock-test-kit --test combat_suite combat_rule_ir_contract`
- `cargo test -p starclock-test-kit --test combat_suite combat_ability_program_execution`
- `cargo test -p starclock-data probe_tests`
- `node tools/config-schema/verify-rule-ir.mjs`
- `node tools/config-production/verify.mjs`

## Cause actor representation

Native event and ability-program observation retains the raw cause-envelope
`CauseActorKind` as an optional fact. `EventFilter.actor_kind` requires exactly
Unit or TimelineActor; an absent fact does not match either requirement. This
filter is conjunctive with existing actor ID/selector filters. Their established
timeline-owner projection is unchanged: that resolved owner is not evidence
that the captured actor was a Unit. A Unit kind alone does not prove an original
roster member, Path eligibility or the absence of a linked-owner relationship.

No event payload, state codec, owner attribution or damage mutation changes.
General Sora actor-kind filter authoring remains pending. The
[pure filter corpus](../crates/starclock-test-kit/tests/suites/core/combat/rule_ir_contract/actor_kind.rs)
covers Unit/TimelineActor/absent facts against all optional filter choices and
conjunctive actor identities. Run `cargo test -p starclock-test-kit --test
combat_suite actor_kind`. Native Walkie-Talkie bridge commands separately prove
that a transformed original still qualifies and its unitless countdown does
not; production equipment admission remains pending.

## Unit-level query

Native `ValueExpr::QueryUnitLevel(subject)` reads the resolved unit's own
`UnitLevel` (1–95) as an Integer. The live bridge captures this fact in the same
immutable `BattleQuerySnapshot` used by trigger and ability-program evaluation.
It does not consult a build catalog, account data, mode difficulty or owner level.
A newly committed summon is visible at the next query boundary; an uncommitted
summon cannot change the current program's read snapshot. Life/presence filtering
belongs to the selector, not this read leaf.

Owner follows existing rule-owner precedence. Actor, Applier and EventTarget
require their exact cause role. CurrentTarget requires a bound iteration or
candidate subject; it does not fall back to the primary target. Missing subjects
produce `MissingValue` context `0x202`; missing readers or units produce
`MissingValue` context `0x222`, never a guessed zero/default or linked-owner value.
Integer arithmetic is checked; Scalar formulas require explicit conversion.

Current-state `MaximumValue` predicates can compare candidate levels. Historical
selectors reject this leaf because their stat snapshot is not this current
battle-query snapshot; automatic-primary validation rejects it because the
pre-declaration frame supplies no battle-query reader. Modifier-only readers do
not supply levels. General Sora expression authoring remains unimplemented.

The [pure read corpus](../crates/starclock-test-kit/tests/suites/core/combat/rule_ir_contract/unit_level.rs)
covers every subject role, rule-owner precedence, missing frames/readers/units,
Integer typing, explicit conversion and overflow. The
[native command corpus](../crates/starclock-test-kit/tests/suites/core/combat/unit_level_query.rs)
executes level-based True damage on 1/81/95-level targets, level-based maximum
selection, slot mutation, committed summon observation, fresh canonical events,
rejected-command hash/RNG identity, rollback on missing iteration subjects and
static rejection of mistyped or unsupported historical expressions. Run both
with `cargo test -p starclock-test-kit --test combat_suite unit_level`.

This is shared capability evidence, not admission or exact base-DMG evidence for
any Curio. The Parallel Universe Walkie-Talkie consumer remains pending.
