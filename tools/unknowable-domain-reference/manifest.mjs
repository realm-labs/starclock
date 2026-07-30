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
  "content-manifests/unknowable-domain-v1/content-manifest.json",
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
function derivedRecord(entry, id, value, fields = {}) {
  return {
    id: String(id),
    source: `${entry.file}#${entry.index}`,
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
      reachability: "ExplicitModeSelector",
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
function modeRows(file, key, options = {}) {
  const {
    id = (entry) => entry.row[key],
    fields = {},
  } = options;
  return sourceRows(file).map((entry) => sourceRecord(entry, id(entry), {
    ownership: "UnknowableDomain",
    reachability: "Direct",
    ...fields,
  }));
}
function uniqueBy(rows, key) {
  const values = new Map();
  for (const entry of rows) {
    const id = String(entry.row[key]);
    if (!values.has(id)) values.set(id, entry);
  }
  return [...values.values()];
}

const activityEntries = sourceRows("ExcelOutput/RogueActivityResidentConfig.json");
const modeTitles = sourceRows("ExcelOutput/RogueCommonModeTitle.json");
category(
  "profiles",
  "One project profile for the Version 4.4 MagicRogue activity boundary.",
  [{
    id: "unknowable-domain-v1",
    source: "content-manifests/unknowable-domain-v1/foundation.json",
    evidence_sha256: digest({
      goal_id: "unknowable-domain-reference-v1",
      sub_mode: "MagicRogue",
      game_version: "4.4",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "UnknowableDomain",
    reachability: "Direct",
  }],
);
category(
  "entry_points",
  "Resident activity and common-title rows whose sub-mode is exactly MagicRogue.",
  [
    ...activityEntries.filter(({ row }) => row.SubMode === "MagicRogue")
      .map((entry) => sourceRecord(entry, `activity:${entry.row.ActivityID}`, {
        ownership: "UnknowableDomain", reachability: "ExplicitModeSelector",
      })),
    ...modeTitles.filter(({ row }) => row.SubMode === "MagicRogue")
      .map((entry) => sourceRecord(entry, `title:${entry.row.SubMode}`, {
        ownership: "UnknowableDomain", reachability: "ExplicitModeSelector",
      })),
  ],
);

const areas = sourceRows("ExcelOutput/RogueMagicArea.json");
category(
  "areas",
  "Every released RogueMagic area row; area group remains an explicit field.",
  areas.map((entry) => sourceRecord(entry, entry.row.AreaID, {
    ownership: "UnknowableDomain",
    reachability: "Direct",
    area_group: entry.row.AreaGroupID,
  })),
);
category(
  "difficulty_compositions",
  "Every released RogueMagic difficulty-composition row.",
  modeRows("ExcelOutput/RogueMagicDifficultyComp.json", "DifficultyCompID"),
);
category(
  "difficulty_drops",
  "Every released area/world-level difficulty drop-display row.",
  modeRows("ExcelOutput/RogueMagicDifficultyDrop.json", "AreaID", {
    id: ({ row }) => `${row.AreaID}:${row.WorldLevel}`,
  }),
);
category(
  "layers",
  "Every released RogueMagic layer identity.",
  modeRows("ExcelOutput/RogueMagicLayer.json", "LayerID"),
);
category(
  "layer_rooms",
  "Every released ordered layer-room position.",
  modeRows("ExcelOutput/RogueMagicLayerRoom.json", "LayerID", {
    id: ({ row }) => `${row.LayerID}:${row.RoomIndex}`,
  }),
);
const roomRows = sourceRows("ExcelOutput/RogueMagicRoom.json");
category(
  "rooms",
  "Every released RogueMagic room identity and room-type binding.",
  roomRows.map((entry) => sourceRecord(entry, entry.row.RogueRoomID, {
    ownership: "UnknowableDomain",
    reachability: "Direct",
    room_type: entry.row.RogueRoomType,
  })),
);
category(
  "room_types",
  "Distinct room types present in released RogueMagic room bindings.",
  [...new Set(roomRows.map(({ row }) => row.RogueRoomType))].map((roomType) => ({
    id: roomType,
    source: "ExcelOutput/RogueMagicRoom.json",
    evidence_sha256: digest(roomRows
      .filter(({ row }) => row.RogueRoomType === roomType)
      .map(({ row }) => row.RogueRoomID)),
    evidence_quality: "ExactStructured",
    ownership: "UnknowableDomain",
    reachability: "Referenced",
  })),
);
category(
  "finish_conditions",
  "Every released RogueMagic finish/progress condition.",
  modeRows("ExcelOutput/RogueMagicFinishway.json", "ID"),
);

const alignmentRows = sourceRows("ExcelOutput/RogueMagicStyleTypeSelect.json");
category(
  "alignments",
  "All four released Extrapolation Alignment selectors.",
  alignmentRows.map((entry) => sourceRecord(entry, entry.row.EnumType, {
    ownership: "UnknowableDomain",
    reachability: "Direct",
  })),
);
const scepterRows = sourceRows("ExcelOutput/RogueMagicScepter.json");
category(
  "scepters",
  "Distinct Scepter definitions at their first released level.",
  uniqueBy(scepterRows, "ScepterID").map((entry) =>
    sourceRecord(entry, entry.row.ScepterID, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
      style_type: entry.row.StyleType,
    })),
);
category(
  "scepter_levels",
  "Every released Scepter definition/level pair.",
  scepterRows.map((entry) =>
    sourceRecord(entry, `${entry.row.ScepterID}:${entry.row.ScepterLevel}`, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
      style_type: entry.row.StyleType,
    })),
);
category(
  "scepter_locked_components",
  "Every released Scepter-level locked Component binding.",
  scepterRows.flatMap((entry) =>
    (entry.row.LockMagicUnit ?? []).map((locked, index) =>
      derivedRecord(
        entry,
        `${entry.row.ScepterID}:${entry.row.ScepterLevel}:${index}`,
        locked,
        {
          ownership: "UnknowableDomain",
          reachability: "Referenced",
          scepter_id: String(entry.row.ScepterID),
          scepter_level: entry.row.ScepterLevel,
          component_id: String(locked.GDDPJLJKGEO),
          component_level: locked.LPCBFACBGAE,
        },
      ))),
);
const slotLayouts = new Map();
for (const entry of scepterRows) {
  const key = JSON.stringify(entry.row.TrenchCount);
  if (!slotLayouts.has(key)) slotLayouts.set(key, entry);
}
category(
  "slot_layouts",
  "Distinct released Active/Attach/Passive Scepter slot-count layouts.",
  [...slotLayouts.entries()].map(([layout, entry], index) =>
    derivedRecord(entry, index + 1, JSON.parse(layout), {
      ownership: "UnknowableDomain",
      reachability: "Referenced",
    })),
);

const componentRows = sourceRows("ExcelOutput/RogueMagicUnit.json");
category(
  "components",
  "Distinct Component definitions at their first released level.",
  uniqueBy(componentRows, "MagicUnitID").map((entry) =>
    sourceRecord(entry, entry.row.MagicUnitID, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
      component_category: entry.row.MagicUnitCategory,
      component_type: entry.row.MagicUnitType,
    })),
);
category(
  "component_levels",
  "Every released Component definition/level pair.",
  componentRows.map((entry) =>
    sourceRecord(entry, `${entry.row.MagicUnitID}:${entry.row.MagicUnitLevel}`, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
      component_category: entry.row.MagicUnitCategory,
      component_type: entry.row.MagicUnitType,
    })),
);
category(
  "decision_components",
  "Distinct Ultra-category Components; later normalization freezes their choice programs.",
  uniqueBy(componentRows.filter(({ row }) => row.MagicUnitCategory === "Ultra"),
    "MagicUnitID").map((entry) =>
      sourceRecord(entry, entry.row.MagicUnitID, {
        ownership: "UnknowableDomain",
        reachability: "ExplicitModeSelector",
      })),
);
category(
  "component_categories",
  "Distinct released Component category values.",
  [...new Set(componentRows.map(({ row }) => row.MagicUnitCategory))]
    .map((value) => ({
      id: value,
      source: "ExcelOutput/RogueMagicUnit.json",
      evidence_sha256: digest(value),
      evidence_quality: "ExactStructured",
      ownership: "UnknowableDomain",
      reachability: "Referenced",
    })),
);
category(
  "component_types",
  "Distinct released Active, Attach and Passive Component type values.",
  [...new Set(componentRows.map(({ row }) => row.MagicUnitType))]
    .map((value) => ({
      id: value,
      source: "ExcelOutput/RogueMagicUnit.json",
      evidence_sha256: digest(value),
      evidence_quality: "ExactStructured",
      ownership: "UnknowableDomain",
      reachability: "Referenced",
    })),
);
const componentBuffIds = new Set(componentRows.map(({ row }) =>
  row.MagicUnitMazeBuffID));
