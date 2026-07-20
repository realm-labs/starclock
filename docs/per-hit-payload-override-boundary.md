# Per-Hit Payload Override Boundary

Normalized `HitPlanHit` damage and Toughness ratios remain structural metadata:
they describe the action's hit distribution and must each sum to one. They are
not sufficient for every executable formula. Bounce actions can repeat one full
coefficient and Toughness amount on every hit, while Blast actions can select
different parameter keys for primary and adjacent targets.

Each hit therefore accepts three optional executable overrides:

- `damage_parameter_key_override` selects a hit-specific effective-level key;
- `damage_operation_ratio_decimal` replaces only the coefficient multiplier;
- `toughness_amount_decimal` supplies the exact raw Toughness for that hit.

The ability binding still owns the scaling stat, damage class and element. A
hit override without a complete damage binding, or any payload without an
element, fails catalog loading. Missing override values fall back to M09's
binding-wide parameter and normalized shares.

Production proofs cover both exceptional shapes. Every one of Asta's five
Meteor Storm bounces retains the full level-selected coefficient (`0.25` at
level 1, `0.625` at level 15) and 30 Toughness. Kafka's Caressing Moonlight
retains separate primary/adjacent keys: `0.8`/`0.3` and 60/30 Toughness at level
1, then `2.0`/`0.75` and the same Toughness at level 15.
