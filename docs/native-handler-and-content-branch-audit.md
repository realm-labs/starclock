# Native-handler and content-branch audit

Goal 01 batch `G01-P4-B10` closes the exceptional-code review for the V1a
mechanism probes. The compiled production registry revision is
`native-registry-v1`; it contains zero battle handlers. This is an explicit
review result, not an unimplemented placeholder.

## V1a review result

All eight dedicated probe scopes are expressible through the completed typed
Rule IR and generic Phase 4 mechanics:

| Probe | Shared expression boundary |
|---|---|
| Asta modifier | bounded slots, stacking groups, dynamic modifiers, effects and reset policy |
| Kafka DoT | effect application, chance, snapshots, tagged detonation and queued follow-up |
| Clara counter | after-hit triggers, distinct cause roles, bounded charges and queued counters |
| Firefly damage | checked HP/resources, formula expressions, weakness, Toughness and Super Break |
| Firefly transform | transform, ability replacement and presence operations |
| Aglaea memosprite | summon, link/presence, departure and generic owner policies |
| Trailblazer Elation | keyed team resources, forced Skill payment/ownership and Elation damage |
| Yao Guang Elation | generic shared actors, keyed resources, forced Skills and Elation damage |

The complete decisions and evidence paths live in
`policy/native-handler-audit.json`. Every reviewed probe has zero native-handler
rows, rule bindings and invocation operations. Partial observation status still
tracked by the research register is separate from this architecture decision;
it can refine authored values without creating character branches in the core.

## Registry and data boundary

`starclock-rules` owns one immutable sorted production registration slice.
Loading an enabled Sora `NativeHandler` row requires exact agreement with its
compiled registration for stable key, domain, version, argument-schema digest,
determinism note, owner, IR-insufficiency decision and removal condition.
Whitespace-only decisions are rejected. Disabled rows are metadata only and do
not require a compiled handler.

Generated operations may lower `InvokeNativeHandler` only when the referenced
row passed that audit, and a rule may declare only an admitted handler. Combat
catalog validation separately requires each invocation to match its owning
rule. A handler continues to receive only read-only `RuleEvaluationInput` plus
typed arguments and can return only ordinary `RuleEmission` values or a typed
fault.

## Scattered content-ID rejection

`node tools/repository-check/verify-native-handlers.mjs` runs in the universal
repository gate. It verifies:

- exact correspondence between the policy, generated production rows and the
  compiled static registration count;
- exact coverage of every `config/probes/v1a` scope and the absence of handler
  rows, bindings or invocation payloads in those probes;
- existence of every recorded decision-evidence path;
- absence of build/content identity types and V1a kit symbols from
  `starclock-combat/src` and `starclock-rules/src`; and
- absence of control flow comparing hard-coded or raw numeric definition IDs.

Generic branching on semantic enums, source classes, tags, action kinds,
effects, formula stages and typed operation variants remains legal. Adding a
future handler requires a preceding manifest mechanic batch, a written reason
the shared IR is insufficient, exact static/data metadata, focused deterministic
tests and an explicit removal condition.

## Focused evidence

- `node tools/repository-check/verify-native-handlers.mjs`
- `cargo test -p starclock-rules --all-targets --all-features`
- `cargo test -p starclock-data probe_tests --all-features`
- `cargo test -p starclock-combat --test rule_ir_contract --all-features`
- `cargo check -p starclock-combat -p starclock-rules --all-targets --all-features --target aarch64-pc-windows-msvc`