category(
  "component_effects",
  "RogueMagic maze-buff rows directly referenced by released Component levels.",
  sourceRows("ExcelOutput/RogueMagicMazeBuff.json")
    .filter(({ row }) => componentBuffIds.has(row.ID))
    .map((entry) => sourceRecord(entry, `${entry.row.ID}:${entry.row.Lv}`, {
      ownership: "UnknowableDomain",
      reachability: "Referenced",
    })),
);

category(
  "mode_constants",
  "Every released common RogueMagic constant; client-only display constants are excluded.",
  modeRows("ExcelOutput/RogueMagicConstCommon.json", "ConstValueName"),
);
category(
  "layer_effects",
  "Every released RogueMagic layer-effect row.",
  modeRows("ExcelOutput/RogueMagicLayerEffect.json", "LayerEffectID"),
);
category(
  "maze_buffs",
  "Every released RogueMagic maze-buff row, including Component and progression effects.",
  modeRows("ExcelOutput/RogueMagicMazeBuff.json", "ID", {
    id: ({ row }) => `${row.ID}:${row.Lv}`,
  }),
);
category(
  "talents",
  "Every released mechanically authored RogueMagic Talent level.",
  modeRows("ExcelOutput/RogueMagicTalent.json", "TalentID", {
    id: ({ row }) => `${row.TalentID}:${row.Level}`,
  }),
);
category(
  "unlocks",
  "Every released RogueMagic unlock row.",
  modeRows("ExcelOutput/RogueMagicUnlock.json", "RogueUnlockID"),
);
category(
  "score_inputs",
  "Every released layer/room score input retained for finish-boundary review; account rewards remain excluded.",
  modeRows("ExcelOutput/RogueMagicScore.json", "LayerNum", {
    id: ({ row }) => `${row.WorldLevel ?? "default"}:${row.LayerNum}:${row.RoomNum}`,
  }),
);

