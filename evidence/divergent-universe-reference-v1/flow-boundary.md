# Divergent Universe Version 4.4 Flow Boundary

## Scope

This dossier records the `G11-P1-B1` entry and stage-flow boundary for the
pinned Version 4.4 sources. It is reference evidence only and does not claim a
runtime state machine.

## Exact selectors

- `ExcelOutput/RogueActivityResidentConfig.json#104` selects `TournRogue`,
  activity `105` and activity module `6002201`.
- `ExcelOutput/RogueTournModule.json#7` resolves module `6002201` to main
  tournament `3` and sub-tournament `1`.
- `ExcelOutput/RogueCommonModeTitle.json#7` binds the stable `TournRogue`
  title. Released TextMaps name the common mode “Divergent Universe” /
  “差分宇宙” and the resident Version 4.4 activity “Divergent Universe:
  Arcadian Chronicles” / “差分宇宙•乐园漫记”.
- Twenty-eight `RogueTournArea` rows have the exact `Tourn3` selector:
  two `Guide`, thirteen `Formal` and thirteen `WeekChallenge`.
- Their stable reference closure contains twenty-two
  `RogueTournDifficulty` rows and eleven `RogueTournLayer` rows.
- Thirteen `RogueTournFinishway` rows explicitly contain
  `Cond_InRogueTournMode(3)`.

Every normalized row carries the exact source path, zero-based row locator and
SHA-256 evidence digest. Large TextMap hashes are parsed as decimal strings;
the evidence digest is computed from the ordinary JSON-parsed source row so it
matches the frozen manifest.

## Room boundary

The pinned snapshot contains:

- zero `RogueTournRoom` rows with `TournMode = Tourn3`;
- zero `RogueTournLayerRoom` rows whose `LayerID` belongs to the eleven-layer
  current closure; and
- 848 `RogueTournRoom` rows with `TournMode = Tourn2`.

The absence of a `Tourn3` room partition does not prove that every `Tourn2`
room is currently offered. Goal 11 therefore retains all 848 rows as
`Shared`/`Cataloged` review records with
`reachability_disposition = UnprovenSharedCandidate`. They have no offered
pool or StageConfig membership and cannot satisfy `DataReady`.

Each candidate is replaced only when released stage/config evidence proves an
exact promotion or exclusion using its source path, row locator and evidence
digest. Prefix, numerical range, adjacent IDs and the fact that a row exists
in a `RogueTourn` table are not membership evidence.

## Flow policy

Area rows provide an ordered layer list but do not publish room selection,
transition timing, field-level carry behavior or terminal reset order. The
generated `stage-flow.json` therefore uses the explicitly replaceable
`ordered-tourn3-area-layer-flow-v1` `ProjectPolicy`:

1. enter the first authored layer;
2. advance through the authored layer order;
3. reach the area terminal after the final authored layer;
4. carry run inventory, Equation progress and temporary builds across
   room/layer boundaries; and
5. clear run state and temporary builds while preserving permanent unlocks
   after finalization.

This policy preserves deterministic reference semantics but is not presented
as observed parity. It must be replaced when released flow configuration or
reproducible observations bind the missing field-level behavior.

## Reproduction

```text
node tools/divergent-universe-reference/import-flow.mjs \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/import-flow.mjs --check \
  --source-cache <turnbasedgamedata-cache>
node tools/divergent-universe-reference/verify-flow.mjs \
  --source-cache <turnbasedgamedata-cache>
```

The verifier requires exact manifest receipts for the entry, module, finish,
area, difficulty, layer and room-candidate categories. It also rejects any
room candidate promoted to an offered pool without stronger evidence.
