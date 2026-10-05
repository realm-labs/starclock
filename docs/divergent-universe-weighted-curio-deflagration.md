# Most Raucous: Deflagration authoring boundary

## Released facts

The current Version 4.4 Tourn3 reference identity is
`divergent-universe.weighted-curio.1012`, not a name- or ID-range-inferred pool.
Pinned released revision `fd978d6ef09f941fba644c731ab54abd6f7c3568` of
[Dimbreath/turnbasedgamedata](https://gitlab.com/Dimbreath/turnbasedgamedata),
checked 2026-10-05, establishes Hex 1012 / MazeBuff 633412 / level 1. The
production Sources sheet retains exact file hashes, paths and row locators.
The current normalized pack digest remains
`59fe8211da025d6526f08f9d0a8e1a9955ae60219ba8b82fa6cab9f1f2b544b6`;
its Weighted Curio file is digest-checked against that index. The unresolved
reference RogueMazeBuff locator is not rewritten to assert original parity.

The six canonical parameters are `0.5/1/1.5/2/2/2`: other-Burn fractions
for one through four Fire characters, a twice-base Fire DoT multiplier and
two-turn lifetime. Both released text maps at hash `8602529829111077351`
say Deflagration is also Burn. The current
[Arcadian Chronicles Curio table](https://honkai-star-rail.fandom.com/wiki/Divergent_Universe%3A_Arcadian_Chronicles/Curio),
checked 2026-10-05, independently corroborates the clauses and describes base
damage as increasing with enemy level and Threshold Protocol. It does not
supply the hidden numeric formula.

The merged `Level_RogueBuff_Ability_HEX_S7.json` has exactly one
`Name=StageAbility_633412`. Its entry callback counts living Fire originals
using a servant-excluding team selector. Its enemy callback observes
`OnAfterBeingAttacked`. SuperBurn declares `ReplaceByCaster` and
`OnCreate/OnPhase1/OnCustomEvent`. Natural `OnPhase1` damage then calls
`TriggerModifierCustomEvent`, excluding its own SuperBurn callback by name;
`OnCustomEvent` produces damage but does not send that notification again.
The shared [damage-kind filter](rule-event-observation-runtime-boundary.md#damage-settlement-kind)
can represent this distinction, without assuming that every DoT-class event
is a natural tick.

Source expressions query HPRatio and contain a fixed operand `100`; opaque
postfix bytes and dynamic hashes are not a decoded formula. Ninety-five
canonical Group 1 HPRatio values are separately factual. Choosing this group
and interpreting Protocol HP scaling for Deflagration remain policy.

## Base damage policy

`VersionedProjectPolicyTargetGroupOneHpRatioProtocolHpFloor` chooses the
selected enemy's own level, Group 1's authored HPRatio, fixed operand 100,
then one plus the immutable Protocol maximum-HP increase. Each checked Scalar
multiplication floors to six places. Target-level correspondence, group,
Protocol interpretation, operation order and rounding are independent
low-confidence choices, not observed parity. They preserve known level and
Protocol dependence without claiming the hidden source expression is solved.

Alternatives include stage-level scaling, a different group, a separately
decoded difficulty factor or different arithmetic boundaries. Source 147
has `ProjectPolicy` quality and hashes the authored base note, not an upstream
blob. Replace each field independently when released typed semantics or
reproducible numerical traces establish it. Native all-level/Protocol consumers
are required before battle admission.

## Execution policy

`VersionedProjectPolicyOriginalFireAfterActionNaturalTickBurns` selects Fire
originals through the mapped canonical Basic scaling-damage element. The
initial alive/present original Fire count is captured once at BattleStarted;
its one-through-four factual fractions do not recapture on later roster events.
Linked units are excluded even when they borrow original form/bindings/formation.

An eligible original Unit's complete Attack ActionResolved applies one
Guaranteed Deflagration per distinct living/present opposing EventTarget at
AfterAction, priority zero. The selected native policy replaces the target's
previous Deflagration across casters, rather than claiming the source's
ReplaceByCaster coexistence is already represented. It captures twice authored
base damage, uses one dispellable Burn stack, SourceSnapshotTargetDynamic,
TurnStart ticking and a two-turn TargetTurnEnd lifetime.

Only the effect's own-source natural DotTick may detonate other Burns at its
captured entry-count fraction. Ordinary and periodic Break Burns share the
existing stable effect-instance selection; all Deflagration instances are
excluded. Other Burn sources/lifetimes are retained. External DotDetonation
must not cascade into other Burns. No second timeline, state machine or live
Activity mutation is permitted.

Complete-action correspondence, original-element mapping, captured-count
retention, Guaranteed application, cross-caster replacement, snapshot, dispel,
presence and source/actor borrowing remain separately replaceable low-confidence
fields. Alternatives include hit-end admission, caster-local coexistence,
enemy-owned effects, resistible application or live Fire counts. Source 148
hashes the authored execution note with `ProjectPolicy` quality. Replace each
field independently with released typed evidence or reproducible current traces.

## Current implementation and acceptance

The production workbook owns one `WeightedCurioDeflagrations` row and 95
`WeightedCurioDeflagrationLevels` rows. Sora/domain loading validates all exact
joins, decimals, provenance and independently hashed policy notes. Status is
`AuthoredOperandsPendingNative`: neither loading nor this contract executes
Deflagration. Native assembly and normal equipment admission still reject this
unsupported identity; the equipment-definition count remains 14/17.

Required command consumers include zero through four Fire originals,
multihit/multitarget cardinality, independent casters, two real ticks/expiry,
ordinary and Break Burns, external detonation nonrecursion, transformation,
linked/countdown exclusion, rejected-command invariance, fresh event payloads
and hashes, and unequip in both families. Forge admission, accepted-equipment
replay and both complete-run release gates remain pending. No reference
obligation, mechanic program, semantic family, gap or policy is terminalized.
