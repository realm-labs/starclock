# Shared Elation formula and native operation

The combat domain exposes a pure, named Elation calculator and an explicit
native `HitOperationDefinition::ElationDamage`. These are shared prerequisites,
not a completed production content mechanic. Sora authoring and typed lowering
now expose the same explicit operation and separate stat. Existing ordinary-formula
`DamageClass::Elation` operations retain their authored inputs; the new typed
operation does not silently reinterpret them.

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

## Native stat and operation integration

Native battle-local `StatKind::Elation` queries now use the same stat pipeline
as other additive ratios. Their neutral base is explicitly zero in live Rule IR
execution, initial modifier capture and Event/Action selector snapshots. Native
authored BaseAdd/Flat/etc. modifiers may supply intrinsic and conditional
contributions. `QueryBaseStat` continues to return zero after such additions;
`QueryStat` returns their checked effective result. Ordinary damage boost is
still a separate formula stage, not the meaning of this property.

Live Rule IR reads observe the program's input boundary. Expressions evaluate
before that program's ordered emissions commit; applying a property modifier
does not retroactively reevaluate another expression in the same program.
An explicit later program/phase can observe the newly committed value.

Zero is a **ProjectPolicy neutral input**, not a factual import of every released
character's intrinsic Elation. It avoids inventing a universal nonzero base while
allowing native definitions to provide explicit contributions. No saturation,
floor or cap is selected for the signed Scalar query result. The pure calculator
independently rejects a negative effective `1 + elation` factor. Released form
bases must be authored with their own provenance rather than assumed from this
neutral value. Confidence for a universal released-base interpretation is low;
replace any content-level policy when source-bound authored bases are compiled.

Real-command tests in
`crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/elation_stat.rs`
prove zero/live/base reads, a temporary +50% property addition, eventful expiry,
initial source/value snapshots versus dynamic reads, distinct Current/Event/Action
selector observations, rejection hashes/RNG and fresh reconstruction. The test
damage expressions deliberately observe the property through ordinary Rule IR;
they are not a substitute for testing the dedicated Elation battle operation.

The dedicated operation takes immutable `catalog::action::elation::ElationDamageDefinition`
inputs: resolved level base, original coefficient, resolved meter factor,
merrymaking, explicit elemental RES/penetration/bounds, unbroken factor and
element. Negative bases/factors, invalid RES bounds and a negative effective
merrymaking factor reject at construction. A hit's explicit damage share
multiplies the original coefficient with checked nearest-ties-even arithmetic.
The catalog does not query progression, infer RES from weakness or invent a
level curve or points conversion.

At the operation input boundary the resolver reads source Elation and target
effective DEF using the shared stat pipeline; the actual source actor supplies
attacker level. Formula contexts retain ability/damage tags, element and
Source/Target direction. Target DEF is read as that unit's own derived stat
(Source direction for the stat owner), so unfiltered DEF buffs still apply;
incoming factor contributions separately require Target direction. Dedicated
CRIT stat and target-probability queries
retain that explicit element. Existing ordinary CRIT contexts remain unchanged.
The same bounded hit-policy service handles Never, PerTarget, Shared and the
strict current-HP guaranteed threshold. Mixed ordinary/dedicated operations
share the existing per-hit sample cache; Shared retains one raw draw, not one
boolean for all target thresholds. Certain/impossible outcomes do not draw.

`ElationDamageModifiers` projects already-filtered shared stages without
querying ordinary DamageBoost, Weaken or elemental build DMG Boost:

| Stage | Dedicated projection |
|---|---|
| Source Flat | Add to the resolved level base before coefficient application. |
| Source Crit | Add to the resolved CRIT factor, independently of the decided CRIT result. |
| Incoming Defense, Resistance, Vulnerability, Broken | Add to each resolved factor, with explicit modifier direction. Source Resistance also adds to the RES factor. |
| Incoming Mitigation | Multiply the mitigation factor by `1 - contribution`; require the contribution in `[0,1]`. |
| Source DamageOverride | A positive override replaces raw damage and is floored once. |
| Source DamageFinalMultiply | Otherwise multiply unfloored raw damage by the checked nonnegative final factor, then floor. |

This projection is an explicit **ProjectPolicy** use of the shared stage language,
not a verified hidden released formula. Factor additions are not DEF-ignore or
RES-penetration stat fields; explicit RES bounds apply before stage additions.
Alternatives include future source-bound DEF-ignore/RES-stat inputs or a verified
content-specific projection. Confidence for original-game parity is low; replace
the mapping when admitted released programs and independently reproduced vectors
resolve it, updating pure-stage and real-command tests together. No double alias
from Elation to ordinary DMG Boost is installed.

The raw/final result passes into the existing guard, shield, HP, metric, event
and defeat mutation boundary without recalculation through ordinary formulas.
Accepted commands, faults, IDs, scheduling, RNG and hashing use the single shared
engine. Real-command tests in
`crates/starclock-test-kit/tests/suites/core/combat/effect_resource_pipeline/elation_damage.rs`
cover exclusions versus existing ordinary Elation-class formulas, hit shares,
explicit factors, directional/element stage filters, effective DEF, final flooring,
mixed-operation CRIT draw counts, strict HP thresholds, shields/HP floor/team
guard/defeat, property addition/expiry and reproducible overflow faults. Every
successful fixture compares events, state hashes and RNG to fresh reconstruction;
rejected starts retain state hash/RNG.

