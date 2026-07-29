# Divergent Universe Arithmetic Mapping Boundary

## Released facts

The pinned CHS and EN TextMaps describe Arithmetic Mapping as a
Divergent-Universe-only enhancement of character levels, Light Cone levels,
Traces and Relics:

- character level is raised only when it is below the cap for the current
  Equilibrium Level;
- unlocked inactive Traces are activated, and below-required Trace levels are
  raised to the requirement;
- equipped Relics are replaced with compatible temporary Relics only when
  their total Enhancement Level is below the current requirement; and
- all mapping enhancements are effective only inside Divergent Universe.

Those conditional statements also define the already-sufficient boundary:
when a stated below-threshold predicate is false, the source does not authorize
replacing that field. The public text names Light Cone levels but does not
publish their exact comparison condition or temporary Light Cone identity.

The evidence locators are
`TextMap/TextMapEN.json#3050660410227566581` and
`TextMap/TextMapCHS.json#3050660410227566581`. Normalized summaries are
independently written and do not copy the long overview text.

## Structured catalogs

- `RogueTournBuildRefAvatar` contains 84 exact eligible catalog rows and their
  authored sort weights.
- `RogueTournAvatar` contains 79 avatar-to-opaque-`SpecialAvatarID` bindings.
- `RogueTournRole` contains 95 avatar-to-role-buff bindings.
- Every role buff resolves exactly in `RogueMazeBuff`, including its modifier
  name, in-battle binding key and four-to-seven canonical parameters.

The three catalogs deliberately have different stable-ID sets. Ten eligible
avatars have no special-avatar row, five special-avatar rows are absent from
the eligibility catalog, and sixteen role rows have no special-avatar row.
The normalized data preserves these differences rather than forcing a
one-to-one join.

Four role locators (`1014`, `1015`, `1508`, `1509`) lack a released
`AvatarConfig` identity in the fixed snapshot. They remain `Cataloged` numeric
source locators with no character-name claim. All other identities resolve
through `AvatarConfig` and the bilingual TextMaps.

## Unavailable fields and policy

The fixed released sources do not expose the per-avatar temporary Trace,
Light Cone or Relic loadout behind `SpecialAvatarID`, nor the exact Light Cone
threshold. These fields remain `Unspecified`; the opaque ID is retained
without inventing equipment.

Released text does not state when mapping is reevaluated after a party or
equipment change. The replaceable reference policy evaluates at run entry and
after an accepted party change, modifying only fields whose published
below-threshold predicate holds. At run finalization it removes all temporary
mapping state without changing account characters, Light Cones or Relics.
This policy is not observed parity and has no runtime lowering.

Replace the policy when released mapping-info rows, flow configuration or
reproducible observations establish exact temporary loadouts, Light Cone
conditions and refresh checkpoints.

## Reproduction

```text
node tools/divergent-universe-reference/import-arithmetic-mapping.mjs \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/import-arithmetic-mapping.mjs --check \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/verify-arithmetic-mapping.mjs \
  --source-cache <turnbasedgamedata-cache>
```

The verifier closes all 258 manifest receipts, resolves every role buff,
retains all catalog-set differences, rejects promotion of the four unresolved
public identities and requires explicit mode-only teardown.
