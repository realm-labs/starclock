# Rule-emission lifecycle runtime boundary

Status: implemented by `G01-P7-M06`. This is generic combat infrastructure;
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

Missing linked-unit or countdown definitions fail catalog construction. A
runtime failure rolls back the whole command, including allocated IDs, emitted
events, RNG draws, effects, links, queued work and rule-slot mutations.

Hit lifecycle events now retain the action actor as their explicit applier.
This makes effect attribution from a Hit Ended rule unambiguous without
inferring applier from owner, actor or target later.

## Deliberate compatibility limits

The current Rule IR schema has no queue-boundary field, so rule-emitted queued
actions enter `AfterAction`. An explicit authored boundary belongs to the
production schema/compiler follow-up rather than an implicit runtime default.

Skill Point and suppressed-cost rule queues are executable. A keyed
`TeamResource` payment remains rejected until the data compiler supplies a
canonical key-to-`SourceDefinitionId` registry. Named Character/Team resource
mutations have the same prerequisite. The remaining non-representative
emissions—including shields, break and Toughness-layer operations, weakness
removal, true damage, extra turns and informational events—remain fail-closed
until their ordinary-service mappings are completed.

Effect-bound rule/modifier instantiation and production authoring of linked-unit
and countdown definitions are likewise later import prerequisites. No content
may receive `DataReady` while one of its reachable emissions depends on these
unresolved paths.

## Regression evidence

`ability_program_execution.rs` executes one rule that mutates its slot, applies
and removes an effect, queues and resolves a counter, summons a memosprite and
creates a countdown. `rule_ir_contract.rs` proves unresolved summon/countdown
references are rejected during catalog construction.

Validation commands:

```text
cargo test -p starclock-combat --test ability_program_execution --all-features
cargo test -p starclock-combat --test rule_ir_contract --all-features
cargo test -p starclock-combat --all-targets --all-features
node tools/repository-check/verify-source-policy.mjs
```
