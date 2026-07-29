#!/usr/bin/env node

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
const generatorArgs = [
  "tools/divergent-universe-reference/manifest.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
];
execFileSync(process.execPath, generatorArgs, { cwd: root, stdio: "inherit" });

const manifest = json(
  "content-manifests/divergent-universe-v1/content-manifest.json",
);
assert(
  manifest.schema_revision === "starclock.divergent-universe-content-manifest.v1",
  "unsupported Divergent Universe content manifest revision",
);
assert(
  manifest.goal_id === "divergent-universe-reference-v1"
    && manifest.profile === "divergent-universe-v1",
  "Divergent Universe manifest identity drift",
);
assert(
  manifest.snapshot.game_version === "4.4"
    && manifest.snapshot.source_revision
      === "fd978d6ef09f941fba644c731ab54abd6f7c3568"
    && manifest.snapshot.identity_revision
      === "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  "Divergent Universe manifest snapshot drift",
);
assert(
  JSON.stringify(manifest.enabled_module) === JSON.stringify({
    sub_mode: "TournRogue",
    tourn_mode: "Tourn3",
    activity_module_id: 6002201,
    main_tourn_id: 3,
    sub_tourn_id: 1,
  }),
  "enabled Version 4.4 module drift",
);

for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.count === category.records.length,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  assert(category.records.every((record) =>
    ["DivergentUniverse", "Shared", "SharedCandidate"].includes(record.ownership)
      && [
        "Direct",
        "DirectModeTable",
        "DirectModeConfig",
        "ExplicitModeSelector",
        "TransitiveReference",
        "PendingStageClosure",
        "SourceObligation",
      ].includes(record.reachability)
      && ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)),
  `${categoryId} contains an incomplete ownership/evidence record`);
}
assert(
  manifest.counts.categories === Object.keys(manifest.categories).length
    && manifest.counts.records === Object.values(manifest.categories).reduce(
      (sum, category) => sum + category.count,
      0,
    ),
  "aggregate denominator drift",
);
for (const [groupId, group] of Object.entries(manifest.counter_groups))
  assert(
    group.required === group.categories.reduce(
      (sum, categoryId) => sum + manifest.categories[categoryId].count,
      0,
    ),
    `${groupId} counter category sum drift`,
  );

assert(ids("entry_points").join(",") === "activity:105,title:TournRogue",
  "TournRogue entry-point selector drift");
assert(ids("enabled_modules").join(",") === "6002201",
  "enabled-module selector drift");
assert(ids("areas").length === 28, "Tourn3 area denominator drift");
assert(ids("difficulties").length === 22, "Tourn3 difficulty closure drift");
assert(ids("layers").length === 11, "Tourn3 layer closure drift");
assert(ids("layer_rooms").length === 0,
  "the source gained a directly matching Tourn3 layer-room row");
assert(ids("room_reuse_candidates").length === 848
  && records("room_reuse_candidates").every(
    ({ ownership, reachability }) =>
      ownership === "SharedCandidate" && reachability === "PendingStageClosure",
  ),
"room reuse was incorrectly promoted to proven reachability");
assert(ids("finish_conditions").length === 13,
  "Tourn3 finish-condition denominator drift");

const areaRows = sourceRows("ExcelOutput/RogueTournArea.json")
  .filter((row) => row.HILINOJPLGA === "Tourn3");
const difficultyIds = new Set(areaRows.flatMap((row) => row.EODCEHDOAEB).map(String));
const layerIds = new Set(areaRows.flatMap((row) => row.GLNDIILFKBN).map(String));
assert(setEqual(new Set(ids("difficulties")), difficultyIds),
  "difficulty category is not the Tourn3 area reference closure");
assert(setEqual(new Set(ids("layers")), layerIds),
  "layer category is not the Tourn3 area reference closure");

const useBuffType = sourceRows("ExcelOutput/RogueTournUseBuffType.json")
  .find((row) => row.TournMode === "Tourn3");
