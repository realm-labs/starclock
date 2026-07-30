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
const correctionPath = path.join(
  root,
  "content-manifests/currency-wars-v1/source-correction.json",
);
const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const correction = JSON.parse(fs.readFileSync(correctionPath, "utf8"));
const rowCache = new Map();

const groupDefinitions = {
  profiles_gambit_entries_finish:
    "GridFight identity, released entry/module, unlock and terminal records.",
  planes_difficulties_ranks_nodes_rooms:
    "GridFight division, route, node, level and tutorial topology records.",
  squad_hp_action_value_projections:
    "GridFight Stage and penalty rows that author Squad HP and action-value boundaries.",
  roster_cost_shop_team_size_economy:
    "GridFight roster, level, offer, price, resource and economy records.",
  positions_character_empowerments:
    "GridFight front/back position, skill, servant and battle-override records.",
  bonds_members_levels:
    "GridFight trait, sub-trait, threshold and contribution records.",
  star_states_copy_combinations:
    "GridFight star, rank attachment and combination records.",
  build_mappings_equipment_conversions:
    "GridFight equipment, recommendation, craft and forge records.",
  investment_environment_strategy_persona:
    "GridFight Augment, Portal, Projection, Talent and enhancement records; Persona is superseded.",
  blessings_levels_formulas:
    "GridFight affix and MazeBuff bridge rows; shared Blessing membership still requires reference closure.",
  curios_miracles_hex_states:
    "Closed direct GridFight Curio/Miracle/Hex namespace result; shared reachability remains a later closure.",
  events_variants_choices:
    "GridFight Pray, Present, assistant and tutorial task records.",
  currencies_shops_services:
    "GridFight function, shop, consumable, item and special-service records.",
  encounter_groups_waves_enemy_slots:
    "GridFight monster, elite, camp, difficulty and formation-wave records.",
  mechanic_rules:
    "Remaining GridFight table rows plus every direct GridFight configuration file.",
  semantic_fixtures:
    "Non-shrinking reference-semantic fixture-family obligations.",
};
const categories = Object.fromEntries(Object.entries(groupDefinitions)
  .map(([id, membershipBasis]) => [id, {
    id,
    membership_basis: membershipBasis,
    count: 0,
    records: [],
  }]));

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
  "investment-environment-strategy-and-augment",
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
  "goal11-selector-separation-reconciliation",
];

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function digest(value) {
  return crypto.createHash("sha256")
    .update(JSON.stringify(value)).digest("hex");
}
function fileDigest(file) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(file)).digest("hex");
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function sourceRows(file) {
  if (!rowCache.has(file)) {
    const rows = JSON.parse(fs.readFileSync(path.join(sourceRoot, file), "utf8"));
    if (!Array.isArray(rows)) throw new Error(`expected source array: ${file}`);
    rowCache.set(file, rows);
  }
  return rowCache.get(file);
}
function add(group, record) {
  if (!categories[group]) throw new Error(`unknown counter group ${group}`);
  categories[group].records.push(record);
}
function upstreamIdentity(row) {
  const preferredKeys = [
    "ID", "SeasonID", "ModuleID", "StageID", "NodeID", "RouteID",
    "DivisionID", "AvatarID", "RoleID", "TraitID", "AugmentID",
    "EquipmentID", "MonsterID", "GroupID", "RuleID", "BuffID",
  ];
  const values = {};
  for (const key of preferredKeys)
    if (row[key] !== undefined && row[key] !== null)
      values[key] = String(row[key]);
  return values;
}
function groupFor(table) {
  const name = table.replace(/^GridFight/u, "");
  if (/^(?:SeasonModule|SettleRank|Unlock|GuideQuest|PrayQuestFinishWay)/u
    .test(name))
    return "profiles_gambit_entries_finish";
  if (/^(?:Division|Node|Binary|TutorialStage|LevelBaseValue|StageLevelValue)/u
    .test(name) || name === "StageRoute")
    return "planes_difficulties_ranks_nodes_rooms";
  if (/^(?:Stage|PenaltyRule)$/u.test(name))
    return "squad_hp_action_value_projections";
  if (/^(?:RoleBasicInfo|RoleChoose|CoreRole|RoleAutoWeight|RarityWeight|PlayerLevel|LevelV2|Const|BasicBonus|Bonus|RandomBonus|CombinationBonus|VictoryBonus)/u
    .test(name))
    return "roster_cost_shop_team_size_economy";
  if (/^(?:Front|Back|ServantSkill|RoleGlobalModifier|RoleProperty|RoleSkillDisplay|RoleSwitch|RankSkillModify|SummonBE|CyreneModify|ElationEquip|GenderOverride|OverrideRoleVO)/u
    .test(name))
    return "positions_character_empowerments";
  if (/^(?:Trait|SubTrait|ModuleSubTrait|ModuleSwitchTrait|ModuleTraitSwitch|SeasonTrait)/u
    .test(name))
    return "bonds_members_levels";
  if (/^(?:RoleStar|ServantStar|RankAttachment|AvatarRankConfig)/u.test(name))
    return "star_states_copy_combinations";
  if (/^(?:Equipment|Equip|RoleRecommendEquip|EquipRecommendRole|Craft|Forge|SeasonCraft)/u
    .test(name))
    return "build_mappings_equipment_conversions";
  if (/^(?:Augment|Portal|Projection|Proj|Talent|Enhance|SelectEnhance|Orb|SeasonAugment|SeasonPortal|SeasonTalent|ModuleBanAugment|ModuleBanPortal)/u
    .test(name))
    return "investment_environment_strategy_persona";
  if (/(?:Mazebuff|MazeBuff)/u.test(name) || /^Affix/u.test(name))
    return "blessings_levels_formulas";
  if (/^(?:Pray|Present|AssistantMessage|TutorialTask)/u.test(name))
    return "events_variants_choices";
  if (/^(?:FuncManage|LotteryShop|ShopPrice|Consumables|Items|SpecialGoods|SeasonItem|GamePlayResource)/u
    .test(name))
    return "currencies_shops_services";
  if (/^(?:Monster|EliteGroup|EnemyDifficultyLv|FormationWave|Camp)/u
    .test(name))
    return "encounter_groups_waves_enemy_slots";
  return "mechanic_rules";
}
function evidenceOnlyTable(table) {
  return /(?:Old|Expired|AssistantMessage|GamePlayResource|GenderOverride|GuideQuest|GuideQuestGoToWiki|HandBookReward|OverrideRoleVO|ScoreReward|SkinCutin|TraitVideo)$/u
    .test(table);
}

