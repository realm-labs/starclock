# Unknowable Domain encounter boundary

Version 4.4 freezes 1,524 source parents: all 1,518
`RogueMagicRoom` rows and six distinct enemy identities directly displayed by
`RogueMagicArea.WorldLevel2DisplayMonster`.

## Resolved facts

- The six shared enemy identities join by exact stable source ID to the Goal 01
  enemy templates and variants. They retain bilingual names, exact variant IDs
  and the complete released display-level rows.
- The 13 areas define 13 area display pools. Their literal
  `DifficultyIDList` selectors and display order are preserved without assigning
  semantics that the source does not publish.
- The 1,518 room parents retain all ten released room types. The 832 Battle,
  Elite, Boss and Encounter rows are combat-capable boundaries; the other 686
  rows require no combat-wave expansion at this snapshot.

## Unpublished selector

Every `RogueMagicRoom` row contains only `RogueRoomID` and `RogueRoomType`.
The released
`Config/Level/Maze/MazeRogue/Rogue260/RogueMagic_Group_Monster.json`
program finishes the room but contains no stage, monster or encounter-group
selector. No `StageConfig` row contains a `RogueMagic`, `Rogue260` or
`MagicRogue` selector.

Reverse lookup is deliberately rejected. The six identities happen to occur in
53 StageConfig rows, 26 RogueMonster rows and 34 RogueMonsterGroup rows, but
those records span shared and other-mode content. Same enemy identity, table
prefix, ID range or apparent namespace cannot prove Unknowable Domain
reachability. Consequently this batch imports zero accepted encounter groups,
zero waves and zero enemy slots.

Each combat-capable room is `UnresolvedNoReleasedSelector`; each displayed boss
is `EnemyIdentityResolvedStageUnresolved`. Both states are nonblocking reference
boundaries, remain `DataReady`, and fail closed. They may be replaced only by
released structured data that publishes a forward Unknowable Domain
room/area-to-`RogueMonsterGroup`, `RogueMonster` or `StageConfig` selector.

No row in this batch is runtime-lowered.
