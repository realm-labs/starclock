# Authoritative numeric implementation

Goal 01 batch `G01-P2-B1` implements numeric policy revision
`fixed-i64-6dp-v1` in `starclock-combat`. The public API contains only
Starclock-owned fixed-width domain values; `fixnum = 0.9.5` remains private to
the numeric module.

## Representation and operations

`Scalar` stores signed `i64` millionths. `Ratio` has the same representation
but communicates a dimensionless value. Named checked methods cover addition,
subtraction, negation, fixed/integer multiplication, fixed/integer division and
integral finalization. No generic arithmetic operators hide overflow or a
rounding boundary.

All precision-losing methods require one of:

- `Floor`;
- `Ceil`;
- `TowardZero`;
- `AwayFromZero`;
- `NearestTiesAway`;
- `NearestTiesEven`.

The pinned backend supplies fixed-width storage and checked exact operations.
Its built-in rounding vocabulary does not contain all six normative modes, so
one private quotient primitive promotes products/numerators to `i128`, computes
the truncating quotient/remainder and applies the selected sign/tie rule before
checked conversion to `i64`. Products of two `i64` raw values and division
numerators scaled by one million fit this intermediate. Division by zero and
result overflow are typed errors.

## Domain wrappers

`StatValue`, `Speed`, `ActionGauge`, `Probability`, `Hp`, `ShieldAmount`,
`DamageAmount`, `HealingAmount` and `RawToughness` reject illegal construction.
Speed is strictly positive; fixed/integral quantities are non-negative;
probability uses integer millionths in `[0, 1_000_000]`. Formula finalization
rounds once and then checks the output domain.

`NumericError` distinguishes overflow, division by zero, invalid conversion and
out-of-domain input. Saturation and implicit clamping are absent; later formula
code must name a documented combat-rule bound explicitly.

## Mechanical gates

The repository source policy scans authoritative `src` roots in combat, build,
data, rules, replay, AI, activity and Standard mode crates and rejects Rust
floating-point type tokens. Test/tool roots remain available for the explicitly
non-authoritative formula oracle owned by `G01-P2-B5`.

`numeric_golden.rs` binds the revision to exact raw multiplication, division,
sign, tie, overflow, conversion and formula-finalization vectors. The same test
runs on every native CI platform; it contains no platform math or float path.

`numeric_formula_oracle.rs` is the deliberately non-authoritative floating-point
research oracle added by `G01-P2-B5`. It composes the public numeric primitives
through the documented derived-stat, ordinary damage, defense, resistance,
mitigation, healing, shield, Toughness, Break, Super Break, application-chance,
Energy, Speed and Action Gauge equations. Exact state-finalization vectors,
typed overflow/domain failures and all six rounding modes are fixed assertions;
4,096 deterministic legal multiplication/division cases must remain within the
single declared `0.000001` resolution. The oracle exists only under `tests/` and
does not define production formula behavior ahead of Phase 3.
