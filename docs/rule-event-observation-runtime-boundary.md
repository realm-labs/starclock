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
