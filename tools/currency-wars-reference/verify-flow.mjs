#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/currency-wars-reference/import-flow.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/currency-wars-v1");
const expected = {
  "profiles.json": 1,
  "gambit-modes.json": 2,
  "modules.json": 4,
  "entries.json": 2,
  "finish-conditions.json": 135,
  "area-groups.json": 1,
  "areas.json": 26,
  "difficulties.json": 97,
  "layers.json": 75,
  "rooms.json": 5,
  "nodes.json": 493,
  "domain-compositions.json": 5,
  "stage-flow.json": 493,
};
const rowsByFile = Object.fromEntries(Object.keys(expected)
  .map((file) => [file, json(path.join(outputRoot, file))]));
for (const [file, count] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  assert(rows.length === count, `${file} row count drift`);
  assert(unique(rows.map(({ id }) => id)), `${file} contains duplicate IDs`);
  assert(rows.every(validEnvelope), `${file} contains an invalid envelope`);
}

const normalizedSchema = json(path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
));
for (const [file, rows] of Object.entries(rowsByFile)) {
  const fileContract = normalizedSchema.files.find((entry) => entry.file === file);
  assert(fileContract, `${file} is missing from the normalized schema`);
  for (const row of rows)
    assert(fileContract.required_domain_fields.every((field) =>
      Object.hasOwn(row, field)),
    `${file}/${row.id} lacks a required domain field`);
}

const profile = rowsByFile["profiles.json"][0];
assert(profile.sub_mode === "GridFight"
  && profile.tourn_mode === ""
  && profile.guide_tab_id === "1003"
  && profile.guide_data_id === "301"
  && profile.name_en === "Currency Wars"
  && profile.name_zh_cn === "货币战争"
  && profile.module_id === "currency-wars.module.7100501"
  && profile.module_ids.length === 4
  && profile.coverage_state === "Researched"
  && profile.runtime_enabled === false,
"GridFight profile/module boundary drift");
assert(rowsByFile["gambit-modes.json"].map(({ mode_kind: kind }) => kind)
  .sort(compare).join(",") === "Overclock,Standard"
  && rowsByFile["gambit-modes.json"].every((row) =>
    row.evidence_quality === "ExactPublicText"
      && row.coverage_state === "Researched"
      && row.initial_resources_resolution === "DeferredToP1B3"),
"Currency Wars Gambit identity drift");
assert(rowsByFile["modules.json"].map(({ source_id: id }) => id).join(",")
  === "7100201,7100301,7100401,7100501"
  && rowsByFile["modules.json"].every((row, index) =>
    row.sub_mode === "GridFight"
      && row.tourn_mode === ""
      && row.main_tourn_id === 1
      && row.sub_tourn_id === index + 1),
"GridFight season-module closure drift");
assert(rowsByFile["entries.json"].map(({ source_id }) => source_id)
  .sort(compare).join(",") === "1003,301"
  && rowsByFile["entries.json"].every((row) =>
    row.source_refs.some((ref) =>
      ref.path === "ExcelOutput/GuideRogueTab.json"
        || ref.path === "ExcelOutput/GuideRogueData.json")),
"GridFight Guide entry closure drift");

const finish = rowsByFile["finish-conditions.json"];
assert(finish.filter(({ condition_kind: kind }) =>
  kind === "BattleStageRule").length === 15
  && finish.filter(({ condition_kind: kind }) =>
    kind === "BattlePenaltyRule").length === 114
  && finish.filter(({ condition_kind: kind }) =>
    kind === "SettlementRank").length === 6,
"GridFight Stage/settlement terminal closure drift");
assert(sourceLocatorSet(finish, "ExcelOutput/GridFightStage.json").size === 15
  && sourceLocatorSet(finish, "ExcelOutput/GridFightPenaltyRule.json").size === 114
  && sourceLocatorSet(finish, "ExcelOutput/GridFightSettleRank.json").size === 6,
"GridFight terminal source exact-once drift");

const areas = rowsByFile["areas.json"];
const layers = rowsByFile["layers.json"];
const nodes = rowsByFile["nodes.json"];
const areaIds = new Set(areas.map(({ id }) => id));
const layerIds = new Set(layers.map(({ id }) => id));
const nodeIds = new Set(nodes.map(({ id }) => id));
assert(rowsByFile["area-groups.json"][0].area_ids.length === 26
  && rowsByFile["area-groups.json"][0].area_ids.every((id) => areaIds.has(id)),
"GridFight route-group closure drift");
assert(areas.every((area) =>
  area.area_type === "StageRoute"
    && area.gambit_binding_quality === "Unresolved"
    && area.difficulty_resolution === "DivisionStageSeparateAxis"
    && area.layer_ids.every((id) => layerIds.has(id))),
"GridFight route-area boundary drift");
assert(layers.every((layer) =>
  [1, 2, 3].includes(layer.layer_number)
    && layer.ordered_node_ids.every((id) => nodeIds.has(id))),
"GridFight three-Plane layer closure drift");
const planeCounts = Object.fromEntries([1, 2, 3].map((plane) => [
  plane,
  layers.filter(({ layer_number: value }) => value === plane).length,
]));
assert(JSON.stringify(planeCounts) === JSON.stringify({
  1: 26,
  2: 25,
  3: 24,
}), "GridFight three-Plane denominator drift");