## Rule IR expression bridge

`RuleOperationTemplate::ElationDamage` is the explicit program path to the same
dedicated operation. Its public `rule::model::elation::ElationDamageExpressions`
retains nine Scalar expressions: resolved base, original/meter/merrymaking
operands, RES/penetration/bounds and unbroken factor. No integer-to-ratio cast,
points conversion or default factor is inferred. Authored expressions may use
the existing checked stat/resource/parameter queries and explicit conversions.

The mutation-free evaluator resolves every operand and constructs a validated
immutable definition before any of that program's emissions commit. Rule catalog
validation requires Scalar operands; unbound/direct program evaluation also returns
typed type/domain/overflow errors. Signed RES and valid negative merrymaking are
retained, not clamped. A failed later operand cannot commit an earlier damage
proposal. Replacement-only execution rejects the mutating Elation proposal, and
the ordinary program step/emission/iteration budgets still apply.

The bridge retains `CurrentTarget` for bounded `ForEach`/`CurrentSubject` targeting,
applies the enclosing hit's explicit share once to the coefficient, and uses its
CRIT policy/cache when `can_crit` is true; false selects Never. The expression
operands are frozen at the read-only boundary, while live Elation/DEF and the CRIT
decision resolve at operation execution. Thus a preceding committed property
addition can affect live Elation without retroactively reevaluating that program's
meter expression; an explicit later phase may observe the new value.

Pure contract vectors in `rule_ir_contract/elation.rs` cover exact signed inputs,
all nine non-Scalar fields, domain/overflow failures, replacement rejection and
budget limits. Real commands in `effect_resource_pipeline/elation_rule_ir.rs`
cover frozen versus live inputs, native/Rule IR hit-share precision, shared and
per-target CRIT draws, per-subject stat reads and rollback without damage after an
invalid later operand. Both files are under the test kit's core combat suite.

## Production authoring capability

The Sora `StatKind::Elation` transport value is explicitly converted into the
domain stat; numeric transport IDs are not cast into domain ordinals. Stat
queries, modifiers and snapshot stat references use this shared conversion.
`OperationPayload::ElationDamage` requires nine `ValueExpression` references,
an element and CRIT eligibility. It lowers into the dedicated Rule IR template,
not ordinary damage. Every operand remains an explicit caller input; no level
lookup, default points conversion or elemental RES inference is inserted.

The documented [clean-target openpyxl author](../tools/config-production/README.md)
owns 13 `policy.probe.elation` expressions and one unbound self-damage operation
in the production workbooks. The expressions retain canonical decimal strings,
signed RES/merrymaking and a dynamic owner-property query. The resolved-meter
expression deliberately uses `1 + owner Elation` as a **synthetic capability
probe**, not a factual Punchline formula. Existing cells and released content
bindings remain unchanged; these rows grant no content coverage.

The real generated production bundle feeds
`operation_lower/elation_tests.rs` in `starclock-data`. Native fixture assembly
attaches the converted operation and the converted production owner selector.
Accepted commands produce raw `47.2212`, finalized `47` with the neutral
property, or raw `106.2477`, finalized `106` with a native +0.5 property binding.
The binding is a fixture, not a released intrinsic base. Both runs compare
events/hashes against fresh reconstruction and draw no RNG. Replacing each
of the nine references with the authored Integer negative probe faults without
damage or HP mutation; invalid start commands preserve the state hash.

Schema, templates, readers, workbook rows, binary/debug exports and current
fixture bundles regenerate under Sora 0.6.1. The fixture bundle generators
compare two independent exports; production verification checks deterministic
drift and read-only workbook synchronization. No previous-schema decoder is
retained.

The test kit's production-input-sensitive Universe state/event goldens share
`support/universe_state_manifest.rs`, binding both current core and Universe
bundle SHA-256 values. Activity stream derivation includes the configuration
digest, so changing the core bundle can change a fixed seed's topology, rewards
and nested-battle count. The current canonical state/event vectors and exact
run counts reflect these current inputs; terminal/carry/real-command assertions
remain in place rather than accepting old inputs or weakening the comparisons.
Mode-level Reward selection vectors and positive source-room/evolution seeds
also reflect current inputs. Seed discovery runs explicitly as ignored tests;
default evolution regressions execute a fixed public acquisition corpus while
retaining both probability branches and fresh replay verification.

Released operation bindings, level-table compilation and released damage
parity remain unimplemented.
Punchline snapshots and Aha/Certified Banger lifecycle
remain independently owned by their authored resources, effects and actions.
The Weighted Curio Sapient Pen is not implemented by adding this calculator.
No Divergent Universe obligation/program/family changes disposition; the
17-curio denominator and eight executing equipment effects are unchanged.

```text
cargo test -p starclock-combat formula::elation
cargo test -p starclock-test-kit --test combat_suite elation_stat
cargo test -p starclock-test-kit --test combat_suite elation_damage
cargo test -p starclock-test-kit --test combat_suite rule_elation
cargo test -p starclock-data production_elation
cargo fmt --all -- --check
cargo clippy -p starclock-combat --all-targets -- -D warnings
cargo test -p starclock-combat
```
