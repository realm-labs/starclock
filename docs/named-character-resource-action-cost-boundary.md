# Named character-resource action-cost boundary

Goal 01 batch `G01-P7-M13` extends the common action resource envelope with
form-scoped named costs. This is the generic prerequisite for released
zero-Energy Ultimates whose payable currency is a character resource rather
than Energy, including the Phase 7 C02 Castorice import.

## Immutable policy and catalog contract

`ActionResourcePolicy` owns a canonical, strictly stable-key-ordered list of
`CharacterResourceCost` values. Every key must be nonempty, every amount must
be positive and duplicate or noncanonical keys fail closed. A named cost makes
an Ultimate payable for catalog validation; a participant's required fallback
normal action must still have no Skill Point, Energy or named-resource cost.

Production lowering recognizes `CharacterResource` + `Spend` deltas at
`ActionStarted`. Those rows retain their exact key and scalar amount. A
zero-Energy Ultimate with at least one such cost no longer receives the legacy
synthetic Energy fallback. Costs are sorted and duplicate keys reject the
catalog instead of becoming order-dependent.

## Legality, payment and rollback

Every legal-command enumeration checks that the acting unit defines each
named resource and has at least the authored amount. This applies equally to
normal actions, Ultimate interrupt offers and automatically queued actions.
Missing or insufficient state therefore produces no offer and never reaches a
mutation boundary.

Accepted costs commit inside the existing action transaction after
`ActionDeclared` and before `ActionStarted`. Each key is deducted with checked
scalar arithmetic through the journaled character-resource setter and emits a
`ResourceEventData::CharacterResource` fact containing the unit, exact key,
before, after and maximum values. Any later action fault follows the common
transaction rollback path, restoring all earlier keyed deductions together
with the rest of the action state.

Focused combat tests pin invalid-cost rejection, zero-Energy Ultimate catalog
acceptance, insufficient-resource offer removal, forged-command immutability,
exact deduction and event payloads. A data-layer regression test pins Sora
lowering without synthesized Energy.