const difficulties = rowsByFile["difficulties.json"];
assert(difficulties.length === 97
  && sourceLocatorSet(difficulties,
    "ExcelOutput/GridFightDivisionInfo.json").size === 97
  && sourceLocatorSet(difficulties,
    "ExcelOutput/GridFightDivisionStage.json").size === 97
  && difficulties.every((row) =>
    Number.isInteger(Number(row.rank_bounds.division_level))
      && Number(row.rank_bounds.division_level) >= 1
      && Number(row.rank_bounds.division_level) <= 9
      && row.gambit_rules.standard_score_rule
      && row.gambit_rules.overclock_score_rule),
"GridFight Division difficulty closure drift");

const rooms = rowsByFile["rooms.json"];
const compositions = rowsByFile["domain-compositions.json"];
const nodeTypes = ["Boss", "CampMonster", "EliteBranch", "Monster", "Supply"];
assert(rooms.map(({ room_type: type }) => type).sort(compare).join(",")
  === nodeTypes.sort(compare).join(",")
  && compositions.map(({ domain_type: type }) => type)
    .sort(compare).join(",") === nodeTypes.sort(compare).join(","),
"GridFight NodeType closure drift");
assert(nodes.every((node) =>
  node.plane_id === `currency-wars.plane.${node.ordinal > 0
    ? node.layer_id.match(/chapter\.(\d+)$/u)?.[1]
    : ""}`
    && layerIds.has(node.layer_id)
    && node.domain_composition_id
    && node.room_pool_id
    && node.stage_id
    && node.penalty_bonus_rule_id),
"GridFight Node reference closure drift");
assert(sourceLocatorSet(nodes, "ExcelOutput/GridFightStageRoute.json").size === 493
  && sourceLocatorSet(nodes,
    "ExcelOutput/GridFightNodeTemplate.json").size === 493,
"GridFight route/template rows are not imported exactly once");

const flow = rowsByFile["stage-flow.json"];
assert(flow.length === 493
  && sourceLocatorSet(flow, "ExcelOutput/GridFightStageRoute.json").size === 493
  && flow.every((row) =>
    row.evidence_quality === "ProjectPolicy"
      && row.coverage_state === "Researched"
      && row.carry_rules.length === 0
      && row.reset_rules.length === 0
      && row.lifecycle_resolution === "UnspecifiedByStageRoute"
      && row.ordered_node_refs.every((id) => nodeIds.has(id))),
"GridFight StageRoute transition/lifecycle boundary drift");
for (const layer of layers) {
  const group = layer.ordered_node_ids.map((id) =>
    nodes.find((node) => node.id === id));
  assert(group.every(Boolean)
    && group.every((node, index) =>
      node.ordinal === index + 1
        && node.next_node_id === (group[index + 1]?.id ?? "")),
  `${layer.id} authored Node order drift`);
}

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.schema_revision
  === "starclock.currency-wars-row.v1"),
"row schema revision drift");
assert(allRows.every((row) => !row.id.includes("tourn")
  && !row.id.includes("persona")
  && row.source_refs.every((ref) =>
    !ref.path.includes("RogueTourn") && !ref.path.includes("RoguePersona"))),
"superseded Tourn/Persona source escaped into corrected flow");
assert(allRows.every((row) => row.name_en !== row.name_zh_cn
  || /[^\x00-\x7F]/u.test(row.name_zh_cn)
  || row.tags.includes("settle-rank")),
"bilingual authoring surface drift");

const digest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars GridFight flow verified (${allRows.length.toLocaleString("en-US")} ` +
  `rows; 26 routes, 75 Plane layers, 493 Nodes; digest ` +
  `${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}
function sourceLocatorSet(rows, sourcePath) {
  return new Set(rows.flatMap(({ source_refs: refs }) =>
    refs.filter(({ path: refPath }) => refPath === sourcePath)
      .map(({ locator }) => locator)));
}
function validEnvelope(row) {
  return row
    && /^[a-z0-9][a-z0-9._:-]*$/u.test(row.id)
    && row.name_en
    && row.name_zh_cn
    && row.summary_en
    && row.summary_zh_cn
    && ["CurrencyWars", "Shared"].includes(row.ownership)
    && ["Cataloged", "Researched", "DataReady", "Blocked"].includes(
      row.coverage_state,
    )
    && Array.isArray(row.source_refs)
    && row.source_refs.length > 0
    && row.source_refs.every((ref) =>
      /^[0-9a-f]{64}$/u.test(ref.sha256)
        && ref.revision
        && ref.path
        && ref.locator !== undefined)
    && Array.isArray(row.tags)
    && JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort(compare));
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
