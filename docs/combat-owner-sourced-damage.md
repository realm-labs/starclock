# Owner-sourced rule damage

`RuleOperationTemplate::DamageFromOwner` is a generic combat capability for a
rule/program owner to produce damage in response to a different unit's event.
It adds no character, mode or content-ID branch to the shared resolver.

## Evaluation and attribution

Amounts and target selectors are evaluated by the existing pure Rule IR
evaluator and immutable trigger snapshot. `QueryStat(Actor)` still reads the
original actor; `QueryStat(Owner)` reads the executing rule owner. An `Actor`
selector still targets the original actor even when the subsequent damage is
produced by the owner.

Only the emitted operation's cause roles are rebound: owner, actor and applier
all identify the program owner. Root command, ancestor action, phase, hit,
primary target and source-definition identity are retained. The operation's
parent is the preceding committed event in the existing ordered emission chain.
No extra action, attack envelope or normal timeline turn is created. Sibling
emissions keep their original cause and evaluation snapshot.

## Shared execution

The existing typed `Damage` operation owns source stats, level, modifiers,
elemental amplification, target defense/resistance, Crit, shields, guards,
defeat settlement, damage credit, checked arithmetic and events. Authored
`class` and `element` are not replaced with true damage. `can_defeat = false`
uses the existing one-HP floor; it does not resurrect an ineligible target.

`can_crit = false` performs no Crit draw. Otherwise the program's Crit policy
uses a fresh operation-local draw group, so the observed actor's cached result
cannot become the owner's result, and the owner's result cannot pollute later
actor-sourced damage. No alternative RNG or formula implementation is added.
The existing program damage-share contract is unchanged.

Wrong expression types and unresolved selectors reject during catalog
validation, using the same checks as ordinary `Damage`. State-dependent
execution failures retain the shared deterministic fault behavior. Ordinary
`Damage` continues to use the observed actor; owner attribution is explicit.

## Boundary and verification

The bounded Rust IR/factory surface exposes this operation; this change does
not add an opcode to the production Sora operation schema or alter a workbook.
Native command-level tests in
`crates/starclock-test-kit/tests/suites/core/combat/owner_damage.rs` exercise
owner-versus-actor formula inputs, original actor selectors and expressions,
cause ancestry, no extra action, unchanged ordinary damage, independent Crit
groups, nonlethal boundaries, defeat credit, rejected commands and fresh
deterministic reconstruction.

This is a prerequisite, not an implemented Divergent Universe equipment
effect. The current five of seventeen Weighted Curio effects, original
obligations, pending terminal dispositions and full-run gates remain unchanged.
