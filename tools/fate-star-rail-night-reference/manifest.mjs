#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = argument("--source-cache");
const check = args.includes("--check");
const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const output = path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json");
const inventory = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/source-inventory.json"));

const directTableFiles = inventory.records.filter(({ repository, path: relative }) =>
  repository === "turnbasedgamedata" && /^ExcelOutput\/Fate.*\.json$/u.test(relative));
const directConfigFiles = inventory.records.filter(({ repository, path: relative }) =>
  repository === "turnbasedgamedata" && relative.startsWith("Config/") &&
    (relative.startsWith("Config/Gameplays/Fate/") || relative.includes("FateRin")));

const obligations = [];
for (const file of directTableFiles) {
  const rows = json(path.join(turnRoot, file.path));
  rows.forEach((row, index) => obligations.push(tableObligation(file, row, index)));
}
for (const file of directConfigFiles)
  obligations.push(configObligation(file));

const directValues = new Set();
for (const file of directTableFiles)
  collectSafeIntegers(json(path.join(turnRoot, file.path)), directValues);

const stageRows = sourceRows("ExcelOutput/StageConfig.json");
const fateStages = stageRows.filter(({ row }) => row.StageType === "FateActivity");
for (const selected of fateStages)
  obligations.push(sharedObligation(selected, "Stage", "ExactStageTypeSelector"));

const monsterIds = new Set();
for (const { row } of fateStages) collectNamedIntegers(row.MonsterList, /^Monster\d+$/u, monsterIds);
const monsterRows = sourceRows("ExcelOutput/MonsterConfig.json")
  .filter(({ row }) => monsterIds.has(row.MonsterID));
for (const selected of monsterRows)
  obligations.push(sharedObligation(selected, "EnemyVariant", "FateStageMonsterReference"));

const templateIds = new Set(monsterRows.map(({ row }) => row.MonsterTemplateID));
const templateRows = sourceRows("ExcelOutput/MonsterTemplateConfig.json")
  .filter(({ row }) => templateIds.has(row.MonsterTemplateID));
for (const selected of templateRows)
  obligations.push(sharedObligation(selected, "EnemyTemplate", "EnemyVariantTemplateReference"));

const skillIds = new Set();
for (const { row } of templateRows) collectSafeIntegers(row.AISkillSequence, skillIds);
const skillRows = sourceRows("ExcelOutput/MonsterSkillConfig.json")
  .filter(({ row }) => skillIds.has(row.SkillID));
for (const selected of skillRows)
  obligations.push(sharedObligation(selected, "EnemySkill", "EnemyTemplateSkillReference"));

for (const table of [
  ["ExcelOutput/BattleArea.json", "ID", "BattleArea"],
  ["ExcelOutput/BattleAreaUnifiedConfig.json", "ID", "BattleAreaConfig"],
  ["ExcelOutput/MazeBuff.json", "ID", "MazeBuff"],
  ["ExcelOutput/BattleTargetConfig.json", "ID", "BattleTarget"],
  ["ExcelOutput/BattleEventConfig.json", "BattleEventID", "BattleEvent"],
  ["ExcelOutput/MonsterStatusConfig.json", "StatusID", "EnemyStatus"],
]) {
  const [relative, idField, family] = table;
  for (const selected of sourceRows(relative).filter(({ row }) =>
    Number.isSafeInteger(row[idField]) && row[idField] >= 1000 &&
      directValues.has(row[idField])))
    obligations.push(sharedObligation(selected, family, "DirectScalarReference",
      family === "BattleEvent" || family === "BattleTarget" ? "ResearchRequired" : "DataReady"));
}

const zeroFamilies = ["Blessing", "Curio", "Occurrence", "Shop", "Service", "RunCurrency"];
for (const family of zeroFamilies)
  obligations.push({
    obligation_id: `zero-${slug(family)}`,
    family: "ExactZeroPool",
    source_family: family,
    ownership: "FateStarRailNight",
    disposition: "DataReady",
    locator: `selector-closure:${family}`,
    source_path: "content-manifests/fate-star-rail-night-v1/source-inventory.json",
    source_sha256: inventory.canonical_records_sha256,
    evidence_quality: "ExactStructured",
    note: "No dedicated Fate/FateRin table, Fate gameplay config or selected transitive shared row exposes this generic family.",
  });

obligations.sort((left, right) => compareText(left.obligation_id, right.obligation_id));
assert(new Set(obligations.map(({ obligation_id: id }) => id)).size === obligations.length,
  "duplicate manifest obligation id");