assert(useBuffType && setEqual(
  new Set(ids("blessing_paths")),
  new Set(useBuffType.UseBuffTypeList.map(String)),
), "Blessing Path/type closure drift");
const blessingLevelRows = sourceRows("ExcelOutput/RogueTournBuff.json")
  .filter((row) => useBuffType.UseBuffTypeList.includes(row.RogueBuffType));
assert(ids("blessings").length
  === new Set(blessingLevelRows.map((row) => row.MazeBuffID)).size,
"Blessing identity exact-once closure drift");
assert(ids("blessing_levels").length === blessingLevelRows.length,
  "Blessing level closure drift");
assert(records("equations").every(({ source }) => {
  const row = sourceRow(source);
  return row.TournMode === "Tourn3";
}), "Equation category contains a non-Tourn3 row");
assert(records("curio_states").every(({ source }) => {
  const row = sourceRow(source);
  return row.TournMode === "Tourn3";
}), "Curio state category contains a non-Tourn3 row");
assert(records("grand_miracles").every(({ source }) => {
  const row = sourceRow(source);
  return row.TournMode === "Tourn3";
}), "Grand Miracle category contains a non-Tourn3 row");

const occurrenceIds = new Set(ids("occurrence_variants"));
const serviceIds = new Set(ids("mode_service_npcs"));
const currentNpcIds = new Set(
  sourceRows("ExcelOutput/RogueTournNPC.json")
    .filter((row) => row.NPCJsonPath?.includes("RogueNPC_410"))
    .map((row) => String(row.RogueNPCID)),
);
assert(occurrenceIds.size + serviceIds.size === currentNpcIds.size
  && [...occurrenceIds, ...serviceIds].every((id) => currentNpcIds.has(id)),
"current NPC exact-once Occurrence/service partition drift");

const inventory = json(
  "content-manifests/divergent-universe-v1/source-inventory.json",
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
const mechanicPaths = inventory.records
  .filter(({ family }) => mechanicFamilies.has(family))
  .map(({ path: sourcePath }) => sourcePath)
  .sort(compare);
assert(JSON.stringify(ids("mechanic_source_files"))
  === JSON.stringify(mechanicPaths),
"mechanic source-file closure drift");
assert(
  manifest.ownership_policy.fail_closed.includes(
    "TournRogue/Tourn3/module-6002201",
  ),
  "manifest ownership policy is not fail-closed",
);
assert(
  manifest.exclusions.named_mode_source_files.every((record) =>
    record.ownership === "EvidenceOnly"
      && record.reachability === "Excluded"),
  "named mode exclusion classification drift",
);
assert(
  manifest.exclusions.historical_rows.every((record) =>
    record.ownership === "EvidenceOnly"
      && record.reachability === "Excluded"),
  "historical selector exclusion classification drift",
);

const ownership = {};
for (const category of Object.values(manifest.categories))
  for (const record of category.records)
    ownership[record.ownership] = (ownership[record.ownership] ?? 0) + 1;
assert(
  JSON.stringify(Object.fromEntries(Object.entries(ownership).sort()))
    === JSON.stringify(manifest.counts.ownership),
  "ownership aggregate drift",
);

console.log(
  `Divergent Universe content manifest verified ` +
  `(${manifest.counts.records.toLocaleString("en-US")} obligations; ` +
  `${manifest.counts.categories} categories; ` +
  `${manifest.counts.ownership.DivergentUniverse ?? 0} mode-owned, ` +
  `${manifest.counts.ownership.Shared ?? 0} shared and ` +
  `${manifest.counts.ownership.SharedCandidate ?? 0} fail-closed candidates).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function records(categoryId) {
  return manifest.categories[categoryId].records;
}

function ids(categoryId) {
  return records(categoryId).map(({ id }) => id).sort(compare);
}

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}

function sourceRows(relative) {
  return JSON.parse(fs.readFileSync(path.join(sourceRoot, relative), "utf8"));
}

function sourceRow(locator) {
  const separator = locator.lastIndexOf("#");
  const file = locator.slice(0, separator);
  const index = Number(locator.slice(separator + 1));
  return sourceRows(file)[index];
}

function unique(values) {
  return new Set(values).size === values.length;
}

function setEqual(left, right) {
  return left.size === right.size && [...left].every((value) => right.has(value));
}

function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
