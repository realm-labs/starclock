# Rule-emission lifecycle runtime boundary

Status: implemented by `G01-P7-M06` and completed for production by
`G01-P7-M07`. This is generic combat infrastructure;
it does not promote any production character or Light Cone by itself.

## Authoritative lowering

After Rule IR evaluation, the resolver lowers the representative mutation set
through the same transactional services used by authored hit plans:

- `SetSlot`, `AddSlot`, and `ModifyStateSlot` address the exact active rule
  instance and retain the operation's authored update policy;
- `ApplyEffect` retains its explicit chance draw purpose, while `RemoveEffect`
  removes only the requested definition through the normal effect store;
- `QueueAction` resolves authored actor and target selectors once, validates the
  resulting target commitment against the queued ability, and enters the
  deterministic reaction queue with source/rule/instance/trigger ordering;
- `Summon` resolves a catalog-owned linked-unit definition and uses the ordinary
  owner-link lifecycle service;
- `CreateCountdown` resolves a catalog-owned countdown definition, allocates a
  timeline-only actor and creates the same canonical owner link used by other
  linked entities.
- `ModifyResource` resolves a character key against the selected unit form or
  a team key against the battle specification's explicit numeric binding;
  forced queue payment uses the same team binding;
- `EmitRuleEvent` creates an ordinary typed `RuleSignal` event that downstream
  Rule triggers observe through the normal event dispatcher.

Missing linked-unit or countdown definitions fail catalog construction. A
runtime failure rolls back the whole command, including allocated IDs, emitted
events, RNG draws, effects, links, queued work and rule-slot mutations.

Queue emissions retain an explicit `AfterHit`, `AfterPhase`, `AfterAction` or
`BeforeTimeline` boundary. No compiler or resolver default selects the timing.

Effect definitions instantiate their ordered modifier and rule bindings only
when a new effect instance is inserted. Refresh preserves those instances;
replacement, explicit removal, expiry, phase carry and wave carry tear them
down with the owning effect. Attachment ownership, named resources and team
semantic keys are part of canonical state encoding `sha256-v3`.

Hit lifecycle events retain the action actor as their explicit applier.
This makes effect attribution from a Hit Ended rule unambiguous without
inferring applier from owner, actor or target later.

## Deliberate compatibility limits

The production and committed probe inventory reaches `Damage`, `ConsumeHp`,
`ReduceToughness`, `SuperBreak`, `ApplyEffect`, `DetonateDot`, resource/slot
mutation, action shift/queue, presence/lifecycle, weakness addition, ability
replacement and informational signal operations. Every one has an ordinary
transaction-service mapping.

Unreached emissions—true damage, shields, explicit break, Toughness-layer
creation/removal, weakness removal, extra turns, replacement proposals and
native handlers—remain fail-closed. A later content partition must register an
`M08+` prerequisite before authoring one of those operations; M07 does not
claim unexercised semantics.

## Regression evidence

`ability_program_execution.rs` executes named character/team mutations,
team-resource payment, an explicit queue boundary and a typed signal alongside
the M06 lifecycle set. Separate cases prove dynamic effect templates and
effect-owned rule/modifier creation and teardown. `linked_lifecycle.rs` proves
owner-scaled production-shaped linked units and a separately authored
countdown ending transformation. `starclock-data::catalog_tests` asserts that
the production bundle contains the Aglaea linked unit, Firefly countdown,
Asta effect attachment and Clara named resource.

Validation commands:

```text
cargo test -p starclock-combat --test ability_program_execution --all-features
cargo test -p starclock-combat --test rule_ir_contract --all-features
cargo test -p starclock-combat --test linked_lifecycle --all-features
cargo test -p starclock-combat --all-targets --all-features
cargo test -p starclock-data --all-targets --all-features
node tools/config-production/verify.mjs
node tools/repository-check/verify-source-policy.mjs
```
