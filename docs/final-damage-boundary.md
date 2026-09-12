# Final damage multiplier boundary

`FormulaStage::DamageFinalMultiply` is a source-owned damage factor, separate
from `FinalMultiply` on an underlying stat and from the additive DamageBoost
stage. It has no content IDs or mode branches. DU Tawot state 9073 binds its
authored 50% component to this stage under an explicit scope and lifetime policy;
see [Curio battle stats](divergent-universe-curio-battle-stats.md).

## Resolution contract

- Ordinary Direct, DoT, Additional and Elation damage query this stage for
  their respective formula purpose on the source, with the existing contextual
  filters and snapshot rules. Target-owned final factors do not amplify hits.
- Initial Break, Super Break and subsequent base Break effect damage use the
  corresponding Break/SuperBreak purpose. An explicit applier takes precedence
  over the current turn actor, so a victim's turn does not transfer ownership
  of a continuing Break effect. This also applies to the earlier source-side
  Break formula inputs.
- Each stacking group retains its declared aggregation policy. The resulting
  groups multiply in canonical group order; an empty stage resolves to one.
  Negative evaluated operands reject the formula even if multiplying two
  negatives could produce a positive result. Zero is a valid multiplier.
- Apply the factor to the unfloored fixed-point raw damage after existing
  damage factors, using checked arithmetic and nearest-ties-even rounding.
  Derive applied integer damage by flooring that resulting raw value. Do not
  multiply a previously floored integer: `1.9 × 1.5` yields raw `2.85` and
  calculated damage `2`, not `1`. Damage guards, shields, HP capping, event
  collection and reaction processing remain downstream and unchanged.
- Source-modifier-bypassing operations, including TrueDamage programs, remain
  unamplified. A positive absolute `DamageOverride` still wins without further
  scaling. Healing, HP costs, base stats and action order do not query this stage.
- Numeric errors enter the existing deterministic fault boundary. The factor
  does not introduce RNG, a separate mutation path, or new Activity state.

## Verification scope

Pure vectors cover unfloored input, neutral/zero multipliers, fixed-point ties,
negative values and overflow. Real command regressions cover the four ordinary
classes, purpose and source/target isolation, multiplicative groups, coexistence
with stat and additive-damage modifiers, multi-hit absolute overrides, true
damage, all three Break event kinds and deterministic faults before HP damage.

These shared-engine tests establish the formula boundary, not source-program
completeness. Separate mode-owned tests exercise Tawot 9073's production passive,
paid acquisition, destruction/repair, verified-result lifetime and encoded replay.
Its uncertain applicability and stacking remain replaceable policy, not exact
observed parity; the complete five-battle public route remains pending.
