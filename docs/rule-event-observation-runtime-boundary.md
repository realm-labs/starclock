# Rule event observation runtime boundary

This boundary makes authored rule observations exact instead of reducing them to broad event families. `RuleTrigger` retains the complete `EventPattern` point, and dispatch requires both its indexed family and exact point to match. A trigger authored for `ActionStarted`, for example, cannot run on `ActionResolved`.

The immutable event projection exposes generic source class, cause roles, selector-relative owner/actor/applier/target membership, action kind, ability tag, element, damage class, resource kind, ancestry, and typed event values. Missing facts fail closed. Runtime filtering never infers one cause role from another.

Expressions can read Energy, Skill Points, named character resources, named team resources, event properties, and selector sums. Conditions can query life/presence, effect existence, weakness, broken state, selector cardinality, resource bounds, and event-property comparisons. These reads use immutable snapshots captured before rule proposals execute, so evaluation cannot observe its own uncommitted mutations.

The resolver projects only facts carried by the committed battle event. Unsupported or absent facts remain `None`; they are not synthesized from content identities. Production and fixture catalogs share the same lowering path.

Verification:

- `cargo test -p starclock-combat --test rule_ir_contract`
- `cargo test -p starclock-combat --test ability_program_execution`
- `cargo test -p starclock-data probe_tests`
- `node tools/config-schema/verify-rule-ir.mjs`
- `node tools/config-production/verify.mjs`
