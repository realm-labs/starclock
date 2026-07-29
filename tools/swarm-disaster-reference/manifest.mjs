#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(args.find((argument) => !argument.startsWith("--")) ?? ".");
const sourceRoot = path.join(root, ".cache/content-reference/turnbasedgamedata");
const output = path.join(
  root,
  "content-manifests/swarm-disaster-v1/content-manifest.json",
);
const categories = {};

function sourceRows(file) {
  const rows = JSON.parse(fs.readFileSync(path.join(sourceRoot, file), "utf8"));
  if (!Array.isArray(rows)) throw new Error(`expected source array: ${file}`);
  return rows.map((row, index) => ({ row, index, file }));
}
function sourceRecord(entry, id, fields = {}) {
  return {
    id: String(id),
    source: `${entry.file}#${entry.index}`,
    evidence_sha256: digest(entry.row),
    evidence_quality: "ExactStructured",
    ...fields,
  };
}
function nestedRecord(file, key, value, id, fields = {}) {
  return {
    id: String(id),
    source: `${file}#${key}`,
    evidence_sha256: digest(value),
    evidence_quality: "ExactStructured",
    ...fields,
  };
}
function inheritedRecords(file, predicate = () => true) {
  const relative = `content-reference/standard-universe-v1/${file}`;
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"))
    .filter(predicate)
    .map((row) => ({
      id: row.id,
      source: relative,
      evidence_sha256: digest(row),
      evidence_quality: "ExactStructured",
      ownership: "Shared",
      reachability: "InheritedSharedPool",
    }));
}
function category(id, membershipBasis, records) {
  const ordered = [...records].sort((left, right) => compare(left.id, right.id));
  if (new Set(ordered.map(({ id: recordId }) => recordId)).size !== ordered.length)
    throw new Error(`duplicate IDs in category ${id}`);
  categories[id] = {
    id,
    membership_basis: membershipBasis,
    count: ordered.length,
    records: ordered,
  };
  return ordered;
}

