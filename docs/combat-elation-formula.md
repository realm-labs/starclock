# Shared Elation calculator

The combat domain now exposes a pure, named Elation calculator. This is a
shared prerequisite, not a new battle operation or a completed content mechanic.
Existing `DamageClass::Elation` operations still use their authored ordinary
formula inputs; this calculator does not silently reinterpret them.

## Contract

`formula::elation::ElationDamageContext` accepts a resolved level base, original
damage ratio, decided CRIT result, additive Elation ratio, resolved Punchline
multiplier (`meter_multiplier`), additive merrymaking ratio and target factors.
The core names only a generic resolved meter; mode/content code supplies the
Punchline or Certified Banger interpretation. It accepts neither
ordinary damage boost nor weaken. ATK and Break base damage are not fallback
bases. The result retains each named factor and raw/finalized amounts.

The explicit calculator contract is:

```text
base × original × crit × (1 + elation) × resolved_punchline
     × (1 + merrymaking) × defense × resistance × vulnerability
     × mitigation × broken
```

DEF, bounded RES, additive vulnerability and multiplicative mitigation use the
same checked services as ordinary damage. CRIT eligibility and RNG remain the
caller's responsibility. Signed Elation/merrymaking additions may reduce their
effective factor to zero; negative effective factors reject rather than clamp.
Other factors must be nonnegative. Invalid domains and overflow return typed
`NumericError` values without state mutation.

The factor order and six-decimal nearest-ties-even multiplication are an
explicit project precision policy. Damage is floored to an integer once, at
the end. A zero later factor does not mask an earlier overflow. These choices
are not an original-game last-digit parity claim.

## Evidence and policy boundary

The current released structured-data source is
[`Dimbreath/turnbasedgamedata`](https://gitlab.com/Dimbreath/turnbasedgamedata/-/tree/fd978d6ef09f941fba644c731ab54abd6f7c3568),
revision `fd978d6ef09f941fba644c731ab54abd6f7c3568`, Version 4.4,
accessed 2026-10-01. The exact source facts used here are:

| Path and locator | SHA-256 | Fact / quality |
|---|---|---|
| `ExcelOutput/ElationBasicLevelDamage.json`, `Level=80` | `a8500d0632e41591e7db565309cdd47b939ea2847539fc4746b2fda1282c91df` | Exact decimal `7535.107`; released structured-data fact, used only in a calculator test, not a production level-table import. |
| `ExcelOutput/AvatarPropertyConfig.json`, `PropertyType=ElationDamageAddedRatio` and `ElationDamageAddedRatioBase` | `7fa7b65f6ee196c80640eae0a77b4dc134eb705dda227cd1b849683a3c4f982a` | Separate Elation property identity; released structured-data fact, not evidence for a particular stat snapshot or aggregation. |

The source establishes a distinct level base/property but does not establish
all hidden multiplier, snapshot and precision semantics. The above explicit
factor composition is therefore a **ProjectPolicy calculator contract**, not
promoted observed parity. A bounded released-source/public-text search did not
yield an admitted executable formula plus independently reproduced damage
traces. No leak/preview source was admitted.

The caller must supply the resolved Punchline factor. No inferred nonlinear
points formula, zero-points default, cap, Certified Banger substitution,
resource spend or shared-actor level inheritance is installed here. Alternatives
include a future verified points-to-factor calculator or an authored resolved
factor. Selecting either belongs to a separate source-bound runtime change.

Confidence for hidden factor/rounding parity is low. Replace these policy
choices when admitted released executable formulas and reproducible vectors
resolve them, updating the current tests in the same change. Boundary tests
cover separate factors, the exact level-base operand, all target stages, CRIT
eligibility, decimal ties/final flooring, invalid inputs and overflow.

## Remaining integration

Production stat authoring/lowering, battle-local Elation stat queries,
level-table compilation, a typed Elation operation and modifier-stage binding
remain unimplemented. Punchline snapshots and Aha/Certified Banger lifecycle
remain independently owned by their authored resources, effects and actions.
The Weighted Curio Sapient Pen is not implemented by adding this calculator.
No Divergent Universe obligation/program/family changes disposition; the
17-curio denominator and eight executing equipment effects are unchanged.

```text
cargo test -p starclock-combat formula::elation
cargo fmt --all -- --check
cargo clippy -p starclock-combat --all-targets -- -D warnings
cargo test -p starclock-combat
```
