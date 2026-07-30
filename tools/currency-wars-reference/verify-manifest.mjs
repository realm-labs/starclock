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
const inventory = json("content-manifests/currency-wars-v1/source-inventory.json");
assert(manifest.schema_revision === "starclock.currency-wars-content-manifest.v2",
  "unsupported corrected Currency Wars manifest revision");
assert(manifest.goal_id === "currency-wars-reference-v1"
  && manifest.profile === "currency-wars-v1",
"Currency Wars manifest identity drift");
assert(manifest.snapshot.game_version === "4.4"
  && manifest.snapshot.source_revision
    === "fd978d6ef09f941fba644c731ab54abd6f7c3568"
  && manifest.snapshot.identity_revision
    === "7b349e39ee0f6f3bf814567995829b99c95e7a93",
"Currency Wars manifest snapshot drift");
for (const input of Object.values(manifest.inputs))
  assert(input.sha256 === fileDigest(input.path),
    `manifest input digest drift: ${input.path}`);
assert(JSON.stringify(manifest.enabled_selector) === JSON.stringify({
  guide_type: "GridFight",
  guide_tab_id: 1003,
  guide_data_id: 301,
  selector_sources: [
    {
      path: "ExcelOutput/GuideRogueTab.json",
      locator: "2",
      sha256: "984f6e53d53424adb2962c19dbc0a6e1cd039adad2bba3393962f4339274a976",
      evidence_summary:
        "Guide tab 1003 explicitly sets GuideType to GridFight and names Currency Wars.",
    },
    {
      path: "ExcelOutput/GuideRogueData.json",
      locator: "5",
      sha256: "3162accac44c825d114b06c9b71f08f520a78c2dfcd91a272e99bfa1c341cb5e",
      evidence_summary:
        "Guide data 301 selects tab 1003 and names Currency Wars.",
    },
  ],
}), "GridFight selector drift");

const allowedOwnership = new Set(["CurrencyWars", "Shared", "EvidenceOnly"]);
const allowedReachability = new Set([
  "DirectModeTable",
  "ExplicitModeSelector",
  "SourceObligation",
  "ExcludedHistorical",
  "ExcludedPresentation",
]);
const allRecordIds = [];
const tableRowLocators = [];
for (const [categoryId, category] of Object.entries(manifest.categories)) {
  assert(category.id === categoryId && category.count === category.records.length,
    `${categoryId} denominator drift`);
  assert(unique(category.records.map(({ id }) => id)),
    `${categoryId} contains duplicate IDs`);
  for (const record of category.records) {
    assert(allowedOwnership.has(record.ownership)
      && allowedReachability.has(record.reachability)
      && ["ExactStructured", "ProjectPolicy"].includes(record.evidence_quality)
      && /^[0-9a-f]{64}$/u.test(record.evidence_sha256)
      && typeof record.source === "string",
    `${categoryId}/${record.id} has incomplete ownership/evidence`);
    allRecordIds.push(`${categoryId}:${record.id}`);
    if (/^ExcelOutput\/GridFight.*\.json#[0-9]+$/u.test(record.source))
      tableRowLocators.push(record.source);
  }
}
assert(unique(allRecordIds), "manifest contains duplicate category-record IDs");
assert(Object.keys(manifest.categories).length === 16
  && Object.keys(manifest.counter_groups).length === 16,
"corrected counter-group denominator drift");
for (const [groupId, group] of Object.entries(manifest.counter_groups))
  assert(group.categories.join(",") === groupId
    && group.required === manifest.categories[groupId].count,
  `${groupId} counter category sum drift`);

const tableInventory = inventory.records.filter(({ family }) =>
  family === "currency_wars_gridfight_table");
const configInventory = inventory.records.filter(({ family }) =>
  family === "currency_wars_gridfight_config");
assert(tableInventory.length === 153 && configInventory.length === 984,
  "GridFight inventory closure drift");
assert(manifest.source_closure.gridfight_tables.length === 153
  && manifest.source_closure.gridfight_configs.count === 984,
"GridFight manifest source closure drift");
let expectedRows = 0;
const expectedLocators = [];
for (const record of tableInventory) {
  const rows = sourceRows(record.path);
  expectedRows += rows.length;
  rows.forEach((_, index) => expectedLocators.push(`${record.path}#${index}`));
  const closure = manifest.source_closure.gridfight_tables.find(
    ({ path: sourcePath }) => sourcePath === record.path,
  );
  assert(closure
      && closure.sha256 === record.sha256
      && closure.row_count === rows.length,
  `GridFight table closure drift: ${record.path}`);
}
assert(expectedRows === 18234
  && manifest.counts.gridfight_table_rows === expectedRows
  && tableRowLocators.length === expectedRows
  && setEqual(new Set(tableRowLocators), new Set(expectedLocators)),
"GridFight table rows are not accounted exactly once");
const configRecords = manifest.categories.mechanic_rules.records
  .filter(({ id }) => id.startsWith("config:"));
assert(configRecords.length === 984
  && setEqual(new Set(configRecords.map(({ source }) => source)),
  new Set(configInventory.map(({ path: sourcePath }) => sourcePath))),
"GridFight config files are not accounted exactly once");

assert(manifest.counts.records === expectedRows + 984 + 2 + 2 + 28,
  "aggregate obligation denominator drift");
assert(manifest.counts.categories === Object.keys(manifest.categories).length
  && manifest.counts.reconciliation_conflicts === 0,
"aggregate count drift");
assert(manifest.source_closure.closed_absence_claims.length === 2
  && manifest.source_closure.closed_absence_claims.every(({ result }) =>
    result === "ProvenEmptyDirectNamespace"),
"direct empty-family closure drift");
assert(manifest.reconciliation.length === 1
  && manifest.reconciliation[0].goal === "Goal 11"
  && manifest.reconciliation[0].state === "ResolvedDistinctSelector"
  && manifest.reconciliation[0].commit
    === "982af8887fdd9ba29f1a323efc0ff5f6595ba411",
"Goal 11 selector-separation receipt drift");
assert(manifest.ownership_policy.direct_namespace_rule.includes("alone is insufficient")
  && manifest.ownership_policy.fail_closed.includes("typed transitive reference"),
"fail-closed ownership policy drift");

console.log(
  `Corrected Currency Wars manifest verified ` +
  `(${manifest.counts.records.toLocaleString("en-US")} obligations; ` +
  "18,234 exact GridFight rows; 984 config files; " +
  "Goal 11 selector resolved as distinct).",
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
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
function unique(values) {
  return new Set(values).size === values.length;
}
function setEqual(left, right) {
  return left.size === right.size && [...left].every((value) => right.has(value));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
