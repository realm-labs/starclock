# Currency Wars Version 4.4 Flow Boundary

## Scope

This dossier records the `G12-P1-B1` entry, Gambit and three-Plane Node-flow
boundary for the pinned Version 4.4 sources. It is reference evidence only and
does not claim a runtime state machine.

## Exact Currency Wars identity

Released EN/CHS TextMap rows explicitly name:

- `TextMapEN.json#3667032256414715511` and the matching CHS locator:
  “Currency Wars” / “货币战争”;
- `#16168571866306406443`: “Standard Gambit” / “标准博弈”;
- `#6780709645179175648`: “Overclock Gambit” / “超频博弈”;
- `#6393633547126864112`: every match contains three Planes and every Plane
  contains multiple Nodes; and
- `#7693488975416237801`: the released bilingual gameplay explanation covering
  the three-Plane victory boundary and both Gambit modes.

The structured host selectors are:

- `ExcelOutput/RogueActivityResidentConfig.json#4`,
  SHA-256 `aa1c9804788a1ee9b74dbf1230768fee695e654d2cd56de595ad235c301e729c`,
  selecting activity `105`, `TournRogue` and module `6002201`;
- `ExcelOutput/RogueTournModule.json#7`,
  SHA-256 `fc9f5c3b215a895c3d6e23d6058289b416cb2f8b2f173d12320e107847857bac`,
  resolving module `6002201` to main tournament `3`, sub-tournament `1`;
- `ExcelOutput/RogueCommonModeTitle.json#3`,
  SHA-256 `7823313890a4ad493cb0ae08a815b5fd1ede087f694f34c80513f95bda79b5ce`,
  binding the host `TournRogue` title; and
- `ExcelOutput/RogueTournAreaGroupByTourn.json#2`,
  SHA-256 `79e3c0b89c994e0a77c062608a54582cd04120d967a4773836d60a7fda4cf337`,
  binding the `Tourn3` Guide area group.

The host title rows say “Divergent Universe” / “差分宇宙”; they do not erase
the distinct released Currency Wars name and mechanics. Goal 11 assigns the
same host selector to Divergent Universe, so Goal 12 retains the exact
source-locator/digest conflict for `G12-P4-B3` instead of modifying Goal 11.

## Area, Plane and Node closure

Twenty-eight `RogueTournArea` rows have the exact `Tourn3` selector: two
`Guide`, thirteen `Formal` and thirteen `WeekChallenge`. Their stable
references close to twenty-two `RogueTournDifficulty` rows and eleven
`RogueTournLayer` rows. Thirteen `RogueTournFinishway` rows explicitly contain
`Cond_InRogueTournMode(3)`.

Unlike the legacy Tourn room path, Currency Wars publishes sixty exact Node
positions in `RoguePersonaLayerRoom`. They cover the same eleven selected
layer IDs and use contiguous one-based ordinals. `RogueTournLayer.LayerNumID`
maps the released run layers to Plane ordinals 1, 2 and 3. Fixed Nodes bind a
`RoguePersonaRoomPreset`; other Nodes bind the released ordered random
composition-type list.

The generated output retains thirty-four exact room presets plus the fixed and
random type-pool constants as thirty-six Domain-composition records. Detailed
room types, attributes, strategies and Persona activation remain owned by
`G12-P1-B8`; this batch records only the topology needed for Node flow.

## Gambit and transition policy

The released bilingual rule text proves the two Gambit identities and their
high-level entry/rank behavior. The fixed structured data does not publish a
direct area-to-Gambit selector. Goal 12 therefore records this replaceable
field policy:

- `Formal` Tourn3 areas map to Standard Gambit;
- paired `WeekChallenge` Tourn3 areas map to Overclock Gambit; and
- `Guide` areas remain tutorial-only.

The area records publish ordered layer lists, and Persona layer-room records
publish ordered Nodes. They do not publish same-boundary transition timing,
field-level carry behavior or final reset order. `stage-flow.json` therefore
uses the explicit `ordered-tourn3-area-layer-flow-v1` `ProjectPolicy`: enter
the first authored Plane, advance in authored order, evaluate terminal state
after the final Plane, carry roster/economy/Squad HP/run inventory between
Nodes and Planes, then clear run-scoped state while preserving rank unlocks.

This deterministic policy is not presented as observed parity. Replace it when
released flow configuration or reproducible observations bind the missing
field-level timing and carry/reset behavior.

## Fail-closed shared room boundary

The snapshot has no `RogueTournRoom` row with `TournMode = Tourn3` and no
legacy `RogueTournLayerRoom` row for the selected eleven layers. All 848
`Tourn2` room rows therefore remain manifest-only `EvidenceOnly` /
`PendingStageClosure` obligations. `rooms.json` is intentionally empty:
neither a prefix, ID range nor the existence of an older shared row promotes
it into Currency Wars.

`G12-P2-B5` may replace each obligation only with an exact Stage/config
promotion or exclusion receipt using source path, row locator and digest.

## Reproduction

```text
node tools/currency-wars-reference/import-flow.mjs \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/import-flow.mjs --check \
  --source-cache <turnbasedgamedata-repository>
node tools/currency-wars-reference/verify-flow.mjs \
  --source-cache <turnbasedgamedata-repository>
```

The verifier binds every structured entry, module, finish, area, difficulty,
layer and Persona Node source receipt to the frozen manifest. It rejects a
promoted room candidate, a non-contiguous Node sequence or any drift in the
two Gambit identities.