category(
  "workbenches",
  "Every released RogueMagic workbench definition.",
  modeRows("ExcelOutput/RogueMagicWorkbench.json", "WorkbenchID"),
);
category(
  "workbench_functions",
  "Every released Scepter/Component shop, compose, reforge and upgrade function.",
  modeRows("ExcelOutput/RogueMagicWorkbenchFunc.json", "FuncID"),
);
category(
  "gamble_groups",
  "Every released RogueMagic gamble group.",
  modeRows("ExcelOutput/RogueMagicGambleGroup.json", "GambleGroupID"),
);
category(
  "gamble_units",
  "Every released RogueMagic gamble outcome unit.",
  modeRows("ExcelOutput/RogueMagicGambleUnit.json", "GambleUnitID"),
);
category(
  "adventure_outcomes",
  "Every released RogueMagic abstract Adventure outcome tier.",
  modeRows("ExcelOutput/RogueMagicAdventureRoom.json", "RoomID"),
);

category(
  "blessings",
  "No released RogueBuff row carries a MagicRogue/260 selector and no RogueMagic table references a standard Blessing; Components are the mode-owned upgrade pool.",
  [],
);
const curioHandbooks = sourceRows("ExcelOutput/RogueHandbookMiracle.json")
  .filter(({ row }) => row.MiracleTypeList.includes(260));
category(
  "curios",
  "Every shared Curio handbook row with explicit mode type 260.",
  curioHandbooks.map((entry) => sourceRecord(entry, entry.row.MiracleHandbookID, {
    ownership: "Shared",
    reachability: "ExplicitModeSelector",
  })),
);
category(
  "curio_states",
  "Every released RogueMagic Curio copy/state row.",
  modeRows("ExcelOutput/RogueMagicMiracle.json", "MiracleID"),
);
category(
  "curio_groups",
  "Every released RogueMagic weighted Curio group.",
  modeRows("ExcelOutput/RogueMagicMiracleGroup.json", "RogueMiracleGroupID"),
);
const occurrenceHandbooks = sourceRows("ExcelOutput/RogueHandBookEvent.json")
  .filter(({ row }) => row.EventTypeList.includes(260));
const occurrenceProgressIds = new Set(occurrenceHandbooks
  .flatMap(({ row }) => row.UnlockNPCProgressIDList ?? [])
  .map((progress) => progress.FDOELDMEBPE));
