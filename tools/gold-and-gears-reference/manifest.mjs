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
  "content-manifests/gold-and-gears-v1/content-manifest.json",
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
function inheritedRecords(file, ownership = "Shared") {
  const relative = `content-reference/standard-universe-v1/${file}`;
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8")).map((row) => ({
    id: row.id,
    source: relative,
    evidence_sha256: digest(row),
    evidence_quality: "ExactStructured",
    ownership,
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
  "One project profile for the Version 4.4 ChessRogueNous activity boundary.",
  [{
    id: "gold-and-gears-v1",
    source: "policy/goal08-foundation.json",
    evidence_sha256: digest({
      goal_id: "gold-and-gears-reference-v1",
      sub_mode: "ChessRogueNous",
      game_version: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "GoldAndGears",
    reachability: "Direct",
  }],
);
category(
  "entry_points",
  "The resident activity, DLC entrance and common title rows whose sub-mode is ChessRogueNous.",
  [
    ...activityEntries.filter(({ row }) => row.SubMode === "ChessRogueNous")
      .map((entry) => sourceRecord(entry, `activity:${entry.row.ActivityID}`, {
        ownership: "GoldAndGears", reachability: "Direct",
      })),
    ...entrances.filter(({ row }) => row.SubType === "ChessRogueNous")
      .map((entry) => sourceRecord(entry, `entrance:${entry.row.ID}`, {
        ownership: "GoldAndGears", reachability: "Direct",
      })),
    ...modeTitles.filter(({ row }) => row.SubMode === "ChessRogueNous")
      .map((entry) => sourceRecord(entry, `title:${entry.row.SubMode}`, {
        ownership: "GoldAndGears", reachability: "Direct",
      })),
  ],
);

const areas = sourceRows("ExcelOutput/RogueDLCArea.json")
  .filter(({ row }) => row.SubType === "ChessRogueNous");
const guideAreas = category(
  "guide_areas",
  "ChessRogueNous area rows outside the five Formal difficulties.",
  areas.filter(({ row }) => row.AreaGroupID !== "Formal")
    .map((entry) => sourceRecord(entry, entry.row.AreaID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
const formalDifficulties = category(
  "formal_difficulties",
  "The five ChessRogueNous Formal area rows, Difficulty_1 through Difficulty_5.",
  areas.filter(({ row }) => row.AreaGroupID === "Formal")
    .map((entry) => sourceRecord(entry, entry.row.AreaID, {
      ownership: "GoldAndGears", reachability: "Direct",
      difficulty: entry.row.Difficulty,
    })),
);
const difficultyIds = new Set(areas.flatMap(({ row }) => row.DifficultyID));
category(
  "difficulty_segments",
  "Shared RogueDLCDifficulty rows directly referenced by ChessRogueNous areas.",
  sourceRows("ExcelOutput/RogueDLCDifficulty.json")
    .filter(({ row }) => difficultyIds.has(row.DifficultyID))
    .map((entry) => sourceRecord(entry, entry.row.DifficultyID, {
      ownership: "Shared", reachability: "Referenced",
    })),
);
category(
  "conundrum_levels",
  "All released RogueNous attribute and additional difficulty definitions.",
  sourceRows("ExcelOutput/RogueNousDifficultyLevel.json")
    .map((entry) => sourceRecord(entry, entry.row.DifficultyID, {
      ownership: "GoldAndGears", reachability: "Direct",
      kind: entry.row.DifficultyType,
    })),
);
const layerIds = new Set(areas.flatMap(({ row }) => row.LayerIDList));
category(
  "planes",
  "Shared DLC layer rows directly referenced by ChessRogueNous areas.",
  sourceRows("ExcelOutput/RogueDLCLayer.json")
    .filter(({ row }) => layerIds.has(row.LayerID))
    .map((entry) => sourceRecord(entry, entry.row.LayerID, {
      ownership: "Shared", reachability: "Referenced",
    })),
);

const chessboards = sourceRows("ExcelOutput/RogueDLCChessBoard.json")
  .filter(({ row }) => row.ChessBoardConfiguration.includes("RogueNous"));
category(
  "chessboards",
  "Shared DLC chessboard rows whose configuration path is in MapRepo160/RogueNous.",
  chessboards.map((entry) => sourceRecord(entry, entry.row.ChessBoardID, {
    ownership: "GoldAndGears", reachability: "Direct",
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
        ownership: "GoldAndGears", reachability: "Direct",
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
      ownership: "GoldAndGears", reachability: "Direct",
      chessboard_id: String(board.row.ChessBoardID),
    }));
  for (const [eventId, event] of Object.entries(config.RogueChestEventMap ?? {}))
    boardEvents.push(nestedRecord(relative, `RogueChestEventMap/${eventId}`, event,
      `${board.row.ChessBoardID}:${eventId}`, {
        ownership: "GoldAndGears", reachability: "Direct",
        chessboard_id: String(board.row.ChessBoardID),
      }));
  for (const rule of config.RogueBlockCreateGroupList ?? []) {
    blockCreateRules.push(nestedRecord(
      relative,
      `RogueBlockCreateGroupList/${rule.BlockCreateID}`,
      rule,
      `${board.row.ChessBoardID}:${rule.BlockCreateID}`,
      {
        ownership: "GoldAndGears", reachability: "Direct",
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
  "Distinct authored PosX columns within every reachable RogueNous chessboard.",
  columns,
);
category(
  "map_nodes",
  "Every authored RogueChestGridItemMap entry in reachable RogueNous chessboards.",
  nodes,
);
category(
  "map_events",
  "Every authored RogueChestEventMap entry in reachable RogueNous chessboards.",
  boardEvents,
);
category(
  "block_create_rules",
  "Every authored RogueBlockCreateGroupList row in reachable RogueNous chessboards.",
  blockCreateRules,
);
category(
  "domains",
  "Distinct BlockType values reachable from RogueNous nodes or block-create rules.",
  [...domainEvidence.entries()].map(([domain, evidence]) =>
    nestedRecord(evidence.file, evidence.key, evidence.value, domain, {
      ownership: "Shared", reachability: "Referenced",
    })),
);
category(
  "beacons",
  "Distinct nonzero MarkCreateRandomList TypeID values reachable from RogueNous boards.",
  [...beaconEvidence.entries()].map(([beacon, evidence]) =>
    nestedRecord(evidence.file, evidence.key, evidence.value, beacon, {
      ownership: "Shared", reachability: "Referenced",
    })),
);

const roomBindings = sourceRows("ExcelOutput/RogueNousRoom.json");
category(
  "room_bindings",
  "All released ChessRogueNous room-to-section membership rows.",
  roomBindings.map((entry) => sourceRecord(entry, entry.row.RogueRoomID, {
    ownership: "GoldAndGears", reachability: "Direct",
  })),
);
const roomIds = new Set(roomBindings.map(({ row }) => row.RogueRoomID));
category(
  "adventure_outcomes",
  "Shared abstract Adventure definitions whose room is reachable in RogueNous.",
  sourceRows("ExcelOutput/RogueDLCAdventureRoom.json")
    .filter(({ row }) => roomIds.has(row.RoomID))
    .map((entry) => sourceRecord(entry, entry.row.RoomID, {
      ownership: "Shared", reachability: "Referenced",
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
  "Distinct displayed boss/enemy identities referenced by ChessRogueNous areas.",
  [...bossChoices.entries()].map(([monsterId, { entry, level }]) =>
    sourceRecord(entry, monsterId, {
      ownership: "Shared", reachability: "Referenced", display_level: level,
    })),
);

category(
  "cognition_ranges",
  "All released RogueNous area cognition ranges.",
  sourceRows("ExcelOutput/RogueNousValueAreaLimit.json")
    .map((entry) => sourceRecord(entry, entry.row.AreaID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
category(
  "secret_conditions",
  "All RogueNous sub-story rows retained only for mechanical area/cognition thresholds.",
  sourceRows("ExcelOutput/RogueNousSubStory.json")
    .map((entry) => sourceRecord(entry, entry.row.StoryID, {
      ownership: "GoldAndGears", reachability: "Direct",
      retained_fields: ["RequireArea", "MinNousValue", "MaxNousValue", "TriggerCondition"],
    })),
);
category(
  "mode_constants",
  "All released common RogueNous constants; client-only presentation constants are excluded.",
  sourceRows("ExcelOutput/RogueNousConstValueCommon.json")
    .map((entry) => sourceRecord(entry, entry.row.ConstValueName, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);

category(
  "dice_categories",
  "All released RogueNous dice branch tags.",
  sourceRows("ExcelOutput/RogueNousDiceBranchTag.json")
    .map((entry) => sourceRecord(entry, entry.row.TagID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
category(
  "dice_definitions",
  "All released RogueNous custom dice branches.",
  sourceRows("ExcelOutput/RogueNousDiceBranch.json")
    .map((entry) => sourceRecord(entry, entry.row.BranchID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
category(
  "dice_path_values",
  "Every dice-branch and selected-Path value binding.",
  sourceRows("ExcelOutput/RogueNousDiceBranchValue.json")
    .map((entry) => sourceRecord(
      entry,
      `${entry.row.BranchID}:${entry.row.AeonID}`,
      { ownership: "GoldAndGears", reachability: "Direct" },
    )),
);
category(
  "dice_slots",
  "All six released RogueNous dice slots.",
  sourceRows("ExcelOutput/RogueNousDiceSlot.json")
    .map((entry) => sourceRecord(entry, entry.row.SlotID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
const diceFaces = sourceRows("ExcelOutput/RogueNousDiceSurface.json");
category(
  "dice_faces",
  "All released RogueNous dice surfaces.",
  diceFaces.map((entry) => sourceRecord(entry, entry.row.SurfaceID, {
    ownership: "GoldAndGears", reachability: "Direct",
  })),
);
category(
  "dice_face_tags",
  "All released RogueNous surface-tag definitions.",
  sourceRows("ExcelOutput/RogueNousSurfaceTag.json")
    .map((entry) => sourceRecord(entry, entry.row.TagID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);
category(
  "knowledge_bindings",
  "Dice surfaces tagged Mark, the released structured locator for Knowledge interactions.",
  diceFaces.filter(({ row }) => (row.TagList ?? []).includes("Mark"))
    .map((entry) => sourceRecord(entry, entry.row.SurfaceID, {
      ownership: "GoldAndGears", reachability: "Direct",
      binding: "Mark",
    })),
);
category(
  "neural_network_nodes",
  "All released RogueNous talent nodes; reward-only fields remain excluded downstream.",
  sourceRows("ExcelOutput/RogueNousTalent.json")
    .map((entry) => sourceRecord(entry, entry.row.TalentID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);

category(
  "paths",
  "All nine Goal 03 stable Path identities are selectable by RogueNousAeon.",
  inheritedRecords("paths.json"),
);
category(
  "resonances",
  "All 36 Goal 03 stable Resonance/Formation identities are shared and reachable.",
  inheritedRecords("resonances.json"),
);
category(
  "blessings",
  "All 162 Goal 03 stable Blessing identities are shared and reachable.",
  inheritedRecords("blessings.json"),
);
category(
  "blessing_levels",
  "Both authored levels of every reachable shared Blessing.",
  inheritedRecords("blessing-levels.json"),
);

const aeons = sourceRows("ExcelOutput/RogueNousAeon.json");
category(
  "path_boosts",
  "One RogueNous mode-entry Path boost for each selectable Aeon.",
  aeons.map((entry) => sourceRecord(entry, entry.row.EffectParam1[0], {
    ownership: "GoldAndGears", reachability: "Direct",
    aeon_id: String(entry.row.AeonID),
  })),
);
const buffGroups = sourceRows("ExcelOutput/RogueBuffGroup.json");
const groupById = new Map(buffGroups.map((entry) => [entry.row.GMLOGNJAIGI, entry]));
const extrapolationRecords = [];
for (const aeon of aeons)
  for (const groupId of [
    aeon.row.BattleEventBuffGroup,
    aeon.row.BattleEventEnhanceBuffGroup,
  ]) {
    const group = groupById.get(groupId);
    if (!group) throw new Error(`missing RogueBuffGroup ${groupId}`);
    for (const buffId of group.row.HECJCAMDGNO)
      extrapolationRecords.push(sourceRecord(group, buffId, {
        ownership: "GoldAndGears", reachability: "Referenced",
        aeon_id: String(aeon.row.AeonID),
        buff_group_id: String(groupId),
      }));
  }
category(
  "resonance_extrapolations",
  "Every buff in RogueNous normal/enhanced BattleEvent groups for all nine Paths.",
  extrapolationRecords,
);
const interplayRecords = [];
for (const cross of sourceRows("ExcelOutput/RogueNousAeonCross.json")) {
  const group = groupById.get(cross.row.BuffGroup);
  if (!group || group.row.HECJCAMDGNO.length !== 1)
    throw new Error(`invalid RogueNous interplay group ${cross.row.BuffGroup}`);
  interplayRecords.push(sourceRecord(cross, group.row.HECJCAMDGNO[0], {
    ownership: "GoldAndGears", reachability: "Direct",
    main_aeon_id: String(cross.row.MainAeonID),
    sub_aeon_id: String(cross.row.SubAeonID),
    buff_group_id: String(cross.row.BuffGroup),
  }));
}
category(
  "resonance_interplays",
  "All released RogueNous main/sub-Aeon interplay bindings.",
  interplayRecords,
);

category(
  "trailblaze_bonuses",
  "Gold and Gears-owned RogueBonus IDs 201 through 205.",
  sourceRows("ExcelOutput/RogueBonus.json")
    .filter(({ row }) => row.BonusID >= 201 && row.BonusID <= 205)
    .map((entry) => sourceRecord(entry, entry.row.BonusID, {
      ownership: "GoldAndGears", reachability: "Direct",
    })),
);

const curioHandbooks = sourceRows("ExcelOutput/RogueHandbookMiracle.json")
  .filter(({ row }) => row.MiracleTypeList.includes(160));
const curioHandbookIds = new Set(curioHandbooks.map(({ row }) =>
  row.MiracleHandbookID));
const curioCopies = sourceRows("ExcelOutput/RogueMiracle.json")
  .filter(({ row }) => row.MiracleID >= 3000
    && row.MiracleID < 4000
    && curioHandbookIds.has(row.UnlockHandbookMiracleID));
const copyByHandbook = new Map(curioCopies.map((entry) => [
  entry.row.UnlockHandbookMiracleID,
  entry,
]));
category(
  "curios",
  "All handbook Curios whose mode membership includes type 160 (Gold and Gears).",
  curioHandbooks.map((entry) => {
    const copy = copyByHandbook.get(entry.row.MiracleHandbookID);
    if (!copy) throw new Error(`missing Gold Curio copy ${entry.row.MiracleHandbookID}`);
    return sourceRecord(entry, entry.row.MiracleHandbookID, {
      ownership: entry.row.MiracleTypeList.includes(100) ? "Shared" : "GoldAndGears",
      reachability: "Direct",
      mode_copy_id: String(copy.row.MiracleID),
    });
  }),
);
category(
  "curio_states",
  "Exactly one 3000-series Gold and Gears mode copy for every reachable Curio.",
  curioCopies.map((entry) => sourceRecord(entry, entry.row.MiracleID, {
    ownership: "GoldAndGears", reachability: "Referenced",
    handbook_id: String(entry.row.UnlockHandbookMiracleID),
  })),
);

const occurrenceHandbooks = sourceRows("ExcelOutput/RogueHandBookEvent.json")
  .filter(({ row }) => row.EventTypeList.includes(160));
category(
  "occurrences",
  "All handbook Occurrences whose mode membership includes type 160.",
  occurrenceHandbooks.map((entry) => sourceRecord(entry, entry.row.EventHandbookID, {
    ownership: entry.row.EventTypeList.includes(100) ? "Shared" : "GoldAndGears",
    reachability: "Direct",
  })),
);
const occurrenceVariants = new Map();
for (const handbook of occurrenceHandbooks)
  for (const progress of handbook.row.UnlockNPCProgressIDList ?? []) {
    const progressId = progress.FDOELDMEBPE;
    if (progressId >= 300000 && progressId < 400000) {
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
  "Distinct 300000-series NPC progress variants reachable from type-160 handbooks.",
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
        ownership: "GoldAndGears",
        reachability: "Referenced",
        handbook_ids: [...handbookIds].map(String).sort(compare),
      },
    )),
);

category(
  "shared_services",
  "The 15 non-bonus service definitions inherited from the Goal 03 reference pack.",
  inheritedRecords("services.json")
    .filter(({ id }) => !id.includes(".trailblaze-bonus.")),
);

const fixtureFamilies = [
  "profile-entry",
  "topology-generation",
  "topology-event-order",
  "cognition-lifecycle",
  "secret-threshold",
  "custom-dice-passive",
  "dice-face-targeting",
  "dice-reroll-and-cheat",
  "knowledge-lifecycle",
  "neural-network-effect",
  "conundrum-stats",
  "conundrum-auxiliary",
  "path-boost",
  "resonance-extrapolation",
  "curio-lifecycle",
  "occurrence-choice",
  "service-and-adventure",
  "encounter-selection",
];
category(
  "semantic_fixture_families",
  "Minimum distinct semantic families; B4 freezes fixture shape and later batches may add cases without shrinking these obligations.",
  fixtureFamilies.map((id) => ({
    id,
    source: "docs/goals/08-gold-and-gears-reference-data.md",
    evidence_sha256: digest({ id, goal_id: "gold-and-gears-reference-v1" }),
    evidence_quality: "ProjectPolicy",
    ownership: "GoldAndGears",
    reachability: "Direct",
  })),
);

const excludedTables = [
  "ExcelOutput/RogueNousEndGameReward.json",
  "ExcelOutput/RogueNousMainStory.json",
  "ExcelOutput/RogueNousMissionReward.json",
  "ExcelOutput/RogueNousStoryDisplay.json",
  "ExcelOutput/RogueNousStoryReward.json",
];
const exclusions = excludedTables.flatMap((file) =>
  sourceRows(file).map((entry) => sourceRecord(entry, `${path.basename(file)}:${entry.index}`, {
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: "story, mission, end-game or account reward presentation",
  })));

const group = (...categoryIds) => ({
  categories: categoryIds,
  required: categoryIds.reduce((sum, id) => sum + categories[id].count, 0),
});
const counterGroups = {
  profiles_entries_bonuses:
    group("profiles", "entry_points", "trailblaze_bonuses"),
  difficulties_and_conundrum_unlock:
    group("formal_difficulties", "difficulty_segments", "conundrum_levels"),
  topology:
    group(
      "guide_areas", "planes", "chessboards", "map_columns", "map_nodes",
      "map_events", "block_create_rules", "room_bindings", "domains", "beacons",
      "boss_choices",
    ),
  cognition_and_secrets:
    group("cognition_ranges", "secret_conditions", "mode_constants"),
  custom_dice:
    group("dice_categories", "dice_definitions", "dice_path_values"),
  dice_slots_faces_tags:
    group("dice_slots", "dice_faces", "dice_face_tags"),
  knowledge_rules: group("knowledge_bindings"),
  neural_network: group("neural_network_nodes"),
  conundrum: group("conundrum_levels"),
  paths_and_resonance:
    group(
      "paths", "resonances", "path_boosts",
      "resonance_extrapolations", "resonance_interplays",
    ),
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
  schema_revision: "starclock.gold-and-gears-content-manifest.v1",
  goal_id: "gold-and-gears-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  },
  profile: "gold-and-gears-v1",
  ownership_policy: {
    GoldAndGears: "mode-owned row or mode-specific representation",
    Shared: "stable shared identity with explicit ChessRogueNous reachability",
    EvidenceOnly: "excluded row retained only to prove the content boundary",
    fail_closed:
      "RogueDLC is a shared framework; only explicit ChessRogueNous selectors, references or inherited shared-pool proofs grant reachability",
  },
  denominator_policy: {
    source_obligation:
      "counts freeze exact source and inherited stable-ID obligations; normalized child rows may expand but cannot remove an obligation",
    topology_edges:
      "released chessboard configs expose nodes/coordinates/events but no edge list; deterministic edge semantics are a B4 ProjectPolicy contract, not invented ExactStructured rows",
    encounters:
      "room bindings and displayed boss identities are the frozen source obligations; P2-B5 expands StageConfig waves/enemy slots with exact-once parent references",
    fixture_families:
      "the 18 family IDs are a non-shrinking minimum; multiple fixtures may satisfy one family",
  },
  exclusions: {
    mode_prefixes: ["RogueEndless", "RogueMagic", "RoguePersona", "RogueTourn"],
    sub_modes: ["ChessRogue", "MagicRogue", "TournRogue", "CosmosRogue"],
    story_account_rows: exclusions,
    story_account_count: exclusions.length,
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
    throw new Error("Gold and Gears content manifest has generated drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded, "utf8");
}
console.log(
  `Gold and Gears content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records} records in ${payload.counts.categories} categories.`,
);

function digest(value) {
  return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
