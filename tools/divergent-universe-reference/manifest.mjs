#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root") ?? ".");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
const output = path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
);
const inventoryPath = path.join(
  root,
  "content-manifests/divergent-universe-v1/source-inventory.json",
);
const categories = {};

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

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

function derivedRecord(entry, id, value, fields = {}) {
  return {
    id: String(id),
    source: `${entry.file}#${entry.index}`,
    evidence_sha256: digest(value),
    evidence_quality: "ExactStructured",
    ...fields,
  };
}

function category(id, membershipBasis, records) {
  const ordered = [...records].sort((left, right) => compare(left.id, right.id));
  if (new Set(ordered.map((record) => record.id)).size !== ordered.length)
    throw new Error(`duplicate IDs in category ${id}`);
  categories[id] = {
    id,
    membership_basis: membershipBasis,
    count: ordered.length,
    records: ordered,
  };
  return ordered;
}

function directRows(file, key, options = {}) {
  const {
    id = (entry) => entry.row[key],
    ownership = "DivergentUniverse",
    reachability = "DirectModeTable",
    fields = () => ({}),
  } = options;
  return sourceRows(file).map((entry) =>
    sourceRecord(entry, id(entry), {
      ownership,
      reachability,
      ...fields(entry),
    }));
}

function selectedRows(file, predicate, key, options = {}) {
  return directRows(file, key, options).filter((record) => {
    const [, locator] = record.source.split("#");
    return predicate(sourceRows(file)[Number(locator)].row);
  });
}

function unique(values) {
  return [...new Set(values)];
}

function uniqueEntries(entries, key) {
  const selected = new Map();
  for (const entry of entries) {
    const id = String(entry.row[key]);
    if (!selected.has(id)) selected.set(id, entry);
  }
  return [...selected.values()];
}

const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const activityRows = sourceRows("ExcelOutput/RogueActivityResidentConfig.json");
const titleRows = sourceRows("ExcelOutput/RogueCommonModeTitle.json");
const moduleRows = sourceRows("ExcelOutput/RogueTournModule.json");
const currentActivity = activityRows.filter(
  ({ row }) => row.SubMode === "TournRogue" && row.ActivityModuleID === 6002201,
);
if (currentActivity.length !== 1)
  throw new Error("expected exactly one TournRogue activity at module 6002201");