category(
  "occurrences",
  "Every shared Occurrence handbook row with explicit mode type 260.",
  occurrenceHandbooks.map((entry) =>
    sourceRecord(entry, entry.row.EventHandbookID, {
      ownership: "Shared",
      reachability: "ExplicitModeSelector",
    })),
);
category(
  "occurrence_variants",
  "Every released RogueMagic NPC/progress variant directly referenced by a type-260 Occurrence handbook.",
  sourceRows("ExcelOutput/RogueMagicNPC.json")
    .filter(({ row }) => occurrenceProgressIds.has(row.RogueNPCID))
    .map((entry) => sourceRecord(entry, entry.row.RogueNPCID, {
      ownership: "UnknowableDomain",
      reachability: "Referenced",
    })),
);
category(
  "mode_service_npcs",
  "RogueMagic NPC graphs not referenced by a type-260 Occurrence handbook; retained for entry and service closure.",
  sourceRows("ExcelOutput/RogueMagicNPC.json")
    .filter(({ row }) => !occurrenceProgressIds.has(row.RogueNPCID))
    .map((entry) => sourceRecord(entry, entry.row.RogueNPCID, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
    })),
);

const bossChoices = new Map();
for (const entry of areas)
  for (const value of entry.row.WorldLevel2DisplayMonster ?? [])
    if (!bossChoices.has(value.DBLDCKODNEN))
      bossChoices.set(value.DBLDCKODNEN, { entry, value });
category(
  "boss_choices",
  "Distinct displayed boss/enemy identities referenced by released RogueMagic areas.",
  [...bossChoices.entries()].map(([monsterId, { entry, value }]) =>
    derivedRecord(entry, monsterId, value, {
      ownership: "Shared",
      reachability: "Referenced",
    })),
);
category(
  "encounter_source_obligations",
  "Every mode room binding plus displayed boss identity; P2-B5 expands exact StageConfig waves and enemy slots.",
  [
    ...roomRows.map((entry) => sourceRecord(entry, `room:${entry.row.RogueRoomID}`, {
      ownership: "UnknowableDomain",
      reachability: "Direct",
    })),
    ...[...bossChoices.entries()].map(([monsterId, { entry, value }]) =>
      derivedRecord(entry, `boss:${monsterId}`, value, {
        ownership: "Shared",
        reachability: "Referenced",
      })),
  ],
);

const sourceInventory = JSON.parse(fs.readFileSync(
  path.join(root, "content-manifests/unknowable-domain-v1/source-inventory.json"),
  "utf8",
));
const mechanicFamilies = new Set([
  "unknowable_adventure_modifier_evidence",
  "unknowable_battle_event_candidate",
  "unknowable_maze_graph_candidate",
  "unknowable_mechanic_evidence",
  "unknowable_progression_graph_candidate",
  "unknowable_service_graph_candidate",
]);
category(
  "mechanic_source_files",
  "Every mode-named ability, battle-event, Adventure, maze, progression and service graph in the focused inventory.",
  sourceInventory.records.filter(({ family }) => mechanicFamilies.has(family))
    .map((record) => ({
      id: record.path,
      source: record.path,
      evidence_sha256: record.sha256,
      evidence_quality: "ExactStructured",
      ownership: "UnknowableDomain",
      reachability: "Direct",
    })),
);
const fixtureFamilies = [
  "profile-entry-and-finish",
  "area-layer-room-transition",
  "difficulty-composition",
  "alignment-selection",
  "scepter-activation",
  "scepter-charge-and-speed",
  "component-slot-legality",
  "component-insertion-removal-replacement",
  "decision-component-choice",
  "component-synthesis",
  "component-upgrade",
  "component-reforge",
  "workbench-offer-and-cost",
  "gamble-offer-and-outcome",
  "talent-and-unlock",
  "layer-and-difficulty-effect",
  "curio-lifecycle",
  "occurrence-choice",
  "service-and-adventure",
  "encounter-selection",
  "wave-and-boss-binding",
  "cross-battle-carry-reset",
  "simultaneous-trigger-order",
  "no-legal-candidate-fallback",
];
category(
  "semantic_fixture_families",
  "Minimum non-shrinking semantic family obligations; later batches may add cases.",
  fixtureFamilies.map((id) => ({
    id,
    source: "docs/goals/10-unknowable-domain-reference-data.md",
    evidence_sha256: digest({ id, goal_id: "unknowable-domain-reference-v1" }),
    evidence_quality: "ProjectPolicy",
    ownership: "UnknowableDomain",
    reachability: "Direct",
  })),
);

