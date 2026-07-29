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
const rowsByFile = Object.fromEntries([
  "profiles.json",
  "gambit-modes.json",
  "modules.json",
  "entries.json",
  "finish-conditions.json",
  "area-groups.json",
  "areas.json",
  "difficulties.json",
  "layers.json",
  "rooms.json",
  "nodes.json",
  "domain-compositions.json",
  "stage-flow.json",
].map((file) => [file, json(path.join(outputRoot, file))]));
const expected = {
  "profiles.json": 1,
  "gambit-modes.json": 2,
  "modules.json": 1,
  "entries.json": 2,
  "finish-conditions.json": 13,
  "area-groups.json": 1,
  "areas.json": 28,
  "difficulties.json": 22,
  "layers.json": 11,
  "rooms.json": 0,
  "nodes.json": 60,
  "domain-compositions.json": 36,
  "stage-flow.json": 111,
};
for (const [file, count] of Object.entries(expected)) {
  const rows = rowsByFile[file];
  assert(rows.length === count, `${file} row count drift`);
  assert(unique(rows.map(({ id }) => id)), `${file} contains duplicate IDs`);
  assert(rows.every(validEnvelope), `${file} contains an invalid envelope`);
}

const manifest = json(path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
));
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
const categoryToFile = {
  entry_points: "entries.json",
  enabled_modules: "modules.json",
  finish_conditions: "finish-conditions.json",
  area_groups: "area-groups.json",
  areas: "areas.json",
  difficulties: "difficulties.json",
  layers: "layers.json",
  persona_layer_room: "nodes.json",
};
for (const [categoryId, file] of Object.entries(categoryToFile)) {
  const category = manifest.categories[categoryId];
  assert(category.count === rowsByFile[file].length,
    `${categoryId} manifest accounting drift`);
  const sourceRefs = new Map(rowsByFile[file].map((row) => {
    const exact = row.source_refs.find((ref) =>
      ref.repository.includes("turnbasedgamedata"));
    return [exact ? `${exact.path}#${exact.locator}` : "", exact?.sha256];
  }));
  for (const record of category.records)
    assert(sourceRefs.get(record.source) === record.evidence_sha256,
      `${categoryId}/${record.id} source receipt drift`);
}

const profile = rowsByFile["profiles.json"][0];
assert(profile.sub_mode === "TournRogue"
  && profile.tourn_mode === "Tourn3"
  && profile.name_en === "Currency Wars"
  && profile.name_zh_cn === "货币战争"
  && profile.module_id === "currency-wars.module.6002201"
  && profile.coverage_state === "Researched"
  && profile.runtime_enabled === false,
"profile module/runtime boundary drift");
assert(rowsByFile["gambit-modes.json"].map(({ mode_kind: kind }) => kind)
  .sort(compare).join(",") === "Overclock,Standard"
  && rowsByFile["gambit-modes.json"].every((row) =>
    row.evidence_quality === "ExactPublicText"
      && row.coverage_state === "Researched"
      && row.initial_resources_resolution === "DeferredToP1B3"),
"Currency Wars Gambit identity drift");
assert(rowsByFile["modules.json"][0].source_id === "6002201"
  && rowsByFile["modules.json"][0].main_tourn_id === 3
  && rowsByFile["modules.json"][0].sub_tourn_id === 1,
"enabled module row drift");
assert(rowsByFile["entries.json"].map(({ source_id }) => source_id)
  .sort(compare).join(",") === "105,TournRogue",
"entry exact-once selector drift");

const areas = rowsByFile["areas.json"];
const difficulties = new Set(rowsByFile["difficulties.json"].map(({ id }) => id));
const layers = new Set(rowsByFile["layers.json"].map(({ id }) => id));
assert(areas.every((area) =>
  area.difficulty_ids.every((id) => difficulties.has(id))
    && area.layer_ids.every((id) => layers.has(id))),
"area difficulty/layer reference closure drift");
assert(rowsByFile["difficulties.json"].every((row) =>
  row.coverage_state === "Researched"
    && row.enemy_scaling_resolution === "DeferredToP1B9"),
"difficulty deferred scaling boundary drift");
assert(new Set(areas.map(({ area_type }) => area_type)).size === 3,
  "Formal/WeekChallenge/Guide area-type boundary drift");
assert(areas.every((row) =>
  row.area_type === "Guide"
    ? row.gambit_mode_id === "" && row.gambit_binding_quality === "Tutorial"
    : row.gambit_mode_id === (row.area_type === "Formal"
      ? "currency-wars.gambit.standard"
      : "currency-wars.gambit.overclock")
      && row.gambit_binding_quality === "ProjectPolicy"),
"Gambit-to-area policy boundary drift");

const rooms = rowsByFile["rooms.json"];
assert(rooms.length === 0
  && manifest.categories.room_reuse_candidates.count === 848
  && manifest.categories.room_reuse_candidates.records.every((row) =>
    row.ownership === "EvidenceOnly"
      && row.reachability === "PendingStageClosure"),
"room candidate was promoted without exact stage/config evidence");

const nodes = rowsByFile["nodes.json"];
assert(nodes.every((node) =>
  layers.has(node.layer_id)
    && node.plane_number >= 1
    && node.plane_number <= 3
    && (node.domain_composition_id || node.room_pool_id)),
"Persona Node reference closure drift");
const nodeGroups = Object.groupBy(nodes, ({ layer_id: layerId }) => layerId);
assert(Object.keys(nodeGroups).length === 11
  && Object.values(nodeGroups).every((group) =>
    group.every((node, index) =>
      node.ordinal === index + 1
        && node.next_node_id === (group[index + 1]?.id ?? ""))),
"Persona Node ordering drift");
assert(rowsByFile["layers.json"].every((layer) =>
  JSON.stringify(layer.ordered_node_ids)
    === JSON.stringify(nodeGroups[layer.id].map(({ id }) => id))),
"layer-to-Node closure drift");

const domainCompositions = rowsByFile["domain-compositions.json"];
assert(domainCompositions.filter(({ selection_policy: policy }) =>
  policy === "FixedPreset").length === 34
  && domainCompositions.filter(({ domain_type: type }) =>
    type === "TypePool").length === 2,
"Persona domain-composition denominator drift");

const flowRows = rowsByFile["stage-flow.json"];
assert(flowRows.every((row) =>
  row.evidence_quality === "ProjectPolicy"
    && row.policy_id === "ordered-tourn3-area-layer-flow-v1"
    && row.source_refs.some((ref) => ref.replacement_condition)),
"stage flow lacks a replaceable policy boundary");
assert(flowRows.filter(({ id }) => id.includes(".terminal")).length === 28,
  "area terminal flow count drift");
assert(flowRows.filter(({ id }) => id.includes(".policy.")).length === 3,
  "carry/reset policy count drift");

const allRows = Object.values(rowsByFile).flat();
assert(allRows.every((row) => row.schema_revision
  === "starclock.currency-wars-row.v1"),
"row schema revision drift");
assert(allRows.every((row) => row.name_en !== row.name_zh_cn
  || /[^\x00-\x7F]/u.test(row.name_zh_cn)),
"bilingual authoring surface drift");

const encodedDigest = crypto.createHash("sha256");
for (const file of Object.keys(rowsByFile).sort(compare))
  encodedDigest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Currency Wars flow verified (${allRows.length.toLocaleString("en-US")} ` +
  `rows; digest ${encodedDigest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
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
