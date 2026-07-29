#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const root = path.resolve(valueAfter("--root") ?? ".");
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
const output = path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
);
const foundationPath = path.join(
  root,
  "content-manifests/currency-wars-v1/foundation.json",
);
const inventoryPath = path.join(
  root,
  "content-manifests/currency-wars-v1/source-inventory.json",
);
const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const categories = {};
const rowCache = new Map();

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function sourceRows(file) {
  if (!rowCache.has(file)) {
    const rows = JSON.parse(fs.readFileSync(path.join(sourceRoot, file), "utf8"));
    if (!Array.isArray(rows)) throw new Error(`expected source array: ${file}`);
    rowCache.set(file, rows.map((row, index) => ({ row, index, file })));
  }
  return rowCache.get(file);
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
function sourceFileRecord(record, fields = {}) {
  return {
    id: record.path,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ...fields,
  };
}
function policyRecord(id, evidence, fields = {}) {
  return {
    id,
    source: "content-manifests/currency-wars-v1/foundation.json",
    evidence_sha256: digest(evidence),
    evidence_quality: "ProjectPolicy",
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
    ownership = "CurrencyWars",
    reachability = "DirectModeTable",
    fields = () => ({}),
  } = options;
  return sourceRows(file).map((entry) => sourceRecord(entry, id(entry), {
    ownership,
    reachability,
    ...fields(entry),
  }));
}
function uniqueEntries(entries, key) {
  const selected = new Map();
  for (const entry of entries) {
    const id = String(entry.row[key]);
    if (!selected.has(id)) selected.set(id, entry);
  }
  return [...selected.values()];
}
function inventoryRecord(sourcePath) {
  const record = inventory.records.find(({ repository, path: recordPath }) =>
    repository === "turnbasedgamedata" && recordPath === sourcePath);
  if (!record) throw new Error(`source inventory path is missing ${sourcePath}`);
  return record;
}

const activityRows = sourceRows("ExcelOutput/RogueActivityResidentConfig.json");
const titleRows = sourceRows("ExcelOutput/RogueCommonModeTitle.json");
const moduleRows = sourceRows("ExcelOutput/RogueTournModule.json");
const currentActivity = activityRows.filter(({ row }) =>
  row.ActivityID === 105
    && row.SubMode === "TournRogue"
    && row.ActivityModuleID === 6002201);
if (currentActivity.length !== 1)
  throw new Error("expected exactly one Currency Wars resident activity");
const currentModule = moduleRows.filter(({ row }) =>
  row.MainTournID === 3
    && row.SubTournID === 1
    && row.ActivityModuleID === 6002201);
if (currentModule.length !== 1)
  throw new Error("expected exactly one Currency Wars Tourn module");

category("profiles",
  "One Candidate reference profile at the frozen Version 4.4 selector.",
  [policyRecord("currency-wars-v1", {
    game_version: "4.4",
    activity_id: 105,
    module_id: 6002201,
    tourn_mode: "Tourn3",
  }, {
    ownership: "CurrencyWars",
    reachability: "Direct",
  })]);
category("gambit_modes",
  "Released Standard and Overclock Gambit identities; detailed entry rules remain P1.",
  ["standard", "overclock"].map((id) => policyRecord(id, {
    id,
    source_boundary: "released Version 4.4 Currency Wars text",
  }, {
    ownership: "CurrencyWars",
    reachability: "Direct",
  })));
category("entry_points",
  "Resident activity and common-title rows with exact TournRogue selectors.",
  [
    ...currentActivity.map((entry) => sourceRecord(entry,
      `activity:${entry.row.ActivityID}`, {
        ownership: "CurrencyWars",
        reachability: "ExplicitModeSelector",
      })),
    ...titleRows.filter(({ row }) => row.SubMode === "TournRogue").map((entry) =>
      sourceRecord(entry, `title:${entry.row.SubMode}`, {
        ownership: "CurrencyWars",
        reachability: "ExplicitModeSelector",
      })),
  ]);
category("enabled_modules",
  "The unique MainTourn 3/SubTourn 1 row selected by activity module 6002201.",
  currentModule.map((entry) => sourceRecord(entry, entry.row.ActivityModuleID, {
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
  })));
category("finish_conditions",
  "Rows whose condition program explicitly tests RogueTourn mode 3.",
  sourceRows("ExcelOutput/RogueTournFinishway.json")
    .filter(({ row }) => JSON.stringify(row).includes("Cond_InRogueTournMode(3)"))
    .map((entry) => sourceRecord(entry, entry.row.ID, {
      ownership: "CurrencyWars",
      reachability: "ExplicitModeSelector",
    })));

const areaGroupRows = sourceRows("ExcelOutput/RogueTournAreaGroupByTourn.json")
  .filter(({ row }) => Object.values(row).includes("Tourn3"));
category("area_groups",
  "Area-group selector rows whose released value is exactly Tourn3.",
  areaGroupRows.map((entry) => sourceRecord(entry,
    `Tourn3:${entry.row.JFMBIOOCPIL}`, {
      ownership: "CurrencyWars",
      reachability: "ExplicitModeSelector",
    })));
const areaRows = sourceRows("ExcelOutput/RogueTournArea.json")
  .filter(({ row }) => row.HILINOJPLGA === "Tourn3");
category("areas",
  "Every area row whose TournMode selector is exactly Tourn3.",
  areaRows.map((entry) => sourceRecord(entry, entry.row.BEOFPCAACEP, {
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
  })));
const difficultyIds = new Set(
  areaRows.flatMap(({ row }) => row.EODCEHDOAEB ?? []).map(String),
);
category("difficulties",
  "Difficulty rows explicitly referenced by a Tourn3 area.",
  sourceRows("ExcelOutput/RogueTournDifficulty.json")
    .filter(({ row }) => difficultyIds.has(String(row.DifficultyID)))
    .map((entry) => sourceRecord(entry, entry.row.DifficultyID, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
const layerIds = new Set(
  areaRows.flatMap(({ row }) => row.GLNDIILFKBN ?? []).map(String),
);
category("layers",
  "Layer rows explicitly referenced by a Tourn3 area.",
  sourceRows("ExcelOutput/RogueTournLayer.json")
    .filter(({ row }) => layerIds.has(String(row.LayerID)))
    .map((entry) => sourceRecord(entry, entry.row.LayerID, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
const roomCandidates = sourceRows("ExcelOutput/RogueTournRoom.json")
  .filter(({ row }) => row.TournMode === "Tourn2");
category("room_reuse_candidates",
  "Tourn2 rooms retained as exact disposition obligations because no Tourn3 room row exists; stage closure must promote or exclude each.",
  roomCandidates.map((entry) => sourceRecord(entry, entry.row.RogueRoomID, {
    ownership: "EvidenceOnly",
    reachability: "PendingStageClosure",
  })));
category("rank_gambit_progression_envelopes",
  "One source-bounded parent for rank, Gambit and simulation-visible progression disposition.",
  [policyRecord("rank-gambit-progression-v1", {
    activity_module_id: 6002201,
    responsibility: "P1-B9",
  }, {
    ownership: "CurrencyWars",
    reachability: "Direct",
  })]);

category("squad_hp_action_value_envelopes",
  "One non-shrinking parent for Squad HP and action-value projection rows.",
  [policyRecord("squad-hp-action-value-v1", {
    ability_files: inventory.records
      .filter(({ family }) => family === "currency_wars_mechanic_evidence")
      .map(({ sha256 }) => sha256),
    responsibility: "P1-B2",
  }, {
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  })]);
category("roster_avatars",
  "Every released Tourn avatar mapping is a roster disposition obligation.",
  directRows("ExcelOutput/RogueTournAvatar.json", "AvatarID", {
    ownership: "Shared",
    reachability: "PendingReferenceClosure",
  }));
category("economy_shop_envelopes",
  "One parent for cost tiers, Gold Coins, refreshes, team Experience and team size.",
  [policyRecord("roster-economy-v1", {
    activity_module_id: 6002201,
    responsibility: "P1-B3",
  }, {
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  })]);
category("role_mappings",
  "Every released avatar role/buff row is a position or Empowerment disposition obligation.",
  directRows("ExcelOutput/RogueTournRole.json", "AvatarID", {
    ownership: "Shared",
    reachability: "PendingReferenceClosure",
  }));
category("position_empowerment_envelopes",
  "One parent for field/bench position, Empowerment and battle override semantics.",
  [policyRecord("position-empowerment-v1", {
    responsibility: "P1-B4",
    direct_ability_programs: 4,
  }, {
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  })]);
category("bond_envelopes",
  "One non-shrinking parent for complete Bond membership/threshold closure.",
  [policyRecord("bonds-v1", {
    responsibility: "P1-B5",
    replacement: "replace with source-backed member/threshold children",
  }, {
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  })]);
category("star_upgrade_envelopes",
  "One non-shrinking parent for copy combination, stars, scaling and teardown.",
  [policyRecord("star-upgrade-v1", {
    responsibility: "P1-B6",
    copies_per_upgrade: 3,
  }, {
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  })]);
category("build_reference_avatars",
  "Every shared build-reference avatar is an exact mapping disposition obligation.",
  directRows("ExcelOutput/RogueTournBuildRefAvatar.json", "AvatarID", {
    ownership: "Shared",
    reachability: "PendingReferenceClosure",
  }));
category("build_source_files",
  "All six shared build tables require an explicit Currency Wars mapping before promotion.",
  inventory.records.filter(({ family }) => family === "shared_build_mapping_candidate")
    .map((record) => sourceFileRecord(record, {
      ownership: "EvidenceOnly",
      reachability: "PendingReferenceClosure",
    })));

const personaTableKeys = new Map([
  ["ExcelOutput/RoguePersonaConstClient.json",
    (entry) => entry.row.ConstValueName],
  ["ExcelOutput/RoguePersonaConstCommon.json",
    (entry) => entry.row.ConstValueName],
  ["ExcelOutput/RoguePersonaLayerRoom.json",
    (entry) => `${entry.row.CBCHIHEOEGK}:${entry.row.EEPIDJJJMAH}`],
  ["ExcelOutput/RoguePersonaRoomAttribute.json",
    (entry) => entry.row.HHPFKDEBMGP],
  ["ExcelOutput/RoguePersonaRoomCompType.json",
    (entry) => entry.row.LLICIMBCNPF],
  ["ExcelOutput/RoguePersonaRoomComposition.json",
    (entry) => `${entry.row.LLICIMBCNPF}:${entry.row.AAGKEBFHLMC}`],
  ["ExcelOutput/RoguePersonaRoomPreset.json",
    (entry) => entry.row.LIIPLGLNPGB],
  ["ExcelOutput/RoguePersonaStyle.json",
    (entry) => entry.row.KLOEJIMMPJM],
  ["ExcelOutput/RoguePersonaStyleGift.json",
    (entry) => entry.row.FMDMDDCBPAM],
  ["ExcelOutput/RoguePersonaTalent.json",
    (entry) => entry.row.PHFMCACHFIJ],
  ["ExcelOutput/RoguePersonaTalentGroup.json",
    (entry) => entry.row.PHFMCACHFIJ],
]);
for (const [file, id] of personaTableKeys) {
  const categoryId = path.posix.basename(file, ".json")
    .replace(/^RoguePersona/u, "persona_")
    .replaceAll(/([a-z0-9])([A-Z])/gu, "$1_$2")
    .toLowerCase();
  category(categoryId,
    "Every Persona row remains an exact Currency Wars disposition obligation; B1/B8 prove its Tourn3 lifecycle.",
    sourceRows(file).map((entry) => sourceRecord(entry, id(entry), {
      ownership: "CurrencyWars",
      reachability: "TransitiveReference",
    })));
}

const activeBuffTypeRow = sourceRows("ExcelOutput/RogueTournUseBuffType.json")
  .find(({ row }) => row.TournMode === "Tourn3");
if (!activeBuffTypeRow) throw new Error("missing Tourn3 Blessing type selector");
const activeBuffTypes = new Set(activeBuffTypeRow.row.UseBuffTypeList.map(String));
category("blessing_paths",
  "Blessing types selected by the Tourn3 UseBuffTypeList.",
  sourceRows("ExcelOutput/RogueTournBuffType.json")
    .filter(({ row }) => activeBuffTypes.has(String(row.RogueBuffType)))
    .map((entry) => sourceRecord(entry, entry.row.RogueBuffType, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
const blessingLevelRows = sourceRows("ExcelOutput/RogueTournBuff.json")
  .filter(({ row }) => activeBuffTypes.has(String(row.RogueBuffType)));
category("blessings",
  "Distinct Blessings whose type is selected by Tourn3.",
  uniqueEntries(blessingLevelRows, "MazeBuffID").map((entry) =>
    sourceRecord(entry, entry.row.MazeBuffID, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
category("blessing_levels",
  "Every level row whose Blessing type is selected by Tourn3.",
  blessingLevelRows.map((entry) =>
    sourceRecord(entry, `${entry.row.MazeBuffID}:${entry.row.MazeBuffLevel}`, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
category("blessing_groups",
  "Every Blessing group row whose selector is exactly Tourn3.",
  sourceRows("ExcelOutput/RogueTournBuffGroup.json")
    .filter(({ row }) => row.TournMode === "Tourn3")
    .map((entry) => sourceRecord(entry, entry.row.RogueBuffGroupID, {
      ownership: "CurrencyWars",
      reachability: "ExplicitModeSelector",
    })));
const formulaRows = sourceRows("ExcelOutput/RogueTournFormula.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category("formulas",
  "Every Formula row whose selector is exactly Tourn3.",
  formulaRows.map((entry) => sourceRecord(entry, entry.row.FormulaID, {
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
  })));
const formulaDisplayIds = new Set(
  formulaRows.map(({ row }) => String(row.FormulaDisplayID)),
);
category("formula_displays",
  "Formula display rows referenced by a Tourn3 Formula.",
  sourceRows("ExcelOutput/RogueTournFormulaDisplay.json")
    .filter(({ row }) => formulaDisplayIds.has(String(row.FormulaDisplayID)))
    .map((entry) => sourceRecord(entry, entry.row.FormulaDisplayID, {
      ownership: "CurrencyWars",
      reachability: "TransitiveReference",
    })));
category("formula_randomizers",
  "All offer programs remain disposition obligations until formula references close.",
  directRows("ExcelOutput/RogueTournFormulaRandom.json", "RandomID", {
    ownership: "EvidenceOnly",
    reachability: "PendingReferenceClosure",
  }));

const miracleRows = sourceRows("ExcelOutput/RogueTournMiracle.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category("curio_states",
  "Every Curio state row whose selector is exactly Tourn3.",
  miracleRows.map((entry) => sourceRecord(entry, entry.row.MiracleID, {
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
  })));
const handbookMiracleIds = new Set(
  miracleRows.map(({ row }) => String(row.HandbookMiracleID)),
);
category("curios",
  "Handbook Curio identities referenced by a Tourn3 state row.",
  sourceRows("ExcelOutput/RogueTournHandbookMiracle.json")
    .filter(({ row }) =>
      handbookMiracleIds.has(String(row.HandbookMiracleID))
        && row.HandbookMiracleID !== undefined)
    .map((entry) => sourceRecord(entry, entry.row.HandbookMiracleID, {
      ownership: "Shared",
      reachability: "TransitiveReference",
    })));
category("curio_groups",
  "All Tourn Curio groups remain exact disposition obligations.",
  directRows("ExcelOutput/RogueTournMiracleGroup.json", "RogueMiracleGroupID", {
    ownership: "EvidenceOnly",
    reachability: "PendingReferenceClosure",
  }));
const hexRows = sourceRows("ExcelOutput/RogueTournHex.json")
  .filter(({ row }) => row.TournMode === "Tourn3");
category("hex_states",
  "Every Hex/Grand Miracle row whose selector is exactly Tourn3.",
  hexRows.map((entry) => sourceRecord(entry, entry.row.HexID, {
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
  })));
category("hex_eligibility",
  "All character eligibility rows remain obligations referenced by the Tourn3 Hex family.",
  directRows("ExcelOutput/RogueTournHexAvatarBaseType.json", "MiracleID", {
    id: (entry) =>
      `${entry.row.MiracleID}:${entry.row.AvatarType}:${entry.row.AvatarDamageType}`,
    ownership: "CurrencyWars",
    reachability: "TransitiveReference",
  }));

const handbookRows = sourceRows("ExcelOutput/RogueTournHandBookEvent.json")
  .filter(({ row }) => (row.UnlockNPCProgressIDList ?? []).some(
    (progress) => Math.floor(progress.FDOELDMEBPE / 100000) === 7,
  ));
category("occurrences",
  "Handbook Occurrences with an explicit current prefix-7 NPC-progress reference.",
  handbookRows.map((entry) => sourceRecord(entry, entry.row.EventHandbookID, {
    ownership: "Shared",
    reachability: "PendingReferenceClosure",
  })));
const currentNpcRows = sourceRows("ExcelOutput/RogueTournNPC.json")
  .filter(({ row }) => row.NPCJsonPath?.includes("RogueNPC_410"));
category("occurrence_service_variants",
  "Every current RogueNPC_410 row requires exact event/service disposition.",
  currentNpcRows.map((entry) => sourceRecord(entry, entry.row.RogueNPCID, {
    ownership: "EvidenceOnly",
    reachability: "PendingReferenceClosure",
  })));

category("workbenches", "All workbenches require explicit Tourn3 service proof.",
  directRows("ExcelOutput/RogueTournWorkbench.json", "WorkbenchID", {
    ownership: "EvidenceOnly", reachability: "PendingReferenceClosure",
  }));
category("workbench_functions",
  "All workbench functions require explicit Tourn3 service proof.",
  directRows("ExcelOutput/RogueTournWorkbenchFunc.json", "FuncID", {
    ownership: "EvidenceOnly", reachability: "PendingReferenceClosure",
  }));
category("gamble_groups", "All gamble groups require explicit Tourn3 service proof.",
  directRows("ExcelOutput/RogueTournGambleGroup.json", "GambleGroupID", {
    ownership: "EvidenceOnly", reachability: "PendingReferenceClosure",
  }));
category("gamble_units", "All gamble units require explicit Tourn3 service proof.",
  directRows("ExcelOutput/RogueTournGambleUnit.json", "GambleUnitID", {
    ownership: "EvidenceOnly", reachability: "PendingReferenceClosure",
  }));
category("curse_chests", "All curse chests require explicit Tourn3 service proof.",
  directRows("ExcelOutput/RogueTournCurseChest.json", "ChestID", {
    ownership: "EvidenceOnly", reachability: "PendingReferenceClosure",
  }));
category("adventure_outcomes",
  "All abstract Adventure outcomes require explicit Tourn3 room/service proof.",
  directRows("ExcelOutput/RogueTournAdventureRoom.json", "RoomID", {
    id: (entry) => `${entry.row.RoomID}:${entry.row.AdventureType}`,
    ownership: "EvidenceOnly",
    reachability: "PendingReferenceClosure",
  }));

category("encounter_source_obligations",
  "Tourn3 area entry locators plus StageConfig root; P2-B5 expands waves/enemy slots.",
  [
    ...areaRows.map((entry) => sourceRecord(entry,
      `area-entry:${entry.row.BEOFPCAACEP}`, {
        ownership: "CurrencyWars",
        reachability: "TransitiveReference",
      })),
    sourceFileRecord(inventoryRecord("ExcelOutput/StageConfig.json"), {
      ownership: "Shared",
      reachability: "TransitiveReference",
    }),
  ]);

const currentNpcPaths = new Set(currentNpcRows.map(({ row }) => row.NPCJsonPath));
const mechanicFamilies = new Set([
  "currency_wars_adventure_modifier_evidence",
  "currency_wars_maze_graph_candidate",
  "currency_wars_mechanic_evidence",
  "tourn_adventure_graph_candidate",
  "tourn_maze_graph_candidate",
  "tourn_occurrence_graph_candidate",
  "tourn_service_graph_candidate",
]);
category("mechanic_source_files",
  "Focused direct/shared source files requiring later rule disposition; no runtime lowering claim.",
  inventory.records.filter((record) =>
    mechanicFamilies.has(record.family) || currentNpcPaths.has(record.path))
    .map((record) => sourceFileRecord(record, {
      ownership: record.family.startsWith("currency_wars_")
        ? "CurrencyWars"
        : "EvidenceOnly",
      reachability: record.family.startsWith("currency_wars_")
        ? "SourceObligation"
        : "PendingReferenceClosure",
    })));

const fixtureFamilies = [
  "profile-gambit-entry-and-terminal",
  "three-plane-node-room-flow",
  "squad-hp-victory-timeout-and-run-failure",
  "roster-offer-cost-purchase-sale-and-cap",
  "gold-coin-refresh-experience-and-team-size",
  "field-bench-position-and-empowerment",
  "automatic-technique-energy-and-lethal-rescue",
  "bond-membership-threshold-and-recompute",
  "star-copy-combine-overflow-and-teardown",
  "owned-trial-build-substitution-and-removal",
  "off-field-conversion-and-equipment-slots",
  "investment-environment-strategy-and-persona",
  "blessing-level-offer-and-enhancement",
  "formula-recipe-progress-and-contribution",
  "curio-state-charge-destruction-and-repair",
  "hex-eligibility-activation-and-teardown",
  "occurrence-choice-cost-and-outcome",
  "shop-service-price-inventory-and-fallback",
  "encounter-wave-elite-and-boss-binding",
  "battle-visible-rule-contribution",
  "cross-battle-state-and-reset",
  "candidate-order-and-no-legal-result",
  "simultaneous-bond-star-and-roster-order",
  "squad-hp-action-value-same-boundary-order",
  "gambit-rank-and-enemy-affix",
  "approximation-replacement-trigger",
  "other-mode-ownership-rejection",
  "goal11-selector-conflict-reconciliation",
];
category("semantic_fixture_families",
  "Minimum non-shrinking semantic fixture obligations; later batches add cases.",
  fixtureFamilies.map((id) => policyRecord(id, {
    id,
    goal_id: "currency-wars-reference-v1",
  }, {
    ownership: "CurrencyWars",
    reachability: "Direct",
  })));

const counterGroups = {
  profiles_gambit_entries_finish: [
    "profiles", "gambit_modes", "entry_points", "enabled_modules",
    "finish_conditions",
  ],
  planes_difficulties_ranks_nodes_rooms: [
    "area_groups", "areas", "difficulties", "layers",
    "room_reuse_candidates", "rank_gambit_progression_envelopes",
  ],
  squad_hp_action_value_projections: ["squad_hp_action_value_envelopes"],
  roster_cost_shop_team_size_economy: [
    "roster_avatars", "economy_shop_envelopes",
  ],
  positions_character_empowerments: [
    "role_mappings", "position_empowerment_envelopes",
  ],
  bonds_members_levels: ["bond_envelopes"],
  star_states_copy_combinations: ["star_upgrade_envelopes"],
  build_mappings_equipment_conversions: [
    "build_reference_avatars", "build_source_files",
  ],
  investment_environment_strategy_persona: [...categoriesWithPrefix("persona_")],
  blessings_levels_formulas: [
    "blessing_paths", "blessings", "blessing_levels", "blessing_groups",
    "formulas", "formula_displays", "formula_randomizers",
  ],
  curios_miracles_hex_states: [
    "curio_states", "curios", "curio_groups", "hex_states", "hex_eligibility",
  ],
  events_variants_choices: ["occurrences", "occurrence_service_variants"],
  currencies_shops_services: [
    "workbenches", "workbench_functions", "gamble_groups", "gamble_units",
    "curse_chests", "adventure_outcomes",
  ],
  encounter_groups_waves_enemy_slots: ["encounter_source_obligations"],
  mechanic_rules: ["mechanic_source_files"],
  semantic_fixtures: ["semantic_fixture_families"],
};
const frozenCounters = Object.fromEntries(Object.entries(counterGroups)
  .map(([id, categoryIds]) => [id, {
    id,
    categories: categoryIds,
    required: categoryIds.reduce((sum, categoryId) =>
      sum + categories[categoryId].count, 0),
  }]));

const historicalRows = [
  ...sourceRows("ExcelOutput/RogueTournModule.json")
    .filter(({ row }) => row.ActivityModuleID !== 6002201),
  ...sourceRows("ExcelOutput/RogueTournAreaGroupByTourn.json")
    .filter(({ row }) => !Object.values(row).includes("Tourn3")),
  ...sourceRows("ExcelOutput/RogueTournArea.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
  ...sourceRows("ExcelOutput/RogueTournFormula.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
  ...sourceRows("ExcelOutput/RogueTournMiracle.json")
    .filter(({ row }) => ["Tourn1", "Tourn2"].includes(row.TournMode)),
  ...sourceRows("ExcelOutput/RogueTournBuffGroup.json")
    .filter(({ row }) => row.TournMode !== "Tourn3"),
  ...sourceRows("ExcelOutput/RogueTournUseBuffType.json")
    .filter(({ row }) => row.TournMode !== "Tourn3"),
].map((entry) => sourceRecord(entry, `${entry.file}#${entry.index}`, {
  ownership: "EvidenceOnly",
  reachability: "Excluded",
  exclusion: "non-Tourn3 module/selector",
}));
const namedModeExclusions = inventory.records
  .filter(({ family }) =>
    family.includes("_exclusion_evidence")
      || family.includes("_boundary_evidence")
      || family === "tourn_test_exclusion_evidence")
  .map((record) => sourceFileRecord(record, {
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    exclusion: record.family,
  }));
const presentationExclusions = inventory.records
  .filter(({ family }) =>
    family === "presentation_account_exclusion_evidence"
      || family === "currency_wars_presentation_locator"
      || family === "tourn_presentation_account_locator")
  .map((record) => sourceFileRecord(record, {
    ownership: "EvidenceOnly",
    reachability: "Excluded",
    exclusion: record.family,
  }));

const selectorConflictSources = [
  currentActivity[0],
  currentModule[0],
  areaGroupRows[0],
].map((entry) => ({
  source: `${entry.file}#${entry.index}`,
  evidence_sha256: digest(entry.row),
  evidence_summary:
    "Exact Version 4.4 TournRogue/Tourn3/module-6002201 selector claimed by both goals.",
}));
const reconciliation = [{
  goal: "Goal 11",
  remote: "origin",
  branch: "codex/goal11-divergent-universe-reference",
  commit: "982af8887fdd9ba29f1a323efc0ff5f6595ba411",
  manifest_path: "content-manifests/divergent-universe-v1/content-manifest.json",
  manifest_sha256: "5cbfa748406204e2d7a2c10c452ac6a87b3864b76461e85bd449c0739f3fc13e",
  state: "ConflictPendingMergeCoordination",
  conflict:
    "Goal 11 classifies TournRogue/Tourn3/module 6002201 as DivergentUniverse while its plan explicitly excludes Currency Wars; Goal 12 uses the same exact selector for Currency Wars.",
  source_records: selectorConflictSources,
  owner: "G12-P4-B3",
  replacement_condition:
    "A merge-stage ownership decision publishes non-overlapping mode selectors or an explicit shared/module-version split and both manifests regenerate from that decision.",
}];

const ownershipCounts = {};
for (const categoryValue of Object.values(categories))
  for (const record of categoryValue.records)
    ownershipCounts[record.ownership] =
      (ownershipCounts[record.ownership] ?? 0) + 1;
const payload = {
  schema_revision: "starclock.currency-wars-content-manifest.v1",
  goal_id: "currency-wars-reference-v1",
  profile: "currency-wars-v1",
  snapshot: {
    game_version: "4.4",
    source_revision: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    identity_revision: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  inputs: {
    foundation: {
      path: "content-manifests/currency-wars-v1/foundation.json",
      sha256: fileDigest(foundationPath),
    },
    source_inventory: {
      path: "content-manifests/currency-wars-v1/source-inventory.json",
      sha256: fileDigest(inventoryPath),
    },
  },
  enabled_module: {
    activity_id: 105,
    sub_mode: "TournRogue",
    tourn_mode: "Tourn3",
    activity_module_id: 6002201,
    main_tourn_id: 3,
    sub_tourn_id: 1,
  },
  ownership_policy: {
    permitted: ["CurrencyWars", "Shared", "EvidenceOnly"],
    fail_closed:
      "Only an explicit enabled selector, typed transitive reference or inherited stable-ID closure can promote a row; pending and conflicting records remain EvidenceOnly.",
    conflict_policy:
      "Goal 11 overlap is recorded by source locator and digest and must not be resolved by editing another Goal's artifacts.",
  },
  counter_groups: frozenCounters,
  counts: {
    categories: Object.keys(categories).length,
    records: Object.values(categories).reduce(
      (sum, categoryValue) => sum + categoryValue.count, 0),
    ownership: Object.fromEntries(Object.entries(ownershipCounts).sort()),
    exclusions: {
      historical_rows: historicalRows.length,
      named_mode_source_files: namedModeExclusions.length,
      presentation_account_source_files: presentationExclusions.length,
    },
    reconciliation_conflicts: reconciliation.length,
  },
  categories,
  exclusions: {
    historical_rows: historicalRows,
    named_mode_source_files: namedModeExclusions,
    presentation_account_source_files: presentationExclusions,
  },
  reconciliation,
};

const encoded = `${JSON.stringify(payload, null, 2)}\n`;
if (check) {
  if (fs.readFileSync(output, "utf8") !== encoded)
    throw new Error("Currency Wars content manifest has generated drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded, "utf8");
}
console.log(
  `Currency Wars content manifest ${check ? "verified" : "generated"}: ` +
  `${payload.counts.records.toLocaleString("en-US")} obligations in ` +
  `${payload.counts.categories} categories; ` +
  `${payload.counts.ownership.CurrencyWars ?? 0} mode-owned, ` +
  `${payload.counts.ownership.Shared ?? 0} shared, ` +
  `${payload.counts.ownership.EvidenceOnly ?? 0} fail-closed evidence.`,
);

function categoriesWithPrefix(prefix) {
  return Object.keys(categories).filter((id) => id.startsWith(prefix)).sort(compare);
}
function digest(value) {
  return crypto.createHash("sha256")
    .update(`${JSON.stringify(value)}\n`).digest("hex");
}
function fileDigest(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