const namedModeExclusions = sourceInventory.records
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
const presentationExclusions = sourceInventory.records
  .filter(({ family }) => family === "presentation_account_exclusion_evidence"
    || family === "unknowable_presentation_locator")
  .map((record) => ({
    id: `${record.repository}/${record.path}`,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    reason: record.family,
  }));

const group = (...categoryIds) => ({
  categories: categoryIds,
  required: categoryIds.reduce((sum, id) => sum + categories[id].count, 0),
});
const counterGroups = {
  profiles_entries_finish_conditions:
    group("profiles", "entry_points", "finish_conditions"),
  areas_difficulties_layers_rooms:
    group(
      "areas", "difficulty_compositions", "difficulty_drops", "layers",
      "layer_rooms", "rooms", "room_types",
    ),
  extrapolation_alignments: group("alignments"),
  scepters_levels_states:
    group("scepters", "scepter_levels", "scepter_locked_components"),
  components_levels_effects:
    group(
      "components", "component_levels", "component_categories",
      "component_types", "component_effects",
    ),
  decision_components_choices: group("decision_components"),
  loadouts_slots_insertion_replacement: group("slot_layouts"),
  synthesis_upgrades_reforges: group("workbench_functions"),
  workbench_gamble_services:
    group("workbenches", "workbench_functions", "gamble_groups", "gamble_units"),
  talents_unlocks_layer_difficulty_effects:
    group(
      "talents", "unlocks", "layer_effects", "maze_buffs", "mode_constants",
      "score_inputs",
    ),
  blessings_enhanced_levels: group("blessings"),
  curios_states: group("curios", "curio_states", "curio_groups"),
  occurrences_variants_choices: group("occurrences", "occurrence_variants"),
  services_adventure_outcomes:
    group(
      "workbenches", "workbench_functions", "gamble_groups", "gamble_units",
      "adventure_outcomes", "mode_service_npcs",
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
  schema_revision: "starclock.unknowable-domain-content-manifest.v1",
  goal_id: "unknowable-domain-reference-v1",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  },
  profile: "unknowable-domain-v1",
  ownership_policy: {
    UnknowableDomain: "mode-owned row or mode-specific representation",
    Shared:
      "stable shared identity with explicit type-260 selector or a direct released reference",
    EvidenceOnly: "excluded source retained only to prove the content boundary",
    fail_closed:
      "only explicit MagicRogue/type-260 selectors, transitive references or inherited stable-ID closures grant reachability; prefixes, IDs and names do not",
  },
  denominator_policy: {
    source_obligation:
      "counts freeze exact source and inherited stable-ID obligations; normalized child rows may expand but cannot remove an obligation",
    blessings:
      "the released source exposes no MagicRogue/type-260 Blessing selector or RogueMagic-to-Blessing reference, so reachable Blessing obligations freeze at zero unless stronger released evidence is registered",
    components:
      "Component rows replace a generic Blessing assumption; Ultra category is the Decision Component candidate boundary pending P1-B5 semantics",
    encounters:
      "room bindings and displayed boss identities are frozen source obligations; P2-B5 expands StageConfig waves and enemy slots with exact-once parent references",
    fixture_families:
      "the 24 family IDs are a non-shrinking minimum; multiple fixtures may satisfy one family",
  },
  exclusions: {
    mode_prefixes: [
      "RogueDLC",
      "RogueEndless",
      "RogueNous",
      "RoguePersona",
      "RogueTourn"
    ],
    sub_modes: [
      "ChessRogue",
      "ChessRogueNous",
      "TournRogue",
      "CosmosRogue"
    ],
    named_mode_source_files: namedModeExclusions,
    named_mode_source_count: namedModeExclusions.length,
    presentation_account_source_files: presentationExclusions,
    presentation_account_source_count: presentationExclusions.length,
    goal08_checkpoint: {
      commit: "2f7b3ccf699c52c2738136b8636d140e053bb2eb",
      manifest_sha256: "88885b409da0037b4db6a41fcfc6adbbb1bc15a681c519e192251e7fef476085",
      records: 7913,
      required_for_foundation: false,
    },
    goal09_checkpoint: {
      commit: "9bd2ad285de4c10e7ab060f00bf078855923a09c",
      manifest_sha256: "e466cae0481d93241eaadf6d894b82898d47c9d4863fea262134cbbac10b8850",
      records: 6963,
      required_ancestor: true,
    },
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
    throw new Error("Unknowable Domain content manifest has generated drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded, "utf8");
}
console.log(
  `Unknowable Domain content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records} records in ${payload.counts.categories} categories.`,
);

function digest(value) {
  return crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex");
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