const guideTabFile = "ExcelOutput/GuideRogueTab.json";
const guideDataFile = "ExcelOutput/GuideRogueData.json";
const guideTabRows = sourceRows(guideTabFile);
const guideDataRows = sourceRows(guideDataFile);
const guideTabIndex = guideTabRows.findIndex((row) =>
  row.ID === 1003 && row.GuideType === "GridFight");
const guideDataIndex = guideDataRows.findIndex((row) =>
  row.ID === 301 && row.TabID === 1003);
if (guideTabIndex !== 2 || guideDataIndex !== 5)
  throw new Error("fixed GridFight Guide selectors drifted");
for (const [file, index, label] of [
  [guideTabFile, guideTabIndex, "guide-tab:1003"],
  [guideDataFile, guideDataIndex, "guide-data:301"],
]) {
  const row = sourceRows(file)[index];
  add("profiles_gambit_entries_finish", {
    id: label,
    source: `${file}#${index}`,
    evidence_sha256: digest(row),
    evidence_quality: "ExactStructured",
    ownership: "CurrencyWars",
    reachability: "ExplicitModeSelector",
    upstream_identity: upstreamIdentity(row),
  });
}
for (const id of ["standard", "overclock"]) {
  add("profiles_gambit_entries_finish", {
    id: `gambit:${id}`,
    source: "content-manifests/currency-wars-v1/foundation.json",
    evidence_sha256: digest({
      id,
      source_boundary: "released Version 4.4 Currency Wars text",
    }),
    evidence_quality: "ProjectPolicy",
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  });
}

const tableInventory = inventory.records
  .filter(({ family }) => family === "currency_wars_gridfight_table")
  .sort((left, right) => compare(left.path, right.path));
const tableClosure = [];
let tableRowCount = 0;
for (const tableRecord of tableInventory) {
  const table = path.posix.basename(tableRecord.path, ".json");
  const group = groupFor(table);
  const rows = sourceRows(tableRecord.path);
  const evidenceOnly = evidenceOnlyTable(table);
  tableClosure.push({
    path: tableRecord.path,
    sha256: tableRecord.sha256,
    row_count: rows.length,
    counter_group: group,
    disposition: evidenceOnly ? "EvidenceOnly" : "CurrencyWars",
  });
  rows.forEach((row, index) => add(group, {
    id: `${table.toLowerCase()}:${String(index).padStart(6, "0")}`,
    source: `${tableRecord.path}#${index}`,
    evidence_sha256: digest(row),
    evidence_quality: "ExactStructured",
    ownership: evidenceOnly ? "EvidenceOnly" : "CurrencyWars",
    reachability: evidenceOnly
      ? (/Old|Expired/u.test(table) ? "ExcludedHistorical" : "ExcludedPresentation")
      : "DirectModeTable",
    table,
    row_index: index,
    upstream_identity: upstreamIdentity(row),
  }));
  tableRowCount += rows.length;
}

const configInventory = inventory.records
  .filter(({ family }) => family === "currency_wars_gridfight_config")
  .sort((left, right) => compare(left.path, right.path));
for (const record of configInventory)
  add("mechanic_rules", {
    id: `config:${record.path}`,
    source: record.path,
    evidence_sha256: record.sha256,
    evidence_quality: "ExactStructured",
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  });

