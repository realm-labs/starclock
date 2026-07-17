# Damage, sustain and HP runtime boundary

`G01-P4-B3` completes the ordinary-damage and sustain kernel in
`starclock-combat`. Formula inputs are explicit immutable domain values. The
core receives no generated Sora rows, source-record strings, floating-point
values or character identities.

## Formula families

Ordinary damage retains separate named stages for mixed scaling terms and
additive base damage, original multiplier, CRIT eligibility/result, additive
DMG Boost, Weaken, DEF, RES and penetration, additive vulnerability,
multiplicative mitigation and broken/unbroken state. Actual-DEF and
level-derived DEF inputs are separate typed variants. RES is clamped only at
its documented boundary. CRIT-ineligible damage cannot acquire a CRIT factor.
The checked fixed-point intermediate is finalized to integral damage once.

Healing and shield creation use their own calculators. Each accepts multiple
scaling terms and additive base values, constructs its documented additive
multiplier block and exposes raw plus finalized amounts. HP consumption is not
damage: it records requested, effective and floor-blocked overflow while
preserving the target's current HP when it is already below the authored floor.

## Authoritative shield and HP operations

The closed hit-operation language now includes `ConsumeHp` and `Shield` beside
damage and healing. Shield creation allocates a monotonic `ShieldInstanceId` and
retains every instance in canonical ID order. The base concurrent policy uses
the largest remaining shield as visible protection while every live instance
absorbs the incoming amount simultaneously. The explicit additive policy
consumes instances in ID order. A target cannot mix active policies without a
typed fault, so content-specific behavior cannot arise from an ambiguous
container accident.

Damage mutates shield instances before applying overflow to HP. Every shield
mutation is journaled and emitted in stable ID order; damage facts separately
report calculated damage, shield absorption and effective HP loss. Shield state
and its next monotonic ID are included in canonical state encoding, semantic
clone/swap transactions and read-only battle views. HP consumption similarly
emits requested, effective and overflow values and never runs defeat settlement.

## Rule IR and Firefly probe

Rule IR adds typed proposals for damage, true damage, healing, shields, HP
consumption, Energy updates, effect application, action advance and countdown
creation. Evaluation stays read-only. Generated production reader rows lower
`Operation`, `Program` and `ProgramStep` into these domain proposals; the
resolver remains the only mutation owner.

The isolated Firefly probe at `config/probes/v1a/firefly-damage` is authored as
production-schema rows, materialized to `.xlsx`, and exported twice by pinned
Sora 0.3.0. Its normal Skill preserves ordered 40% Max HP consumption with a
one-HP floor, 60% Max Energy gain and level-10 200% ATK Fire damage. Its
Ultimate preserves countdown creation, RedMode application, full action advance
and Energy reset order. Five disabled `ProjectFixture` identities are rejected
by production loading and grant no DataReady coverage.

## Focused evidence

- `cargo test -p starclock-combat --test damage_sustain_pipeline --all-features`
- `cargo test -p starclock-combat --test damage_lifecycle --all-features`
- `cargo test -p starclock-data probe_tests --all-features`
- `node tools/config-probes/verify-firefly-damage.mjs`

The tests cover normative 648/720 damage vectors, mixed scaling, additive versus
multiplicative stages, healing/shield/HP overflow, both shield policies,
authoritative mutation/event order, mutation-free failed preparation and the
two source-bound Firefly programs.
