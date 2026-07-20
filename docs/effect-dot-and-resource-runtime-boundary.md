# Effect, DoT and Resource Runtime Boundary

`G01-P4-B5` adds generic authoritative effects and rule-owned state to
`starclock-combat`. The resolver contains no character identity branches. The
Kafka and Asta V1a workbooks remain disabled `ProjectFixture` evidence and
grant zero production coverage.

## Authoritative effects

An effect instance retains monotonic instance identity, definition, source
definition and operation, applier, target, category, dispel class, stacks,
duration clock, tick phase, stack policy, snapshot and teardown policies,
comparison inputs, tags, control mask and application sequence. Canonical state
encoding and read-only views include those fields.

All eight authored stack policies have deterministic instance semantics.
`StrongestWins` compares magnitude, authored priority, source identity and
instance identity in that order. Dispel and cleanse are target-local queries
over dispel class and optional stable tags; non-dispellable effects are never
selected by another category. Control suppresses only the named action family,
so it does not implicitly disable passive rules.

Finite durations advance only at their declared owner-turn, target-turn,
action, wave or battle boundary. Ordinary DoT ticks at its declared effect
phase. A detonation selects qualifying DoT instances on the operation target,
uses the original effect source/applier/element, and does not consume, refresh
or restack the instance. The synthetic battle golden uses an explicit
`OnApplication` capture. The Kafka workbook separately retains its authored
`Dynamic` snapshot declaration for later build/content compilation; the core
does not silently turn that declaration into a full-effect snapshot.

## Chance, aggro and resources

Resistible effect chance uses checked fixed-point multiplication of base
chance, Effect Hit Rate, target Effect RES and target-specific resistance, then
clamps once to probability. Guaranteed outcomes consume no RNG. Intermediate
checks use the purpose-tagged authoritative stream.

Aggro weights are non-negative integer values derived from authored base aggro
and modifiers. Weighted selection preserves candidate order, uses the existing
unbiased integer mapping and consumes no draw for an empty or all-zero pool.

The existing action envelope remains the mutation owner for team Skill Points
and personal Energy costs/gains. Rule IR now retains Energy, Skill Points and
keyed character-resource addresses with explicit update, Energy-regeneration
opt-in and rounding. HP stays in damage, healing and consumption operations.
Source-owned character counters use typed battle rule slots rather than new
fields on combatants.

## State slots and generated data

Battle construction binds catalog rules to monotonic rule instances owned by a
unit or encounter. Each slot retains value kind, battle scope, initial value,
optional bounds, visibility, persistence and ordered reset points. Updates are
checked for type, overflow and bounds before mutation; owner-scoped resets
cannot affect another rule instance. Slots, instances and sequence state are
canonical-hash inputs.

The generated Sora boundary lowers effects, chance-bearing application,
target-local removal, DoT detonation, all three Rule IR resource addresses,
state-slot mutation, `RuleDefinition`, `StateSlot` and `StateSlotReset` into
Starclock-owned domain values. Production still loads only generated readers
from `.sora` bytes; there is no JSON runtime path.

The Asta probe exports a two-target-turn SPD effect independently from its
Charging slot. The Kafka probe exports ordered Skill damage/75% detonation,
Ultimate damage/Shock/full detonation and follow-up damage/Shock programs, plus
a bounded turn-owned once guard reset at `TurnStart`. The exact source payloads
and executable bundle digests are recorded in their probe goldens.

## Focused evidence

- `node tools/config-probes/verify-asta-modifier.mjs`
- `node tools/config-probes/verify-kafka-dot.mjs`
- `cargo test -p starclock-combat --test effect_resource_pipeline --all-features`
- `cargo test -p starclock-data probe_tests --all-features`

The full workspace format, lint, test, generated-drift, source-cache and
alternate-CPU compile gates remain mandatory for the batch.
