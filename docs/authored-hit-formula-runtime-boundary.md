# Authored Hit Formula Runtime Boundary

`AbilityHitPlanBinding` is the production boundary between structural hit plans
and executable damage/Toughness operations. A binding may independently author
a damage payload and a Toughness payload. Damage requires one effective-level
parameter key, one scaling stat, one ordinary damage class and one element;
Toughness requires a base amount and the same explicit element. Partial payloads
fail catalog loading.

The data compiler resolves the parameter against the exact level-1 family or
checked higher-level ability variant. It multiplies the coefficient and base
Toughness by each hit's canonical share, then emits ordered typed operations.
Missing parameters, negative coefficients, invalid decimals and incomplete
bindings fail closed.

`ScalingDamageDefinition` retains scaling stat, coefficient, class and element.
At hit execution the resolver reads the acting unit's live stat through the
shared modifier resolver and applies the coefficient with the repository's
fixed-point rounding policy. The ordinary damage pipeline remains the sole HP
mutation path. Authored Toughness uses the existing checked layer router and
Break pipeline.

M09's production proof binds Asta's Spectrum Beam to ATK, Fire and
`parameter.01`. Level 1 resolves `0.5 × 2000` to 1000 damage; effective level 10
resolves `1.4 × 2000` to 2800. Both retain and execute exactly 30 Toughness. The
generic runtime regression separately executes a scaling hit through a real
battle transaction.

This batch deliberately gives no character readiness credit. Wider V1B
authoring must bind every selected damaging ability and prove its own behavior
before the six representative forms can be promoted.
