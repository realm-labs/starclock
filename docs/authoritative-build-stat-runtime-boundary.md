# Authoritative build-stat runtime boundary

`G01-P7-M05` connects the pure build compiler to the existing modifier and Rule
IR services without introducing build vocabulary into `starclock-combat`.

## Resolved input

`ResolvedCombatantSpec` retains the four authored base-stat contributions needed
by the current character and Light Cone curves: maximum HP, ATK, DEF and SPD.
Each selected modifier also carries a `ResolvedModifierBinding` from its generic
modifier definition ID to the exact selected `RuleSource` ID. The binding is
canonical, one-to-one with the selected modifier set, and cannot name a source
outside the combatant's digest-bound source list.

`starclock-build` preserves attribution while patches are applied. Character
base modifiers use the character source; Trace, Eidolon and Light Cone passive
modifiers use the source of the exact patch that added them. The resolved
combatant digest is versioned as `starclock-resolved-combatant-v2` and includes
the ordered modifier/source pairs.

## Battle runtime

Battle construction validates every modifier definition and source before any
runtime allocation. It then creates one `ActiveModifier` per selected binding in
unit-ID then modifier-definition order. Runtime instance IDs and insertion
sequences come from the central battle sequence state. Owner and subject are the
resolved combatant for this build-long selection; the source ID and class remain
distinct. Linked combatants follow the same validation and transactional
allocation path.

Immutable base ATK, DEF and SPD now live beside maximum HP in private `UnitState`.
The battle-owned `ModifierStore`, complete base values, modifier instances and
next modifier sequence are part of semantic clone/rollback and canonical state
encoding. Read-only views expose retained bases and stable modifier attribution
without exposing generated rows or fixed-point implementation storage.

## Rule IR query bridge

Ability programs build a transaction-local `StatResolver` over authoritative
unit bases, the immutable catalog registry and the battle-owned modifier
instances. A typed `QueryStat` therefore observes selected Trace/Eidolon/Light
Cone modifiers through the existing staged BaseAdd, PercentOfBase, Flat,
FinalAdd and FinalMultiply pipeline. Missing bases, definitions, snapshots,
invalid values, numeric overflow and dependency cycles remain deterministic
evaluation faults and use the ordinary rollback/Faulted policy.

Selected rule bundles are instantiated in stable unit/rule order and now enter
the authoritative event dispatcher. The currently executable authored event
boundary (`Hit::Ended` / `AfterEvent`) evaluates matching rule instances in a
total priority/owner/source/rule/instance/trigger order, resolves one selector
snapshot, supplies the same stat reader, commits once-scope keys only after a
successful evaluation, and routes supported emissions through the same
transactional operation bridge. A drain faults after 4,096 dispatches so an
authored self-trigger cannot run without bound. Emission or budget failure rolls
back both mutations and the trigger ledger.

The focused executable fixture binds a progression source to a +200 Flat ATK
modifier, retains 200 base ATK, and proves that an authored two-hit Rule IR
program resolves 400 ATK before applying exact 25%/75% shares. Build fixtures
separately prove that character, Trace and Eidolon modifier patches retain their
actual source IDs under input reordering. A second executable fixture binds a
rule bundle and proves one action-scoped post-hit program mutates HP exactly once
across a two-hit action.

Adding the retained bases, modifier store and modifier allocation cursor changes
canonical state bytes. The Phase 4 benchmark workload revision, command counts,
replay sizes and provisional budgets remain unchanged; its ten deterministic
final-hash expectations are reblessed to the new authoritative layout and the
broad benchmark gate passes.