category(
  "profiles",
  "One project profile fixed to the released Version 4.4 TournRogue boundary.",
  [{
    id: "divergent-universe-v1",
    source: "content-manifests/divergent-universe-v1/foundation.json",
    evidence_sha256: digest({
      goal_id: "divergent-universe-reference-v1",
      sub_mode: "TournRogue",
      activity_module_id: 6002201,
      game_version: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "DivergentUniverse",
    reachability: "Direct",
  }],
);
category(
  "entry_points",
  "Resident activity and common-title rows with the exact TournRogue selector.",
  [
    ...currentActivity.map((entry) =>
      sourceRecord(entry, `activity:${entry.row.ActivityID}`, {
        ownership: "DivergentUniverse",
        reachability: "ExplicitModeSelector",
      })),
    ...titleRows.filter(({ row }) => row.SubMode === "TournRogue").map((entry) =>
      sourceRecord(entry, `title:${entry.row.SubMode}`, {
        ownership: "DivergentUniverse",
        reachability: "ExplicitModeSelector",
      })),
  ],
);
category(
  "enabled_modules",
  "The module row transitively selected by the resident TournRogue ActivityModuleID.",
  moduleRows.filter(({ row }) => row.ActivityModuleID === 6002201).map((entry) =>
    sourceRecord(entry, entry.row.ActivityModuleID, {
      ownership: "DivergentUniverse",
      reachability: "TransitiveReference",
      main_tourn_id: entry.row.MainTournID,
      sub_tourn_id: entry.row.SubTournID,
    })),
);

const finishRows = sourceRows("ExcelOutput/RogueTournFinishway.json")
  .filter(({ row }) => JSON.stringify(row).includes("Cond_InRogueTournMode(3)"));
category(
  "finish_conditions",
  "Rows whose released condition program explicitly tests RogueTourn mode 3.",
  finishRows.map((entry) =>
    sourceRecord(entry, entry.row.ID, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
    })),
);

const areaGroupRows = sourceRows("ExcelOutput/RogueTournAreaGroupByTourn.json")
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3");
category(
  "area_groups",
  "Area-group selector row whose TournMode is exactly Tourn3.",
  areaGroupRows.map((entry) =>
    sourceRecord(entry, `Tourn3:${entry.row.JFMBIOOCPIL}`, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
    })),
);
const areaRows = sourceRows("ExcelOutput/RogueTournArea.json")
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3");
category(
  "areas",
  "Every area row whose released TournMode selector is exactly Tourn3.",
  areaRows.map((entry) =>
    sourceRecord(entry, entry.row.BEOFPCAACEP, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
      area_type: entry.row.PJGJLMIODBD,
    })),
);
const difficultyIds = new Set(
  areaRows.flatMap(({ row }) => row.EODCEHDOAEB ?? []).map(String),
);
category(
  "difficulties",
  "Difficulty rows referenced by at least one Tourn3 area.",
  sourceRows("ExcelOutput/RogueTournDifficulty.json")
    .filter(({ row }) => difficultyIds.has(String(row.DifficultyID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.DifficultyID, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
const layerIds = new Set(
  areaRows.flatMap(({ row }) => row.GLNDIILFKBN ?? []).map(String),
);
category(
  "layers",
  "Layer rows referenced by at least one Tourn3 area.",
  sourceRows("ExcelOutput/RogueTournLayer.json")
    .filter(({ row }) => layerIds.has(String(row.LayerID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.LayerID, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
category(
  "layer_rooms",
  "Released layer-room rows matching a Tourn3-referenced LayerID; the fixed snapshot contains none.",
  sourceRows("ExcelOutput/RogueTournLayerRoom.json")
    .filter(({ row }) => layerIds.has(String(row.LayerID)))
    .map((entry) =>
      sourceRecord(entry, `${entry.row.LayerID}:${entry.row.RoomIndex}`, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
const roomCandidates = sourceRows("ExcelOutput/RogueTournRoom.json")
  .filter(({ row }) => row.TournMode === "Tourn2");
category(
  "room_reuse_candidates",
  "Conservative Tourn2 room source obligations because the snapshot has no Tourn3 room rows; each remains unproven until P1-B1 resolves stage/config reachability.",
  roomCandidates.map((entry) =>
    sourceRecord(entry, entry.row.RogueRoomID, {
      ownership: "SharedCandidate",
      reachability: "PendingStageClosure",
      room_type: entry.row.RogueRoomType,
    })),
);
const candidateRoomTypes = unique(
  roomCandidates.map(({ row }) => row.RogueRoomType),
);
category(
  "room_types",
  "Distinct room types represented by the conservative room-reuse candidate set.",
  candidateRoomTypes.map((roomType) => ({
    id: roomType,
    source: "ExcelOutput/RogueTournRoom.json",
    evidence_sha256: digest(roomCandidates
      .filter(({ row }) => row.RogueRoomType === roomType)
      .map(({ row }) => row.RogueRoomID)),
    evidence_quality: "ExactStructured",
    ownership: "SharedCandidate",
    reachability: "PendingStageClosure",
  })),
);

category(
  "astronomical_divisions",
  "All direct Astronomical Division level rows in the mode-owned table.",
  directRows("ExcelOutput/RogueTournDivision.json", "DivisionLevel"),
);
category(
  "astronomical_division_effects",
  "All direct Astronomical Division effect rows in the mode-owned table.",
  directRows("ExcelOutput/RogueTournDivisionEffect.json", "DivisionLevel"),
);

category(
  "arithmetic_mapping_avatars",
  "Every avatar-to-temporary-avatar mapping in the direct mode-owned table.",
  directRows("ExcelOutput/RogueTournAvatar.json", "AvatarID"),
);
category(
  "arithmetic_mapping_build_refs",
  "Every avatar eligible for a temporary build reference.",
  directRows("ExcelOutput/RogueTournBuildRefAvatar.json", "AvatarID"),
);
category(
  "arithmetic_mapping_roles",
  "Every avatar-to-temporary-role buff mapping.",
  directRows("ExcelOutput/RogueTournRole.json", "AvatarID"),
);

const formulaRows = sourceRows("ExcelOutput/RogueTournFormula.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category(
  "equations",
  "Every Equation row whose released TournMode selector is exactly Tourn3.",
  formulaRows.map((entry) =>
    sourceRecord(entry, entry.row.FormulaID, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
      formula_category: entry.row.FormulaCategory,
    })),
);
const formulaDisplayIds = new Set(
  formulaRows.map(({ row }) => String(row.FormulaDisplayID)),
);
category(
  "equation_displays",
  "Formula display rows referenced by a Tourn3 Equation.",
  sourceRows("ExcelOutput/RogueTournFormulaDisplay.json")
    .filter(({ row }) => formulaDisplayIds.has(String(row.FormulaDisplayID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.FormulaDisplayID, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
category(
  "equation_randomizers",
  "Conservative direct mode-table obligations for Equation random offer programs.",
  directRows("ExcelOutput/RogueTournFormulaRandom.json", "RandomID"),
);
category(
  "equation_keywords",
  "All mode-owned Equation/Blessing keyword definitions.",
  directRows("ExcelOutput/RogueTournKeyword.json", "KeywordID"),
);
category(
  "equation_keyword_params",
  "All mode-owned Equation/Blessing keyword parameter rows.",
  directRows("ExcelOutput/RogueTournKeywordParam.json", "KeywordID"),
);

const activeBuffTypeRow = sourceRows("ExcelOutput/RogueTournUseBuffType.json")
  .find(({ row }) => row.TournMode === "Tourn3");
if (!activeBuffTypeRow) throw new Error("missing Tourn3 active Blessing type row");
const activeBuffTypes = new Set(
  activeBuffTypeRow.row.UseBuffTypeList.map(String),
);
category(
  "blessing_paths",
  "Blessing Path/type definitions selected by the Tourn3 UseBuffTypeList.",
  sourceRows("ExcelOutput/RogueTournBuffType.json")
    .filter(({ row }) => activeBuffTypes.has(String(row.RogueBuffType)))
    .map((entry) =>
      sourceRecord(entry, entry.row.RogueBuffType, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
const blessingLevelRows = sourceRows("ExcelOutput/RogueTournBuff.json")
  .filter(({ row }) => activeBuffTypes.has(String(row.RogueBuffType)));
category(
  "blessings",
  "Distinct Blessings whose type is present in the Tourn3 active type selector.",
  uniqueEntries(blessingLevelRows, "MazeBuffID").map((entry) =>
    sourceRecord(entry, entry.row.MazeBuffID, {
      ownership: "DivergentUniverse",
      reachability: "TransitiveReference",
      blessing_type: entry.row.RogueBuffType,
    })),
);
category(
  "blessing_levels",
  "Every level row whose Blessing type is present in the Tourn3 active type selector.",
  blessingLevelRows.map((entry) =>
    sourceRecord(entry, `${entry.row.MazeBuffID}:${entry.row.MazeBuffLevel}`, {
      ownership: "DivergentUniverse",
      reachability: "TransitiveReference",
      blessing_type: entry.row.RogueBuffType,
    })),
);
category(
  "blessing_groups",
  "Every Blessing group row whose released TournMode selector is exactly Tourn3.",
  sourceRows("ExcelOutput/RogueTournBuffGroup.json")
    .filter(({ row }) => row.TournMode === "Tourn3")
    .map((entry) =>
      sourceRecord(entry, entry.row.RogueBuffGroupID, {
        ownership: "DivergentUniverse",
        reachability: "ExplicitModeSelector",
      })),
);

const miracleRows = sourceRows("ExcelOutput/RogueTournMiracle.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category(
  "curio_states",
  "Every Curio/Weighted Curio state row whose released TournMode is exactly Tourn3.",
  miracleRows.map((entry) =>
    sourceRecord(entry, entry.row.MiracleID, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
      handbook_id: entry.row.HandbookMiracleID,
    })),
);
const handbookMiracleIds = new Set(
  miracleRows.map(({ row }) => String(row.HandbookMiracleID)),
);
category(
  "curios",
  "Distinct handbook Curio identities transitively referenced by Tourn3 state rows.",
  sourceRows("ExcelOutput/RogueTournHandbookMiracle.json")
    .filter(({ row }) => handbookMiracleIds.has(String(row.HandbookMiracleID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.HandbookMiracleID, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
category(
  "curio_groups",
  "Conservative direct mode-table obligations for Curio offer groups.",
  directRows("ExcelOutput/RogueTournMiracleGroup.json", "RogueMiracleGroupID"),
);

const hexRows = sourceRows("ExcelOutput/RogueTournHex.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category(
  "grand_miracles",
  "Every Grand Miracle/Hex row whose released TournMode is exactly Tourn3.",
  hexRows.map((entry) =>
    sourceRecord(entry, entry.row.HexID, {
      ownership: "DivergentUniverse",
      reachability: "ExplicitModeSelector",
    })),
);
const hexMiracleIds = new Set(
  sourceRows("ExcelOutput/RogueTournHexAvatarBaseType.json")
    .map(({ row }) => String(row.MiracleID)),
);
category(
  "grand_miracle_eligibility",
  "Every mode-owned character Path/element eligibility row that references a Grand Miracle identity.",
  sourceRows("ExcelOutput/RogueTournHexAvatarBaseType.json")
    .filter(({ row }) => hexMiracleIds.has(String(row.MiracleID)))
    .map((entry) =>
      sourceRecord(
        entry,
        `${entry.row.MiracleID}:${entry.row.AvatarType}:${entry.row.AvatarDamageType}`,
        {
          ownership: "DivergentUniverse",
          reachability: "TransitiveReference",
        },
      )),
);

category(
  "titan_types",
  "All direct Golden Blood/Titan type rows.",
  directRows("ExcelOutput/RogueTournTitanType.json", "RogueTitanType"),
);
category(
  "titan_bless_levels",
  "All direct Golden Blood's Boon level rows.",
  directRows("ExcelOutput/RogueTournTitanBless.json", "TitanBlessID", {
    id: (entry) => `${entry.row.TitanBlessID}:${entry.row.TitanBlessLevel}`,
  }),
);
category(
  "titan_talent_levels",
  "All direct Titan talent level rows.",
  directRows("ExcelOutput/RogueTournTitanTalent.json", "ID", {
    id: (entry) => `${entry.row.ID}:${entry.row.Level}`,
  }),
);

category(
  "workbenches",
  "All direct mode-owned workbench definitions.",
  directRows("ExcelOutput/RogueTournWorkbench.json", "WorkbenchID"),
);
category(
  "workbench_functions",
  "All direct mode-owned workbench operations.",
  directRows("ExcelOutput/RogueTournWorkbenchFunc.json", "FuncID"),
);
category(
  "gamble_groups",
  "All direct mode-owned gamble offer groups.",
  directRows("ExcelOutput/RogueTournGambleGroup.json", "GambleGroupID"),
);
category(
  "gamble_units",
  "All direct mode-owned gamble outcomes.",
  directRows("ExcelOutput/RogueTournGambleUnit.json", "GambleUnitID"),
);
category(
  "curse_chests",
  "All direct mode-owned curse-chest service definitions.",
  directRows("ExcelOutput/RogueTournCurseChest.json", "ChestID"),
);

category(
  "permanent_talents",
  "All direct mode-owned Inspiration Circuit talent rows.",
  directRows("ExcelOutput/RogueTournPermanentTalent.json", "TalentID"),
);
category(
  "unlocks",
  "All direct mode-owned unlock-to-finish-condition bindings.",
  directRows("ExcelOutput/RogueTournUnlock.json", "RogueUnlockID"),
);
category(
  "common_constants",
  "All simulation-candidate common constants; client-only constants are excluded.",
  directRows("ExcelOutput/RogueTournConstCommon.json", "ConstValueName"),
);
category(
  "weekly_modifiers",
  "Conservative direct mode-table obligations for weekly/cyclical mechanical modifiers.",
  directRows("ExcelOutput/RogueTournWeeklyChallenge.json", "ChallengeID"),
);
category(
  "room_marks",
  "All direct mode-owned room-mark transition records.",
  directRows("ExcelOutput/RogueTournRoomMark.json", "ICIDICKIDCB", {
    id: (entry) => `${entry.index}:${entry.row.ICIDICKIDCB ?? "none"}`,
  }),
);

const handbookRows = sourceRows("ExcelOutput/RogueTournHandBookEvent.json")
  .filter(({ row }) => (row.UnlockNPCProgressIDList ?? []).some(
    (progress) => Math.floor(progress.FDOELDMEBPE / 100000) === 7,
  ));
const occurrenceNpcIds = new Set(
  handbookRows.flatMap(({ row }) => row.UnlockNPCProgressIDList ?? [])
    .filter((progress) => Math.floor(progress.FDOELDMEBPE / 100000) === 7)
    .map((progress) => String(progress.FDOELDMEBPE)),
);
const currentNpcRows = sourceRows("ExcelOutput/RogueTournNPC.json")
  .filter(({ row }) => row.NPCJsonPath?.includes("RogueNPC_410"));
category(
  "occurrences",
  "Handbook Occurrences with at least one explicit current prefix-7 NPC-progress reference.",
  handbookRows.map((entry) =>
    sourceRecord(entry, entry.row.EventHandbookID, {
      ownership: "DivergentUniverse",
      reachability: "TransitiveReference",
    })),
);
category(
  "occurrence_variants",
  "Current RogueNPC_410 rows whose stable NPC ID is referenced by a selected handbook Occurrence.",
  currentNpcRows
    .filter(({ row }) => occurrenceNpcIds.has(String(row.RogueNPCID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.RogueNPCID, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
);
category(
  "mode_service_npcs",
  "Current RogueNPC_410 rows not claimed by a handbook Occurrence; P2-B4 must classify every service.",
  currentNpcRows
    .filter(({ row }) => !occurrenceNpcIds.has(String(row.RogueNPCID)))
    .map((entry) =>
      sourceRecord(entry, entry.row.RogueNPCID, {
        ownership: "DivergentUniverse",
        reachability: "DirectModeConfig",
      })),
);
category(
  "adventure_outcomes",
  "All direct mode-owned abstract Adventure outcome rows.",
  directRows("ExcelOutput/RogueTournAdventureRoom.json", "RoomID", {
    id: (entry) => `${entry.row.RoomID}:${entry.row.AdventureType}`,
  }),
);

category(
  "encounter_source_obligations",
  "Tourn3 area entry locators plus the conservative room-reuse set and StageConfig root; P2-B5 expands each into waves and enemy slots.",
  [
    ...areaRows.map((entry) =>
      derivedRecord(entry, `area-entry:${entry.row.BEOFPCAACEP}`, {
        map_entry_id: entry.row.JJKLIJNFIBB,
      }, {
        ownership: "DivergentUniverse",
        reachability: "TransitiveReference",
      })),
    ...roomCandidates.map((entry) =>
      derivedRecord(entry, `room:${entry.row.RogueRoomID}`, {
        room_id: entry.row.RogueRoomID,
        room_type: entry.row.RogueRoomType,
      }, {
        ownership: "SharedCandidate",
        reachability: "PendingStageClosure",
      })),
    {
      id: "StageConfig",
      source: "ExcelOutput/StageConfig.json",
      evidence_sha256: inventory.records.find(
        ({ repository, path: sourcePath }) =>
          repository === "turnbasedgamedata"
            && sourcePath === "ExcelOutput/StageConfig.json",
      )?.sha256,
      evidence_quality: "ExactStructured",
      ownership: "Shared",
      reachability: "TransitiveReference",
    },
  ],
);

const mechanicFamilies = new Set([
  "divergent_adventure_graph_candidate",
  "divergent_adventure_modifier_evidence",
  "divergent_maze_graph_candidate",
  "divergent_mechanic_evidence",
  "divergent_npc_graph_candidate",
  "divergent_occurrence_graph_candidate",
  "divergent_service_graph_candidate",
]);
category(
  "mechanic_source_files",
  "Exact focused-inventory files in Divergent-specific mechanic, NPC, Occurrence, service, Adventure and maze families.",
  inventory.records.filter(({ family }) => mechanicFamilies.has(family))
    .map((record) => ({
      id: record.path,
      source: record.path,
      evidence_sha256: record.sha256,
      evidence_quality: "ExactStructured",
      ownership: "DivergentUniverse",
      reachability: "SourceObligation",
    })),
);

const fixtureFamilies = [
  "profile-and-module-selection",
  "ordinary-and-cyclical-entry",
  "area-difficulty-layer-transition",
  "finish-and-cross-battle-reset",
  "arithmetic-mapping-eligibility",
  "arithmetic-mapping-refresh-and-teardown",
  "equation-offer-recipe-progress-expansion",
  "equation-replacement-and-contribution",
  "divergent-blessing-level-and-transform",
  "curio-weight-charge-destruction-repair",
  "grand-miracle-eligibility-and-lifecycle",
  "golden-blood-titan-choice-and-level",
  "threshold-protocol",
  "astronomical-division",
  "star-pioneer-practice-and-cognoculi",
  "workbench-operation-and-price",
  "gamble-offer-outcome-and-fallback",
  "permanent-talent-and-unlock",
  "weekly-modifier-and-room-service",
  "occurrence-choice-cost-and-outcome",
  "adventure-abstract-outcome",
  "encounter-wave-and-boss-binding",
  "simultaneous-trigger-order",
  "no-legal-candidate-fallback",
  "battle-visible-and-cross-battle-contribution",
];
category(
  "semantic_fixture_families",
  "Minimum non-shrinking semantic fixture obligations; later batches may add cases.",
  fixtureFamilies.map((id) => ({
    id,
    source: "docs/goals/11-divergent-universe-reference-data.md",
    evidence_sha256: digest({ id, goal_id: "divergent-universe-reference-v1" }),
    evidence_quality: "ProjectPolicy",
    ownership: "DivergentUniverse",
    reachability: "Direct",
  })),
);

const namedModeExclusions = inventory.records
  .filter(({ family }) => family.includes("_exclusion_evidence")
    && family !== "presentation_account_exclusion_evidence")
  .map((record) => ({
    id: `${record.repository}/${record.path}`,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: record.family,
  }));
const presentationExclusions = inventory.records
  .filter(({ family }) => family === "presentation_account_exclusion_evidence"
    || family === "divergent_presentation_account_locator")
  .map((record) => ({
    id: `${record.repository}/${record.path}`,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: record.family,
  }));
const historicalRows = [
  ...sourceRows("ExcelOutput/RogueTournModule.json")
    .filter(({ row }) => row.ActivityModuleID !== 6002201),
  ...sourceRows("ExcelOutput/RogueTournArea.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
  ...sourceRows("ExcelOutput/RogueTournFormula.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
  ...sourceRows("ExcelOutput/RogueTournMiracle.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
].map((entry) => ({
  id: `${entry.file}#${entry.index}`,
  source: `${entry.file}#${entry.index}`,
  evidence_sha256: digest(entry.row),
  evidence_quality: "ExactStructured",
  ownership: "EvidenceOnly",
  reachability: "Excluded",
  reason: "explicit historical module/Tourn1/Tourn2 selector",
}));

const group = (...categoryIds) => ({
  categories: categoryIds,
  required: categoryIds.reduce((sum, id) => sum + categories[id].count, 0),
});
const counterGroups = {
  profiles_modules_entries_finish_conditions:
    group("profiles", "entry_points", "enabled_modules", "finish_conditions"),
  areas_difficulties_layers_rooms:
    group(
      "area_groups", "areas", "difficulties", "layers", "layer_rooms",
      "room_reuse_candidates", "room_types",
    ),
  threshold_protocol_astronomical_division:
    group("astronomical_divisions", "astronomical_division_effects"),
  arithmetic_mappings:
    group(
      "arithmetic_mapping_avatars", "arithmetic_mapping_build_refs",
      "arithmetic_mapping_roles",
    ),
  equations_recipes_expansion_states:
    group(
      "equations", "equation_displays", "equation_randomizers",
      "equation_keywords", "equation_keyword_params",
    ),
  divergent_blessings_levels_transforms:
    group("blessings", "blessing_levels", "blessing_groups"),
  curios_weighted_curios_states:
    group("curios", "curio_states", "curio_groups"),
  grand_miracles_hex_states:
    group("grand_miracles", "grand_miracle_eligibility"),
  golden_blood_titan_definitions:
    group("titan_types", "titan_bless_levels", "titan_talent_levels"),
  workbench_gamble_services:
    group(
      "workbenches", "workbench_functions", "gamble_groups", "gamble_units",
      "curse_chests",
    ),
  permanent_talents_unlocks_modifiers:
    group(
      "permanent_talents", "unlocks", "common_constants",
      "weekly_modifiers", "room_marks",
    ),
  blessing_path_shared_content_pools: group("blessing_paths"),
  occurrences_variants_choices:
    group("occurrences", "occurrence_variants"),
  services_adventure_outcomes:
    group(
      "mode_service_npcs", "adventure_outcomes", "workbenches",
      "workbench_functions", "gamble_groups", "gamble_units", "curse_chests",
    ),
  encounter_groups_waves_enemy_slots: group("encounter_source_obligations"),
  mechanic_rules: group("mechanic_source_files"),
  semantic_fixtures: group("semantic_fixture_families"),
};

const ownershipCounts = {};
for (const manifestCategory of Object.values(categories))
  for (const record of manifestCategory.records)
    ownershipCounts[record.ownership] = (ownershipCounts[record.ownership] ?? 0) + 1;

const payload = {
  schema_revision: "starclock.divergent-universe-content-manifest.v1",
  goal_id: "divergent-universe-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    identity_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  profile: "divergent-universe-v1",
  enabled_module: {
    sub_mode: "TournRogue",
    tourn_mode: "Tourn3",
    activity_module_id: 6002201,
    main_tourn_id: 3,
    sub_tourn_id: 1,
  },
  ownership_policy: {
    DivergentUniverse:
      "mode-owned row selected directly, by an exact Tourn3 selector or by a transitive reference from the enabled module",
    Shared:
      "shared stable source with an explicit reference from a selected mode row",
    SharedCandidate:
      "conservative frozen source obligation that is not reachable until a later exact stage/config closure proves it",
    EvidenceOnly:
      "excluded source retained only to prove the content boundary",
    fail_closed:
      "only TournRogue/Tourn3/module-6002201 selectors, transitive references or stable-ID closures grant reachability; prefixes, ID ranges and names do not",
  },
  denominator_policy: {
    current_module:
      "Version 4.4 enables only MainTournID 3/SubTournID 1/ActivityModuleID 6002201; Tourn1/Tourn2 selected rows are historical evidence unless separately proven reused",
    unversioned_mode_tables:
      "mechanically relevant unversioned RogueTourn tables are conservative direct obligations, not proof that every row is active in every module",
    room_reuse:
      "the source has no Tourn3 room or matching layer-room rows; all Tourn2 rooms freeze as SharedCandidate obligations and P1-B1 must prove or exclude each before DataReady",
    encounters:
      "area entry locators, room candidates and StageConfig freeze source obligations; P2-B5 expands waves, enemy slots and bosses with exact-once parent references",
    fixture_families:
      "the 25 family IDs are a non-shrinking minimum; multiple fixtures may satisfy one family",
  },
  exclusions: {
    mode_prefixes: [
      "RogueDLC",
      "RogueEndless",
      "RogueMagic",
      "RogueNous",
      "RoguePersona",
    ],
    sub_modes: [
      "ChessRogue",
      "ChessRogueNous",
      "MagicRogue",
      "CosmosRogue",
    ],
    historical_rows: historicalRows,
    historical_row_count: historicalRows.length,
    named_mode_source_files: namedModeExclusions,
    named_mode_source_count: namedModeExclusions.length,
    presentation_account_source_files: presentationExclusions,
    presentation_account_source_count: presentationExclusions.length,
  },
  counts: {
    categories: Object.keys(categories).length,
    records: Object.values(categories).reduce(
      (sum, manifestCategory) => sum + manifestCategory.count,
      0,
    ),
    ownership: Object.fromEntries(Object.entries(ownershipCounts).sort(
      ([left], [right]) => compare(left, right),
    )),
  },
  counter_groups: counterGroups,
  categories,
};

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  const committed = fs.readFileSync(output, "utf8");
  if (committed !== encoded)
    throw new Error("Divergent Universe content manifest has generated drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded, "utf8");
}
console.log(
  `Divergent Universe content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records} records in ${payload.counts.categories} categories.`,
);

function digest(value) {
  return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
