# Goal 16 Departure Growth and Inventory Audit

`G16-P1-B3` freezes the Version 2.2 experience/level loop, Adventure Strategy
candidates, decision resources, slot capacities and deterministic inventory
failure boundaries.

## Experience and team level

The released base program initializes `expForLevel` to exact value `40`. The
structured constants additionally freeze:

- wave multiplier `0.27`;
- level-scaling parameters `0.1`, `1`, `12`;
- normal-enemy experience `2` and `4`;
- elite experience `8`; and
- boss/special experience `0`.

The normalization retains a structural program digest and operation/event
identifiers without copying the raw ability program. It does not infer an
undocumented maximum level or alternate threshold curve.

## Candidate and Adventure Strategy closure

All 11 `EvolveBuildCardConfig` rows resolve exactly to their level-one
MazeBuff rows and card-program binding. Each normalized candidate retains its
stable source IDs, bilingual name, type, exact card/MazeBuff parameters,
selectable-period list and a structural program summary.

The exact candidate-control parameters are:

| Parameter | Value |
|---|---|
| Source weight vector | `18,6,3,3,7,6,2,0,2,0,7` |
| Refresh uses | `3` |
| Exclusion/removal uses | `2` |
| Card refresh uses | `0` |
| Refresh unlock quest | `6070100` |
| Exclusion unlock quest | `6070102` |
| Skip unlock quest | `6070207` |

The source vector is retained without guessing which card/category each
ordinal weights. Hidden offer order, complete weights, refresh memory and
empty-pool behavior remain `ProjectPolicy`: labeled integer RNG, stable
Starclock IDs, one-refresh displayed-ID exclusion with bounded fallback, and
an explicit no-candidate outcome that consumes no inventory resource.

## Slots and failure invariance

| Scope | Weapons | Accessories |
|---|---:|---:|
| Standard initially unlocked | 4 | 4 |
| Standard total capacity | 5 | 6 |
| Origin stage fixed capacity | 3 | 4 |

Five ReferenceOnly inventory operations cover new acquisition, duplicate
upgrade, maximum-level duplicate rejection, full-inventory rejection and slot
expansion. Each carries:

- selected deterministic behavior;
- two rejected alternatives;
- battle-local state ownership;
- affected fixture families;
- low confidence and a released-evidence replacement condition; and
- byte-identical state on failure.

These policies preserve exact slot/max-level facts but are not parity claims.

## Verification

The deterministic generator and verifier cover one threshold contract, 11
strategies, two candidate policies, four slot policies and five inventory
operations:

```text
node tools/galactic-baseballer-reference/normalize-departure-growth.mjs \
  --source-cache .cache/galactic-baseballer-source
node tools/galactic-baseballer-reference/verify-departure-growth.mjs \
  --source-cache .cache/galactic-baseballer-source
```