for (const id of fixtureFamilies)
  add("semantic_fixtures", {
    id,
    source: "content-manifests/currency-wars-v1/source-correction.json",
    evidence_sha256: digest({ id, goal_id: "currency-wars-reference-v1" }),
    evidence_quality: "ProjectPolicy",
    ownership: "CurrencyWars",
    reachability: "SourceObligation",
  });

for (const category of Object.values(categories)) {
  category.records.sort((left, right) => compare(left.id, right.id));
  if (new Set(category.records.map(({ id }) => id)).size
      !== category.records.length)
    throw new Error(`duplicate IDs in category ${category.id}`);
  category.count = category.records.length;
}

const counterGroups = Object.fromEntries(Object.keys(categories).map((id) => [
  id,
  { id, categories: [id], required: categories[id].count },
]));
const ownershipCounts = {};
for (const category of Object.values(categories))
  for (const record of category.records)
    ownershipCounts[record.ownership] =
      (ownershipCounts[record.ownership] ?? 0) + 1;

const historicalBoundaryFiles = inventory.records
  .filter(({ family }) =>
    family.includes("divergent_universe")
      || family.startsWith("tourn_"))
  .map(({ path: sourcePath, sha256, family }) => ({
    source_path: sourcePath,
    evidence_sha256: sha256,
    disposition: "DivergentUniverseBoundary",
    inventory_family: family,
  }))
  .sort((left, right) => compare(left.source_path, right.source_path));

const payload = {
  schema_revision: "starclock.currency-wars-content-manifest.v2",
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
    source_correction: {
      path: "content-manifests/currency-wars-v1/source-correction.json",
      sha256: fileDigest(correctionPath),
    },
  },
  enabled_selector: {
    guide_type: correction.authoritative_selector.guide_type,
    guide_tab_id: correction.authoritative_selector.guide_tab_id,
    guide_data_id: correction.authoritative_selector.guide_data_id,
    selector_sources: correction.authoritative_selector.source_records,
  },
  ownership_policy: {
    permitted: ["CurrencyWars", "Shared", "EvidenceOnly"],
    direct_namespace_rule:
      "GuideType GridFight establishes the namespace; the GridFight string, table prefix, config path, ID range or matching name alone is insufficient.",
    fail_closed:
      "Shared content is promoted only by an explicit selector, typed transitive reference or stable-ID closure. Unresolved shared candidates remain outside the manifest denominator until a positive closure batch records them.",
    correction_policy:
      "Published Tourn3 artifacts remain immutable history and are replaced only by corrective batches in this branch.",
  },
  source_closure: {
    gridfight_tables: tableClosure,
    gridfight_configs: {
      count: configInventory.length,
      inventory_family: "currency_wars_gridfight_config",
      records_category: "mechanic_rules",
    },
    closed_absence_claims: [
      {
        family: "direct GridFight Curio/Miracle/Hex identities",
        result: "ProvenEmptyDirectNamespace",
        proof:
          "No row in the complete 153-table GridFight closure declares a Curio, Miracle or Hex identity; P2-B2 must separately close any shared stable-ID references.",
      },
      {
        family: "direct GridFight Blessing identities",
        result: "ProvenEmptyDirectNamespace",
        proof:
          "The complete GridFight closure contains MazeBuff bridges but no direct Blessing identity table; P2-B1 must resolve every referenced shared buff ID.",
      },
    ],
  },
  counter_groups: counterGroups,
  counts: {
    categories: Object.keys(categories).length,
    records: Object.values(categories).reduce(
      (sum, category) => sum + category.count, 0),
    gridfight_tables: tableClosure.length,
    gridfight_table_rows: tableRowCount,
    gridfight_configs: configInventory.length,
    ownership: Object.fromEntries(Object.entries(ownershipCounts).sort()),
    historical_boundary_files: historicalBoundaryFiles.length,
    reconciliation_conflicts: 0,
  },
  categories,
  exclusions: {
    historical_boundary_files: historicalBoundaryFiles,
  },
  reconciliation: [{
    goal: "Goal 11",
    remote: "origin",
    branch: "codex/goal11-divergent-universe-reference",
    commit: "982af8887fdd9ba29f1a323efc0ff5f6595ba411",
    manifest_path: "content-manifests/divergent-universe-v1/content-manifest.json",
    manifest_sha256: "5cbfa748406204e2d7a2c10c452ac6a87b3864b76461e85bd449c0739f3fc13e",
    state: "ResolvedDistinctSelector",
    outcome:
      "Goal 11 retains TournRogue/Tourn3/module 6002201; Goal 12 is selected by GuideType GridFight/tab 1003/data 301.",
    source_records: correction.authoritative_selector.source_records,
    audit_owner: "G12-P4-B3",
    replacement_condition:
      "Reopen only if a GridFight-originating typed reference reaches a row claimed exclusively by Goal 11.",
  }],
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
  `${payload.counts.records.toLocaleString("en-US")} obligations; ` +
  `${tableRowCount.toLocaleString("en-US")} exact GridFight rows, ` +
  `${configInventory.length.toLocaleString("en-US")} configs.`,
);