const counts = {
  obligations: obligations.length,
  ownership: countBy(obligations, "ownership"),
  disposition: countBy(obligations, "disposition"),
  family: countBy(obligations, "family"),
};
const document = {
  schema_revision: "starclock.fate-star-rail-night-content-manifest.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P0-B3",
  snapshot: { game_version: "4.4", access_date: "2026-08-01" },
  selector_contract: {
    direct_tables: "ExcelOutput/Fate*.json",
    direct_configs: ["Config/Gameplays/Fate/", "Config/**/*FateRin*.json"],
    stage_selector: "StageConfig.StageType == FateActivity",
    transitive_reference_policy: "exact scalar ID or typed Stage-to-enemy reference",
    prefix_alone_is_membership: false,
    inventory_is_denominator: false,
  },
  counts,
  exact_zero_families: zeroFamilies,
  obligations,
};
document.canonical_obligations_sha256 = digest(`${JSON.stringify(obligations)}\n`);

const serialized = `${JSON.stringify(document, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(output), `missing ${path.relative(root, output)}`);
  assert(fs.readFileSync(output, "utf8") === serialized,
    "Goal 19 content manifest drift");
  console.log(summary("verified"));
} else {
  fs.writeFileSync(output, serialized);
  console.log(summary("wrote"));
}

function tableObligation(file, row, index) {
  const evidenceOnlyTables = new Set([
    "FateAvatarDescription.json", "FateBroadcast.json", "FateMasterTalk.json",
    "FateMiscDisplay.json", "FateRinMainMissions.json",
    "FateRinResidentReward.json", "FateRinSwitchDayTalk.json",
  ]);
  const base = path.basename(file.path);
  const evidenceOnly = evidenceOnlyTables.has(base);
  return {
    obligation_id: `row-${slug(base.replace(/\.json$/u, ""))}-${pad(index)}`,
    family: base.replace(/\.json$/u, ""),
    ownership: evidenceOnly ? "EvidenceOnly" : "FateStarRailNight",
    disposition: evidenceOnly ? "EvidenceOnly" : "DataReady",
    locator: `index:${index}`,
    source_path: file.path,
    source_sha256: file.sha256,
    evidence_quality: "ExactStructured",
    top_level_key_count: row && typeof row === "object" ? Object.keys(row).length : 0,
  };
}

function configObligation(file) {
  const layout = file.path.endsWith(".layout.json");
  return {
    obligation_id: `config-${slug(file.path)}`,
    family: file.path.startsWith("Config/Gameplays/Fate/")
      ? "FateGameplayConfig" : "FateFocusedLayout",
    ownership: layout ? "EvidenceOnly" : "FateStarRailNight",
    disposition: layout ? "EvidenceOnly" : "DataReady",
    locator: "file",
    source_path: file.path,
    source_sha256: file.sha256,
    evidence_quality: "ExactStructured",
    note: layout ? "Layout-only source retains identity but not executable mechanic semantics." : "",
  };
}

function sharedObligation(selected, family, relation, disposition = "DataReady") {
  return {
    obligation_id: `shared-${slug(family)}-${pad(selected.index)}`,
    family,
    ownership: "Shared",
    disposition,
    locator: `index:${selected.index}`,
    source_path: selected.path,
    source_sha256: selected.sha256,
    evidence_quality: "ExactStructured",
    relation,
  };
}

function sourceRows(relative) {
  const absolute = path.join(turnRoot, relative);
  const bytes = fs.readFileSync(absolute);
  const sha256 = digest(bytes);
  return JSON.parse(bytes.toString("utf8")).map((row, index) => ({
    path: relative, sha256, index, row,
  }));
}

function collectSafeIntegers(value, target) {
  if (Number.isSafeInteger(value)) target.add(value);
  else if (Array.isArray(value)) for (const entry of value) collectSafeIntegers(entry, target);
  else if (value && typeof value === "object")
    for (const entry of Object.values(value)) collectSafeIntegers(entry, target);
}

function collectNamedIntegers(value, keyPattern, target) {
  if (Array.isArray(value)) for (const entry of value) collectNamedIntegers(entry, keyPattern, target);
  else if (value && typeof value === "object")
    for (const [key, entry] of Object.entries(value)) {
      if (keyPattern.test(key) && Number.isSafeInteger(entry)) target.add(entry);
      collectNamedIntegers(entry, keyPattern, target);
    }
}

function countBy(rows, field) {
  return Object.fromEntries([...new Set(rows.map((row) => row[field]))]
    .sort(compareText).map((value) => [value,
      rows.filter((row) => row[field] === value).length]));
}

function json(absolute) {
  return JSON.parse(fs.readFileSync(absolute, "utf8"));
}

function argument(name) {
  const index = args.indexOf(name);
  assert(index !== -1 && args[index + 1], `${name} requires a value`);
  return path.resolve(args[index + 1]);
}

function slug(value) {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2")
    .replace(/[^A-Za-z0-9]+/gu, "-").replace(/^-|-$/gu, "").toLowerCase();
}

function pad(value) {
  return String(value).padStart(6, "0");
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function summary(verb) {
  return `Goal 19 content manifest ${verb} (${counts.obligations} obligations, ` +
    `${document.canonical_obligations_sha256}).`;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
