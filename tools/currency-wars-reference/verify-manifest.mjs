#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(valueAfter("--source-cache")
  ?? path.join(root, ".cache/content-reference/turnbasedgamedata"));
execFileSync(process.execPath, [
  "tools/currency-wars-reference/manifest.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const manifest = json("content-manifests/currency-wars-v1/content-manifest.json");
assert(manifest.schema_revision === "starclock.currency-wars-content-manifest.v1",
  "unsupported Currency Wars content manifest revision");
assert(manifest.goal_id === "currency-wars-reference-v1"
  && manifest.profile === "currency-wars-v1",
"Currency Wars manifest identity drift");
assert(manifest.snapshot.game_version === "4.4"
  && manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568"
  && manifest.snapshot.identity_revision
    === "7b349e39ee0f6f3bf814567995829b99c95e7a93",
"Currency Wars manifest snapshot drift");
assert(manifest.inputs.foundation.path
    === "content-manifests/currency-wars-v1/foundation.json"
  && manifest.inputs.foundation.sha256
    === fileDigest(manifest.inputs.foundation.path)
  && manifest.inputs.source_inventory.path
    === "content-manifests/currency-wars-v1/source-inventory.json"
  && manifest.inputs.source_inventory.sha256
    === fileDigest(manifest.inputs.source_inventory.path),
"Currency Wars manifest input digest drift");
assert(JSON.stringify(manifest.enabled_module) === JSON.stringify({
  activity_id: 105,
  sub_mode: "TournRogue",
  tourn_mode: "Tourn3",
  activity_module_id: 6002201,
  main_tourn_id: 3,
  sub_tourn_id: 1,
}), "enabled Currency Wars module drift");

const allowedOwnership = new Set(["CurrencyWars", "Shared", "EvidenceOnly"]);
const allowedReachability = new Set([
  "Direct",
  "DirectModeTable",
  "ExplicitModeSelector",
  "TransitiveReference",
  "PendingReferenceClosure",
  "PendingStageClosure",
  "SourceObligation",
]);
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.count === category.records.length,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  assert(category.records.every((record) =>
    allowedOwnership.has(record.ownership)
      && allowedReachability.has(record.reachability)
      && ["ExactStructured", "ExactPublicText", "ProjectPolicy"]
        .includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)
      && typeof record.source === "string"),
  `${categoryId} contains an incomplete ownership/evidence record`);
}
assert(manifest.counts.categories === Object.keys(manifest.categories).length
  && manifest.counts.records === Object.values(manifest.categories).reduce(
    (sum, category) => sum + category.count, 0),
"aggregate denominator drift");
for (const [groupId, group] of Object.entries(manifest.counter_groups))
  assert(group.required === group.categories.reduce(
    (sum, categoryId) => sum + manifest.categories[categoryId].count, 0),
  `${groupId} counter category sum drift`);
assert(Object.keys(manifest.counter_groups).length === 16,
  "status counter-group denominator drift");

assert(ids("profiles").length === 1
  && ids("gambit_modes").join(",") === "overclock,standard",
"profile/Gambit denominator drift");
assert(ids("entry_points").join(",") === "activity:105,title:TournRogue"
  && ids("enabled_modules").join(",") === "6002201",
"Currency Wars entry/module selector drift");
assert(ids("finish_conditions").length === 13,
  "Tourn3 finish-condition denominator drift");
assert(ids("areas").length === 28
  && ids("difficulties").length === 22
  && ids("layers").length === 11,
"Tourn3 area/difficulty/layer closure drift");
assert(ids("room_reuse_candidates").length === 848
  && records("room_reuse_candidates").every(({ ownership, reachability }) =>
    ownership === "EvidenceOnly" && reachability === "PendingStageClosure"),
"room candidates were incorrectly promoted");

const personaCategories = Object.keys(manifest.categories)
  .filter((id) => id.startsWith("persona_"));
assert(personaCategories.length === 11
  && personaCategories.reduce((sum, id) =>
    sum + manifest.categories[id].count, 0) === 547,
"Persona table/row denominator drift");
assert(ids("roster_avatars").length === 79
  && ids("role_mappings").length === 95
  && ids("build_reference_avatars").length === 84
  && ids("build_source_files").length === 6,
"roster/role/build source closure drift");

const useBuffType = sourceRows("ExcelOutput/RogueTournUseBuffType.json")
  .find((row) => row.TournMode === "Tourn3");
assert(useBuffType && setEqual(new Set(ids("blessing_paths")),
  new Set(useBuffType.UseBuffTypeList.map(String))),
"Tourn3 Blessing type closure drift");
assert(ids("blessings").length === 414
  && ids("blessing_levels").length === 828
  && ids("blessing_groups").length === 118
  && ids("formulas").length === 80,
"Blessing/Formula denominator drift");
assert(records("formulas").every(({ source }) =>
  sourceRow(source).TournMode === "Tourn3"),
"Formula category contains a non-Tourn3 row");
assert(ids("curio_states").length === 235
  && ids("curios").length === 179
  && ids("hex_states").length === 17,
"Curio/Hex denominator drift");
assert(records("curio_states").every(({ source }) =>
  sourceRow(source).TournMode === "Tourn3"),
"Curio state category contains a non-Tourn3 row");
assert(records("hex_states").every(({ source }) =>
  sourceRow(source).TournMode === "Tourn3"),
"Hex category contains a non-Tourn3 row");

assert(manifest.reconciliation.length === 1,
  "Goal 11 reconciliation conflict denominator drift");
const conflict = manifest.reconciliation[0];
assert(conflict.goal === "Goal 11"
  && conflict.commit === "982af8887fdd9ba29f1a323efc0ff5f6595ba411"
  && conflict.manifest_sha256
    === "5cbfa748406204e2d7a2c10c452ac6a87b3864b76461e85bd449c0739f3fc13e"
  && conflict.state === "ConflictPendingMergeCoordination"
  && conflict.source_records.length === 3
  && conflict.source_records.every(({ source, evidence_sha256: evidence }) =>
    typeof source === "string" && /^[0-9a-f]{64}$/u.test(evidence)),
"Goal 11 selector conflict receipt drift");
for (const group of Object.values(manifest.exclusions))
  assert(group.every((record) => record.ownership === "EvidenceOnly"
    && record.reachability === "Excluded"
    && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)),
  "exclusion ownership/evidence drift");

const ownership = {};
for (const category of Object.values(manifest.categories))
  for (const record of category.records)
    ownership[record.ownership] = (ownership[record.ownership] ?? 0) + 1;
assert(JSON.stringify(Object.fromEntries(Object.entries(ownership).sort()))
  === JSON.stringify(manifest.counts.ownership),
"ownership aggregate drift");
assert(manifest.ownership_policy.fail_closed.includes("explicit enabled selector")
  && manifest.ownership_policy.conflict_policy.includes("Goal 11"),
"fail-closed ownership policy drift");

console.log(
  `Currency Wars content manifest verified ` +
  `(${manifest.counts.records.toLocaleString("en-US")} obligations; ` +
  `${manifest.counts.categories} categories; ` +
  `${manifest.counts.ownership.CurrencyWars ?? 0} mode-owned, ` +
  `${manifest.counts.ownership.Shared ?? 0} shared and ` +
  `${manifest.counts.ownership.EvidenceOnly ?? 0} fail-closed evidence; ` +
  "1 recorded Goal 11 selector conflict).",
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
function fileDigest(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative))).digest("hex");
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
  return left.size === right.size
    && [...left].every((value) => right.has(value));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
