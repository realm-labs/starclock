# Content Reference Review Fixtures

These fixtures review whether a future Excel/Rule IR transcription preserves the
prepared mechanic contract. They are semantic expectations, not precomputed
runtime hashes; Goal 01 turns them into executable golden scenarios after the
resolver exists.

## Character fixtures

### Asta — unique-target stack ownership

Reference: `character.asta`.

- Skill bounce records every distinct target hit in one action.
- Charging credit uses target weakness at the qualifying hit boundary.
- Repeated hits on one target do not count as distinct targets.
- Charging decay occurs on Asta's authored owner-turn boundary.
- The team ATK aura queries current stacks; Ultimate SPD duration uses its own
  effect clock.

### Clara — attacker identity and counter scope

Reference: `character.clara`.

- An incoming attack retains both attacker and attacked ally in its cause chain.
- Svarog counters the attacker and marks the correct enemy.
- Skill consumes each target's mark only after its damage for that target.
- enhanced counters consume shared charges and may trigger from any attacked
  ally while preserving Clara/Svarog damage ownership.

### Kafka — DoT detonation

Reference: `character.kafka`.

- Detonation selects only qualifying DoTs on the primary target.
- Each detonated DoT retains its original applier, source, snapshot, element and
  remaining duration.
- Detonation does not consume or refresh duration unless an authored Trace or
  Eidolon patch says so.
- The ally-Basic follow-up is once per Kafka turn and applies its Shock in the
  documented operation order.

### Firefly — transformation and countdown

Reference: `character.firefly`.

- normal Skill HP consumption and Energy restoration are atomic and checked.
- Ultimate replaces abilities/stats, advances Firefly and creates an explicit
  countdown actor.
- enhanced Skill applies Fire Weakness before its Toughness operation.
- exit restores the ordinary ability set across wave transition, defeat and
  revival without leaving a countdown or transformation-owned modifier.

### Aglaea — memosprite and joint action

Reference: `character.aglaea`.

- Garmentmaker has independent presence and timeline identity linked to Aglaea.
- summon SPD stacks belong to the summon and use the authored cap/reset scope.
- a joint attack is one action envelope with separate contribution ownership.
- the empowered state ends through its countdown and performs all authored
  summon teardown/resource operations exactly once.

### Silver Wolf LV.999 — Elation boundary

Reference: `character.silver-wolf-lv-999`.

- Elation damage remains a distinct damage category and is not ordinary
  additional/follow-up damage.
- transformation, field and special-resource state have explicit owner and
  teardown scopes.
- triggers that inspect Elation damage do not activate for visually similar
  ordinary damage unless the ability carries the Elation tag.

## Enemy fixtures

### Authored sequence and cooldown

Choose a normal/elite template whose `source_ai` resolves and whose
`ai_sequence_source_skill_ids` has at least three entries.

- initial and recurring cooldowns match normalized ability rows;
- the sequence is stable under source-row reordering;
- target selection uses the normalized target program plus taunt/lock precedence;
- an unavailable target follows an explicit deterministic fallback.

### Charge then release

Use an enemy whose mechanic hints contain a Charging state and a named next
action.

- setup and release are different actions;
- interruption, break or phase transition follows an explicit cancel/carry
  policy;
- the release cannot be selected early by generic AI scoring.

### Summon and owner teardown

Use an Aurumaton, Swarm or equivalent source-backed summoner variant.

- summon slot choice and overflow are deterministic;
- summon ability/presence identity differs from the owner;
- owner defeat and phase transition apply the authored summon teardown policy;
- victory ignores or includes remaining summons according to encounter policy.

### Feigned death or revival

Use a template whose ability evidence includes Feigned Death.

- a killing blow enters the authored downed/feigned state instead of ordinary
  defeat;
- targetability, timeline presence and victory contribution are explicit;
- revival/phase entry occurs once and cannot duplicate rewards or defeat events.

### Linked/shared HP boss

Use `enemy.lord-of-sam-vartastha-yi-asat-pramad.bigboss` or another reviewed
linked-component boss.

- each face/component has independent targetability and action identity;
- shared HP transfer retains damage source and does not recursively retrigger;
- phase-owned components are removed deterministically;
- defeat is evaluated only after the complete linked damage batch.

### Multi-phase boss

Use `enemy.cocolia-mother-of-deception.bigboss` as an initial review candidate.

- phase entry/exit records HP, AV, effects, Toughness, summons and counters as
  explicit carry/reset decisions;
- queued actions from the old phase cannot execute after invalidation;
- phase transition is not mistaken for ordinary defeat or wave advancement.

### Source-gap enemy ability

Select at least one of the 357 records labeled
`ApproximateFromReleasedText`.

- review the pinned source-text hash and record the inferred target/operation
  facts independently;
- preserve exact numerical parameters from the released skill row;
- upgrade the field to `Observed` when a reproducible battle fixture confirms
  it, or retain the approximation reason and tolerance;
- never copy another similarly named enemy's source configuration by assumption.

## Acceptance use

Every shared primitive introduced for these fixtures must remain content-neutral.
A fixture may require a registered native handler, but it may not add a character
or enemy ID branch to the timeline, formula, lifecycle or target resolver.
