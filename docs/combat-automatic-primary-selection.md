# Automatic normal-action primary selection

`AbilityDefinition::with_automatic_primary_selector` is an explicit immutable
catalog binding for Basic/Skill normal commands. It lets enemy/content factories
use the shared rule-selector sampler to commit a Single/Blast primary inside
the battle transaction. There is no mode, character or Path branch in combat.

## Admission and timing

The binding references a dependency-free `RuleUnitSelector`. Catalog validation
requires a Team/Encounter origin, current-state reference, Alive/Present
eligibility, side matching the ordinary action target relation, cardinality
exactly one, no repetition and `Fault` empty-pool policy. Choice is First,
RngUniform or RngWeighted. All/self targeting, historical snapshots,
primary-relative predicates and selector dependencies are rejected.

Pre-declaration weights and stat-comparison expressions admit numeric literals,
stat/base-stat queries and numeric arithmetic/conversions. Event-target queries,
event properties, slots, resources, selector aggregates and conditional programs
are not this binding's input contract. Owner/Actor/Applier query anchors identify
the executing unit; CurrentTarget identifies the candidate. They do not infer
the owner of an ability-granting effect.

Read-only legality still checks actor availability, control, costs and a
nonempty ordinary target pool. A bound ability offers one `UseAbility` with
`primary_target: None`, rather than one command per manual target. Legality
never evaluates random selectors or consumes RNG. Stale decisions and explicit
manual targets are rejected byte-identically by the normal command boundary.

After acceptance and decision closure, selection reads the current stat and
modifier projection before action declaration, costs or hits. The existing
stable candidate ordering, integer sampler, purpose-labeled battle RNG and
journal resolve the selected primary. The ordinary target-commitment builder
then expands Single/Blast, and the existing action envelope owns declaration,
costs, hits, reactions and normal-turn settlement. No extra action is queued.

Empty filtered pools, all-zero weights, negative weights and expression failures
follow the normal deterministic rollback/fault boundary before declaration or
payment. Empty/all-zero pools draw nothing; negative weights are not clamped.
Once committed, subsequent hits use the existing explicit invalidation policy;
this binding does not redefine a later uniform retarget as an Aggro-weighted draw.

## Scope and identity

Unbound abilities retain their offered manual targets. Queued/forced uses retain
their explicit rule/forced target contracts, even when the same Basic ability
is bound for normal commands. There is no automatic conversion of all enemy
abilities, no adapter-owned target RNG and no inferred base weight. A weighted
expression can multiply a content-owned base weight by the
[effective Aggro factor](combat-aggro-weight-factors.md).

This is a bounded Rust catalog-factory surface, not a newly added Sora column
or opcode. Production bindings and their exact configuration identity remain
the consuming factory's responsibility. This shared prerequisite grants no
complete Divergent Universe Curio or original-mechanic coverage credit.

## Verification

Native integration tests exercise real two-hit normal enemy attacks, changed
64-seed weighted primary goldens and fresh reconstruction of events, hashes
and RNG counts; Blast commitment across both hits; stale/manual rejection;
empty/zero/negative fault paths; dead/departed exclusion; unchanged manual
targeting; and static rejection of unavailable contexts and invalid cardinality,
side, history and action shape.