const activityEntries = sourceRows("ExcelOutput/RogueActivityResidentConfig.json");
const entrances = sourceRows("ExcelOutput/RogueDLCEntrance.json");
const modeTitles = sourceRows("ExcelOutput/RogueCommonModeTitle.json");
category(
  "profiles",
  "One project profile for the Version 4.4 ChessRogue activity boundary.",
  [{
    id: "swarm-disaster-v1",
    source: "policy/goal09-foundation.json",
    evidence_sha256: digest({
      goal_id: "swarm-disaster-reference-v1",
      sub_mode: "ChessRogue",
      game_version: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "SwarmDisaster",
    reachability: "Direct",
  }],
);
category(
  "entry_points",
  "The resident activity, DLC entrance and common title rows whose sub-mode is ChessRogue.",
  [
    ...activityEntries.filter(({ row }) => row.SubMode === "ChessRogue")
      .map((entry) => sourceRecord(entry, `activity:${entry.row.ActivityID}`, {
        ownership: "SwarmDisaster", reachability: "Direct",
      })),
    ...entrances.filter(({ row }) => row.SubType === "ChessRogue")
      .map((entry) => sourceRecord(entry, `entrance:${entry.row.ID}`, {
        ownership: "SwarmDisaster", reachability: "Direct",
      })),
    ...modeTitles.filter(({ row }) => row.SubMode === "ChessRogue")
      .map((entry) => sourceRecord(entry, `title:${entry.row.SubMode}`, {
        ownership: "SwarmDisaster", reachability: "Direct",
      })),
  ],
);

const areas = sourceRows("ExcelOutput/RogueDLCArea.json")
  .filter(({ row }) => row.SubType === "ChessRogue");
category(
  "guide_areas",
  "ChessRogue area rows outside the five Formal difficulties.",
  areas.filter(({ row }) => row.AreaGroupID !== "Formal")
    .map((entry) => sourceRecord(entry, entry.row.AreaID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "formal_difficulties",
  "The five ChessRogue Formal area rows, Difficulty_1 through Difficulty_5.",
  areas.filter(({ row }) => row.AreaGroupID === "Formal")
    .map((entry) => sourceRecord(entry, entry.row.AreaID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      difficulty: entry.row.Difficulty,
    })),
);
const difficultyIds = new Set(areas.flatMap(({ row }) => row.DifficultyID));
category(
  "difficulty_segments",
  "Shared RogueDLCDifficulty rows directly referenced by ChessRogue areas.",
  sourceRows("ExcelOutput/RogueDLCDifficulty.json")
    .filter(({ row }) => difficultyIds.has(row.DifficultyID))
    .map((entry) => sourceRecord(entry, entry.row.DifficultyID, {
      ownership: "Shared", reachability: "Referenced",
    })),
);
const layerIds = new Set(areas.flatMap(({ row }) => row.LayerIDList));
category(
  "planes",
  "Shared DLC layer rows directly referenced by ChessRogue areas.",
  sourceRows("ExcelOutput/RogueDLCLayer.json")
    .filter(({ row }) => layerIds.has(row.LayerID))
    .map((entry) => sourceRecord(entry, entry.row.LayerID, {
      ownership: "Shared", reachability: "Referenced",
    })),
);

const chessboards = sourceRows("ExcelOutput/RogueDLCChessBoard.json")
  .filter(({ row }) => !row.ChessBoardConfiguration.includes("/MapRepo160/"));
category(
  "chessboards",
  "DLC chessboard rows whose configuration path is outside MapRepo160.",
  chessboards.map((entry) => sourceRecord(entry, entry.row.ChessBoardID, {
    ownership: "SwarmDisaster",
    reachability: "Direct",
    config_path: entry.row.ChessBoardConfiguration,
  })),
);
const columns = [];
const nodes = [];
const boardEvents = [];
const blockCreateRules = [];
const domainEvidence = new Map();
const beaconEvidence = new Map();
for (const board of chessboards) {
  const relative = board.row.ChessBoardConfiguration;
  const config = JSON.parse(fs.readFileSync(path.join(sourceRoot, relative), "utf8"));
  const positions = new Map();
  for (const [nodeId, node] of Object.entries(config.RogueChestGridItemMap ?? {})) {
    const columnId = String(node.PosX ?? 0);
    if (!positions.has(columnId)) positions.set(columnId, node);
    nodes.push(nestedRecord(relative, `RogueChestGridItemMap/${nodeId}`, node,
      `${board.row.ChessBoardID}:${nodeId}`, {
        ownership: "SwarmDisaster",
        reachability: "Direct",
        chessboard_id: String(board.row.ChessBoardID),
        column: Number(columnId),
      }));
    for (const domain of node.BlockTypeList ?? [])
      if (!domainEvidence.has(domain))
        domainEvidence.set(domain, { file: relative, key: nodeId, value: node });
  }
  for (const [columnId, node] of positions)
    columns.push(nestedRecord(relative, `column/${columnId}`, {
      chessboard_id: board.row.ChessBoardID,
      position_x: Number(columnId),
    }, `${board.row.ChessBoardID}:${columnId}`, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      chessboard_id: String(board.row.ChessBoardID),
    }));
  for (const [eventId, event] of Object.entries(config.RogueChestEventMap ?? {}))
    boardEvents.push(nestedRecord(relative, `RogueChestEventMap/${eventId}`, event,
      `${board.row.ChessBoardID}:${eventId}`, {
        ownership: "SwarmDisaster",
        reachability: "Direct",
        chessboard_id: String(board.row.ChessBoardID),
      }));
  for (const rule of config.RogueBlockCreateGroupList ?? []) {
    blockCreateRules.push(nestedRecord(
      relative,
      `RogueBlockCreateGroupList/${rule.BlockCreateID}`,
      rule,
      `${board.row.ChessBoardID}:${rule.BlockCreateID}`,
      {
        ownership: "SwarmDisaster",
        reachability: "Direct",
        chessboard_id: String(board.row.ChessBoardID),
        domain: rule.BlockType,
      },
    ));
    if (!domainEvidence.has(rule.BlockType))
      domainEvidence.set(rule.BlockType, {
        file: relative,
        key: `block/${rule.BlockCreateID}`,
        value: rule,
      });
    for (const mark of rule.MarkCreateRandomList ?? [])
      if (mark.TypeID !== undefined && !beaconEvidence.has(mark.TypeID))
        beaconEvidence.set(mark.TypeID, {
          file: relative,
          key: `mark/${rule.BlockCreateID}/${mark.TypeID}`,
          value: mark,
        });
  }
}
category(
  "map_columns",
  "Distinct authored PosX columns within every reachable ChessRogue chessboard.",
  columns,
);
category(
  "map_nodes",
  "Every authored RogueChestGridItemMap entry in reachable ChessRogue chessboards.",
  nodes,
);
category(
  "map_events",
  "Every authored RogueChestEventMap entry in reachable ChessRogue chessboards.",
  boardEvents,
);
category(
  "block_create_rules",
  "Every authored RogueBlockCreateGroupList row in reachable ChessRogue chessboards.",
  blockCreateRules,
);
category(
  "domains",
  "Distinct BlockType values reachable from ChessRogue nodes or block-create rules.",
  [...domainEvidence.entries()].map(([domain, evidence]) =>
    nestedRecord(evidence.file, evidence.key, evidence.value, domain, {
      ownership: "Shared", reachability: "Referenced",
    })),
);
category(
  "beacons",
  "Distinct MarkCreateRandomList TypeID values reachable from ChessRogue boards.",
  [...beaconEvidence.entries()].map(([beacon, evidence]) =>
    nestedRecord(evidence.file, evidence.key, evidence.value, beacon, {
      ownership: "Shared", reachability: "Referenced",
    })),
);

const roomBindings = sourceRows("ExcelOutput/RogueDLCRoom.json")
  .filter(({ row }) => row.RogueSubMode === "ChessRogue");
category(
  "room_bindings",
  "All released ChessRogue room-to-section membership rows.",
  roomBindings.map((entry) => sourceRecord(entry, entry.row.RogueRoomID, {
    ownership: "SwarmDisaster", reachability: "Direct",
  })),
);
const roomIds = new Set(roomBindings.map(({ row }) => row.RogueRoomID));
category(
  "adventure_outcomes",
  "Shared abstract Adventure definitions whose room is reachable in ChessRogue.",
  sourceRows("ExcelOutput/RogueDLCAdventureRoom.json")
    .filter(({ row }) => roomIds.has(row.RoomID))
    .map((entry) => sourceRecord(entry, entry.row.RoomID, {
      ownership: "Shared",
      reachability: "Referenced",
      adventure_type: entry.row.AdventureType,
    })),
);
const bossChoices = new Map();
for (const area of areas)
  for (const [monsterId, level] of Object.entries(area.row.DisplayMonsterMap ?? {}))
    if (!bossChoices.has(monsterId))
      bossChoices.set(monsterId, { entry: area, level });
category(
  "boss_choices",
  "Distinct displayed boss/enemy identities referenced by ChessRogue areas.",
  [...bossChoices.entries()].map(([monsterId, { entry, level }]) =>
    sourceRecord(entry, monsterId, {
      ownership: "Shared",
      reachability: "Referenced",
      display_level: level,
    })),
);

category(
  "mode_constants",
  "All released common RogueDLC constants retained for lifecycle and unlock review.",
  sourceRows("ExcelOutput/RogueDLCConstValueCommon.json")
    .map((entry) => sourceRecord(entry, entry.row.ConstValueName, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "boss_decay_levels",
  "All released RogueDLC Boss Decay and Planar Disarray definitions.",
  sourceRows("ExcelOutput/RogueDLCBossDecay.json")
    .map((entry) => sourceRecord(entry, entry.row.BossDecayID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);

const aeons = sourceRows("ExcelOutput/RogueDLCAeon.json");
category(
  "audience_paths",
  "All eight released ChessRogue selectable Path/Audience Die definitions.",
  aeons.map((entry) => sourceRecord(entry, entry.row.AeonID, {
    ownership: "SwarmDisaster", reachability: "Direct",
  })),
);
category(
  "audience_dice",
  "All eight released Path-specific Audience Dice.",
  sourceRows("ExcelOutput/RogueDLCAeonDice.json")
    .map((entry) => sourceRecord(entry, entry.row.AeonDiceID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "dice_faces",
  "All released Path-specific Audience Die faces.",
  sourceRows("ExcelOutput/RogueDLCAeonDiceSurface.json")
    .map((entry) => sourceRecord(entry, entry.row.AeonSurfaceDiceID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      aeon_dice_id: String(entry.row.AeonDiceID),
    })),
);
category(
  "dice_rarities",
  "All released Audience Die face rarity definitions.",
  sourceRows("ExcelOutput/RogueDLCDiceSurfaceRarity.json")
    .map((entry) => sourceRecord(entry, entry.row.Rarity, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);

category(
  "communing_choices",
  "Main-story branch rows with an explicit Aeon alignment, retained as Communing Device choices.",
  sourceRows("ExcelOutput/RogueDLCMainStoryBranch.json")
    .filter(({ row }) => row.AeonID !== undefined)
    .map((entry) => sourceRecord(entry, entry.row.MainStoryBranchID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      aeon_id: String(entry.row.AeonID),
    })),
);
category(
  "pathstrider_cabinets",
  "All released normal and hidden Pathstrider cabinet objectives and point changes.",
  sourceRows("ExcelOutput/RogueDLCAeonCabinet.json")
    .map((entry) => sourceRecord(entry, entry.row.CabinetID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      cabinet_type: entry.row.CabinetType,
    })),
);
category(
  "communing_dimensions",
  "All seven released Communing Trail Path dimensions.",
  sourceRows("ExcelOutput/RogueDLCAeonDimension.json")
    .map((entry) => sourceRecord(entry, entry.row.AeonDimensionID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "communing_trail_nodes",
  "All released Communing Trail nodes with one gameplay effect binding each.",
  sourceRows("ExcelOutput/RogueDLCAeonTalent.json")
    .map((entry) => sourceRecord(entry, entry.row.AeonTalentID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      dimension_id: String(entry.row.AeonDimensionID),
    })),
);
category(
  "pathstrider_finish_conditions",
  "All released DLC finish/progress conditions used by the Pathstrider and unlock graphs.",
  sourceRows("ExcelOutput/RogueDLCFinishWay.json")
    .map((entry) => sourceRecord(entry, entry.row.ID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "pathstrider_unlocks",
  "All released DLC unlock rows; later normalization separates simulation-visible consequences.",
  sourceRows("ExcelOutput/RogueDLCUnlock.json")
    .map((entry) => sourceRecord(entry, entry.row.RogueUnlockID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);
category(
  "mechanical_chapter_locators",
  "Main-story rows retained only for layer, Path-dimension, point and bonus unlock locators.",
  sourceRows("ExcelOutput/RogueDLCMainStory.json")
    .map((entry) => sourceRecord(entry, entry.row.MainStoryID, {
      ownership: "SwarmDisaster",
      reachability: "Direct",
      retained_fields: ["Layer", "UnlockAeonDimension", "UnlockPoint", "IsBonusUnlock"],
    })),
);

const allPaths = JSON.parse(fs.readFileSync(
  path.join(root, "content-reference/standard-universe-v1/paths.json"),
  "utf8",
));
const reachablePathIds = new Set(allPaths
  .filter(({ buff_type: buffType }) => buffType >= 120 && buffType <= 127)
  .map(({ id }) => id));
const reachableBlessingIds = new Set(JSON.parse(fs.readFileSync(
  path.join(root, "content-reference/standard-universe-v1/blessings.json"),
  "utf8",
)).filter(({ path_id: pathId }) => reachablePathIds.has(pathId))
  .map(({ id }) => id));
category(
  "paths",
  "The eight Goal 03 stable Path identities whose buff type is 120 through 127.",
  inheritedRecords("paths.json", ({ id }) => reachablePathIds.has(id)),
);
category(
  "resonances",
  "All 32 Goal 03 Resonance/Formation identities owned by the eight reachable Paths.",
  inheritedRecords("resonances.json", ({ path_id: pathId }) =>
    reachablePathIds.has(pathId)),
);
category(
  "blessings",
  "All 144 Goal 03 Blessing identities owned by the eight reachable Paths.",
  inheritedRecords("blessings.json", ({ id }) => reachableBlessingIds.has(id)),
);
category(
  "blessing_levels",
  "Both authored levels of every reachable shared Blessing.",
  inheritedRecords("blessing-levels.json", ({ blessing_id: blessingId }) =>
    reachableBlessingIds.has(blessingId)),
);
category(
  "path_boosts",
  "One Swarm Disaster mode-entry Path boost for each selectable Aeon.",
  aeons.map((entry) => sourceRecord(entry, entry.row.EffectParam1[0], {
    ownership: "SwarmDisaster",
    reachability: "Direct",
    aeon_id: String(entry.row.AeonID),
  })),
);
const buffGroups = sourceRows("ExcelOutput/RogueBuffGroup.json");
const groupById = new Map(buffGroups.map((entry) => [entry.row.GMLOGNJAIGI, entry]));
const interplayRecords = [];
for (const cross of sourceRows("ExcelOutput/RogueDLCAeonCross.json")) {
  const group = groupById.get(cross.row.BuffGroup);
  if (!group || group.row.HECJCAMDGNO.length !== 1)
    throw new Error(`invalid RogueDLC interplay group ${cross.row.BuffGroup}`);
  interplayRecords.push(sourceRecord(cross, group.row.HECJCAMDGNO[0], {
    ownership: "SwarmDisaster",
    reachability: "Direct",
    main_aeon_id: String(cross.row.MainAeonID),
    sub_aeon_id: String(cross.row.SubAeonID),
    buff_group_id: String(cross.row.BuffGroup),
  }));
}
category(
  "resonance_interplays",
  "All released RogueDLC main/sub-Aeon Resonance Interplay bindings.",
  interplayRecords,
);
category(
  "trailblaze_bonuses",
  "Swarm Disaster-owned RogueBonus IDs 101 through 106.",
  sourceRows("ExcelOutput/RogueBonus.json")
    .filter(({ row }) => row.BonusID >= 101 && row.BonusID <= 106)
    .map((entry) => sourceRecord(entry, entry.row.BonusID, {
      ownership: "SwarmDisaster", reachability: "Direct",
    })),
);

const curioHandbooks = sourceRows("ExcelOutput/RogueHandbookMiracle.json")
  .filter(({ row }) => row.MiracleTypeList.includes(130));
const curioHandbookIds = new Set(curioHandbooks.map(({ row }) =>
  row.MiracleHandbookID));
const curioCopies = sourceRows("ExcelOutput/RogueMiracle.json")
  .filter(({ row }) => row.MiracleID >= 1000
    && row.MiracleID < 2000
    && curioHandbookIds.has(row.UnlockHandbookMiracleID));
const copyByHandbook = new Map(curioCopies.map((entry) => [
  entry.row.UnlockHandbookMiracleID,
  entry,
]));
category(
  "curios",
  "All handbook Curios whose mode membership includes type 130 (Swarm Disaster).",
  curioHandbooks.map((entry) => {
    const copy = copyByHandbook.get(entry.row.MiracleHandbookID);
    if (!copy) throw new Error(`missing Swarm Curio copy ${entry.row.MiracleHandbookID}`);
    return sourceRecord(entry, entry.row.MiracleHandbookID, {
      ownership: entry.row.MiracleTypeList.includes(100)
        ? "Shared"
        : "SwarmDisaster",
      reachability: "Direct",
      mode_copy_id: String(copy.row.MiracleID),
    });
  }),
);
category(
  "curio_states",
  "Exactly one 1000-series Swarm Disaster mode copy for every reachable Curio.",
  curioCopies.map((entry) => sourceRecord(entry, entry.row.MiracleID, {
    ownership: "SwarmDisaster",
    reachability: "Referenced",
    handbook_id: String(entry.row.UnlockHandbookMiracleID),
  })),
);

const occurrenceHandbooks = sourceRows("ExcelOutput/RogueHandBookEvent.json")
  .filter(({ row }) => row.EventTypeList.includes(130));
category(
  "occurrences",
  "All handbook Occurrences whose mode membership includes type 130.",
  occurrenceHandbooks.map((entry) => sourceRecord(entry, entry.row.EventHandbookID, {
    ownership: entry.row.EventTypeList.includes(100)
      ? "Shared"
      : "SwarmDisaster",
    reachability: "Direct",
  })),
);
const occurrenceVariants = new Map();
for (const handbook of occurrenceHandbooks)
  for (const progress of handbook.row.UnlockNPCProgressIDList ?? []) {
    const progressId = progress.FDOELDMEBPE;
    if (progressId >= 100000 && progressId < 200000) {
      if (!occurrenceVariants.has(progressId))
        occurrenceVariants.set(progressId, {
          handbook,
          progress,
          handbookIds: new Set(),
        });
      occurrenceVariants.get(progressId).handbookIds.add(
        handbook.row.EventHandbookID,
      );
    }
  }
category(
  "occurrence_variants",
  "Distinct 100000-series NPC progress variants reachable from type-130 handbooks.",
  [...occurrenceVariants.entries()].map(([
    progressId,
    { handbook, progress, handbookIds },
  ]) =>
    nestedRecord(
      handbook.file,
      `${handbook.index}/UnlockNPCProgressIDList/${progressId}`,
      progress,
      progressId,
      {
        ownership: "SwarmDisaster",
        reachability: "Referenced",
        handbook_ids: [...handbookIds].map(String).sort(compare),
      },
    )),
);
category(
  "shared_services",
  "The 15 non-bonus service definitions inherited from the Goal 03 reference pack.",
  inheritedRecords("services.json", ({ id }) => !id.includes(".trailblaze-bonus.")),
);

const fixtureFamilies = [
  "profile-entry",
  "topology-generation",
  "topology-event-order",
  "domain-replacement",
  "beacon-copy-and-blanking",
  "boss-choice-consequence",
  "countdown-lifecycle",
  "planar-disarray-transition",
  "boss-decay-stack",
  "audience-die-passive",
  "dice-face-targeting",
  "dice-roll-reroll-cheat",
  "communing-choice",
  "communing-dimension-points",
  "communing-trail-effect",
  "pathstrider-progress",
  "path-and-propagation-unlock",
  "resonance-interplay",
  "curio-lifecycle",
  "occurrence-choice",
  "service-and-adventure",
  "encounter-selection",
  "final-boss-consequence",
];
category(
  "semantic_fixture_families",
  "Minimum distinct semantic families; B4 freezes fixture shape and later batches may add cases without shrinking these obligations.",
  fixtureFamilies.map((id) => ({
    id,
    source: "docs/goals/09-swarm-disaster-reference-data.md",
    evidence_sha256: digest({ id, goal_id: "swarm-disaster-reference-v1" }),
    evidence_quality: "ProjectPolicy",
    ownership: "SwarmDisaster",
    reachability: "Direct",
  })),
);

const storyAccountExclusions = [
  ...sourceRows("ExcelOutput/RogueDLCEndGameReward.json"),
  ...sourceRows("ExcelOutput/RogueDLCMainStoryReward.json"),
  ...sourceRows("ExcelOutput/RogueDLCMainStoryBranch.json")
    .filter(({ row }) => row.AeonID === undefined),
  ...sourceRows("ExcelOutput/RogueDLCSubStory.json"),
  ...sourceRows("ExcelOutput/RogueDLCSubStoryGroup.json"),
].map((entry) => sourceRecord(
  entry,
  `${path.basename(entry.file)}:${entry.index}`,
  {
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: "story, collection or account-reward presentation without a direct rule row",
  },
));
const sourceInventory = JSON.parse(fs.readFileSync(
  path.join(root, "content-manifests/swarm-disaster-v1/source-inventory.json"),
  "utf8",
));
const boardPaths = new Set(chessboards.map(({ row }) => row.ChessBoardConfiguration));
const unreferencedTopology = sourceInventory.records
  .filter(({ family }) => family === "swarm_disaster_topology_candidate")
  .filter(({ path: sourcePath }) => !boardPaths.has(sourcePath))
  .map((record) => ({
    id: record.path,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: "non-MapRepo160 config absent from every released chessboard row",
  }));

const group = (...categoryIds) => ({
  categories: categoryIds,
  required: categoryIds.reduce((sum, id) => sum + categories[id].count, 0),
});
const counterGroups = {
  profiles_entries_bonuses:
    group("profiles", "entry_points", "trailblaze_bonuses"),
  difficulties_and_unlocks:
    group("formal_difficulties", "difficulty_segments"),
  topology:
    group(
      "guide_areas", "planes", "chessboards", "map_columns", "map_nodes",
      "map_events", "block_create_rules", "room_bindings", "domains", "beacons",
      "boss_choices",
    ),
  countdown_disarray_decay: group("mode_constants", "boss_decay_levels"),
  paths_and_audience_dice: group("audience_paths", "audience_dice"),
  dice_faces_rarities_controls: group("dice_faces", "dice_rarities"),
  communing_device_cabinets_dimensions:
    group("communing_choices", "pathstrider_cabinets", "communing_dimensions"),
  communing_trail: group("communing_trail_nodes"),
  pathstrider_objectives_unlocks:
    group(
      "pathstrider_finish_conditions", "pathstrider_unlocks",
      "mechanical_chapter_locators",
    ),
  paths_resonances_interplays:
    group("paths", "resonances", "path_boosts", "resonance_interplays"),
  blessings: group("blessings", "blessing_levels"),
  curios: group("curios", "curio_states"),
  occurrences: group("occurrences", "occurrence_variants"),
  services_beacons_adventure:
    group("shared_services", "beacons", "adventure_outcomes", "trailblaze_bonuses"),
  encounter_source_obligations: group("room_bindings", "boss_choices"),
  mechanic_rule_families: group("semantic_fixture_families"),
  semantic_fixture_families: group("semantic_fixture_families"),
};

const ownershipCounts = {};
for (const manifestCategory of Object.values(categories))
  for (const record of manifestCategory.records)
    ownershipCounts[record.ownership] = (ownershipCounts[record.ownership] ?? 0) + 1;
const payload = {
  schema_revision: "starclock.swarm-disaster-content-manifest.v1",
  goal_id: "swarm-disaster-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  },
  profile: "swarm-disaster-v1",
  ownership_policy: {
    SwarmDisaster: "mode-owned row or mode-specific representation",
    Shared: "stable shared identity with explicit ChessRogue reachability",
    EvidenceOnly: "excluded row retained only to prove the content boundary",
    fail_closed:
      "RogueDLC is a shared framework; only explicit ChessRogue selectors, non-MapRepo160 references or inherited shared-pool proofs grant reachability",
  },
  denominator_policy: {
    source_obligation:
      "counts freeze exact source and inherited stable-ID obligations; normalized child rows may expand but cannot remove an obligation",
    topology_edges:
      "released chessboard configs expose nodes/coordinates/events but no edge list; deterministic edge semantics are a B4 ProjectPolicy contract, not invented ExactStructured rows",
    countdown_and_disarray:
      "constants and all BossDecay rows are exact obligations; transition ordering and clamp/carry semantics remain B4 policy fields until stronger evidence is bound",
    encounters:
      "room bindings and displayed boss identities are the frozen source obligations; P2-B5 expands StageConfig waves/enemy slots with exact-once parent references",
    fixture_families:
      "the 23 family IDs are a non-shrinking minimum; multiple fixtures may satisfy one family",
  },
  exclusions: {
    mode_prefixes: ["RogueEndless", "RogueMagic", "RogueNous", "RoguePersona", "RogueTourn"],
    sub_modes: ["ChessRogueNous", "MagicRogue", "TournRogue", "CosmosRogue"],
    gold_checkpoint: {
      commit: "457d05f0e3a7b6fe3abb7e8f142f96fa271f5ecd",
      manifest_sha256: "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
      records: 7913,
    },
    story_account_rows: storyAccountExclusions,
    story_account_count: storyAccountExclusions.length,
    unreferenced_topology_rows: unreferencedTopology,
    unreferenced_topology_count: unreferencedTopology.length,
  },
  counts: {
    categories: Object.keys(categories).length,
    records: Object.values(categories).reduce((sum, value) => sum + value.count, 0),
    ownership: Object.fromEntries(Object.entries(ownershipCounts).sort(([a], [b]) =>
      compare(a, b))),
  },
  counter_groups: counterGroups,
  categories,
};

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = fs.readFileSync(output, "utf8");
  if (committed !== encoded)
    throw new Error("Swarm Disaster content manifest has generated drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded, "utf8");
}
console.log(
  `Swarm Disaster content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records} records in ${payload.counts.categories} categories.`,
);

function digest(value) {
  return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
