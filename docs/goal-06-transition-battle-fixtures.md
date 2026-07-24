# Goal 06 Current-State Transition Battle Fixtures

This document is normative for `G06-P2-B5`.

## Proven transition chain

The transition fixture builds typed Standard Universe snapshots, resolves each
through the shared `StandardUniverseBattleAssembler`, and starts real
combat-core battles from the selected immutable materialization.

It freezes these representative transitions:

| Transition | Battle-visible result |
|---|---|
| `universe.blessing.612344` absent → L1 | new compiled Blessing rule and new combat input |
| `universe.blessing.612344` L1 → L2 | new combat input and different real damage events |
| `universe.curio.8` absent → active | new combat input and its real battle-start damage |
| `universe.curio.8` active → suppressed | battle-start damage is absent and combat input changes |
| suppressed → removed | combat input remains equal when neither state contributes; assembly provenance remains distinct |
| Hunt below threshold → three Path Blessings | Hunt Resonance and its keyed team resource enter the battle |
| Ability Tree node 2 absent → selected with prerequisites | battle-visible modifiers enter every player combatant |
| first battle full state → settled damaged/charged carry | the next dynamically assembled `BattleSpec` embeds exact HP and Energy initial state |

The Blessing, Curio, Resonance and Ability Tree fixtures use the released
typed runtime compilers. They do not inject rule bundles directly into
combat-core.

## Curio lifecycle equivalence

Version 4.4 Standard Universe data does not define a separate disabled state
for `universe.curio.8`. Suppression and removal can therefore have distinct
Activity history and assembly provenance while producing the same
battle-visible input when neither contributes an effect.

This is intentional:

- `AssemblyDigest` answers which current mode state produced the battle;
- `CombatInputDigest` answers what combat-core will execute.

Manufacturing a different combat digest for two equal executable inputs would
break combat-owned identity. If later content gives disabled ownership a
battle-visible rule, resource or modifier, its typed contribution will
naturally change both identities.

## Cross-battle carry

The carry fixture uses the production Activity flow:

1. prepare and dynamically start the first encounter;
2. settle it with exact participant HP and Energy values;
3. select the reward and advance to the next encounter;
4. project the next current-Activity snapshot;
5. dynamically assemble and start the second encounter;
6. validate the embedded `ParticipantInitialState` values;
7. construct a real `Battle` with the paired catalog.

No test-only carry constructor or direct Activity-state mutation is used.
