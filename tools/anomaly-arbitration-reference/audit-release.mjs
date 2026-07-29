#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const packRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
)));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
)));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/verify-pack.mjs"),
], { cwd: root, stdio: "inherit" });

const documents = new Map();
const stableRows = new Map();
let normalizedRows = 0;
for (const contract of schema.files) {
  const document = JSON.parse(await readFile(
    path.join(packRoot, contract.file),
  ));
  documents.set(contract.file, document);
  for (const row of document.records) {
    normalizedRows += 1;
    assert(!stableRows.has(row.id), `duplicate global stable key ${row.id}`);
    stableRows.set(row.id, { file: contract.file, row });
  }
}
assert(normalizedRows === 2103, "normalized row denominator drift");

const manifestRows = Object.entries(manifest.categories).flatMap(
  ([category, value]) => (value.records ?? []).map((row) => ({
    category,
    id: `${category}:${row.id}`,
    row,
  })),
);
const manifestIds = new Set(manifestRows.map(({ id }) => id));
assert(manifestIds.size === 392, "manifest exact-once denominator drift");
assert(manifest.counts.records === 392
  && manifest.counts.ownership.AnomalyArbitration === 76
  && manifest.counts.ownership.Shared === 316,
"manifest ownership counters drift");

const allowedOwnership = new Set(["AnomalyArbitration", "Shared"]);
const evidenceQuality = new Set([
  "ExactStructured",
  "ExactPublicText",
  "Observed",
  "ApproximateFromReleasedText",
  "ProjectPolicy",
]);
const mechanismQuality = new Set([
  "ExactProgram",
  "ExactRelationship",
  "ObservedBehavior",
  "IdentityCrossCheck",
  "PolicyBoundary",
  "ContextOnly",
]);
const sourceReceipts = new Set(
  documents.get("sources.json").records.map((row) => row.source_id),
);
for (const [stableKey, { file, row }] of stableRows) {
  assert(row.name_en.trim() && row.name_zh_cn.trim()
    && row.summary_en.trim() && row.summary_zh_cn.trim(),
  `${file}/${stableKey}: bilingual authoring fields drift`);
  assert(allowedOwnership.has(row.ownership),
    `${file}/${stableKey}: excluded/unknown ownership ${row.ownership}`);
  assert(evidenceQuality.has(row.evidence_quality)
    && mechanismQuality.has(row.mechanism_quality),
  `${file}/${stableKey}: evidence/mechanism quality drift`);
  assert(row.coverage_state === "DataReady" && !row.runtime_executable,
    `${file}/${stableKey}: readiness/runtime boundary drift`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${file}/${stableKey}: provenance missing`);
  for (const source of row.source_refs) {
    assert(sourceReceipts.has(source.source_id),
      `${file}/${stableKey}: unresolved source ${source.source_id}`);
  }
  assert(Array.isArray(row.manifest_record_ids),
    `${file}/${stableKey}: manifest projection missing`);
  for (const id of row.manifest_record_ids) {
    assert(manifestIds.has(id),
      `${file}/${stableKey}: unresolved manifest ID ${id}`);
  }
}

const selector = manifest.active_period_selector;
assert(selector.group_id === 8
  && selector.title_hash === "13457040013447238093"
  && selector.name_en === "Enwreathed by the World"
  && selector.name_zh === "尘世卷中",
"active-period selector drift");
const profile = documents.get("profiles.json").records[0];
const period = documents.get("periods.json").records[0];
const stages = documents.get("stages.json").records;
const expectedStages = [
  "stage.knight-1",
  "stage.knight-2",
  "stage.knight-3",
  "stage.king-normal",
  "stage.king-plight",
];
assert(profile.active_period_id === "period.8"
  && JSON.stringify(profile.stage_ids) === JSON.stringify(expectedStages)
  && period.id === "period.8"
  && period.source_group_id === "8"
  && JSON.stringify(period.alias_ids)
    === JSON.stringify(["801", "802", "803", "804"])
  && JSON.stringify(period.stage_ids)
    === JSON.stringify([
      "30508011", "30508012", "30508013", "30508021", "30508022",
    ])
  && JSON.stringify(stages.map((row) => row.id))
    === JSON.stringify(expectedStages)
  && JSON.stringify(stages.map((row) => row.source_stage_id))
    === JSON.stringify([
      "30508011", "30508012", "30508013", "30508021", "30508022",
    ]),
"active-period profile/period/stage closure drift");

const relationChecks = [
  ["team-slots.json", "stage_id", "stages.json"],
  ["stage-results.json", "stage_id", "stages.json"],
  ["encounters.json", "stage_id", "stages.json"],
  ["encounter-waves.json", "encounter_id", "encounters.json"],
  ["enemy-slots.json", "encounter_id", "encounters.json"],
  ["enemy-slots.json", "enemy_id", "enemies.json"],
  ["enemy-skills.json", "enemy_id", "enemies.json"],
];
for (const [file, field, targetFile] of relationChecks) {
  const targets = new Set(documents.get(targetFile).records.map(
    (row) => row.id,
  ));
  for (const row of documents.get(file).records) {
    assert(targets.has(row[field]),
      `${file}/${row.id}: unresolved ${field} ${row[field]}`);
  }
}
const enemyIds = new Set(documents.get("enemies.json").records.map(
  (row) => row.id,
));
for (const status of documents.get("enemy-statuses.json").records) {
  assert(status.enemy_id === "config-program-closure"
    || enemyIds.has(status.enemy_id),
  `${status.id}: unresolved status owner ${status.enemy_id}`);
}
const sources = new Set(documents.get("sources.json").records.map(
  (row) => row.id,
));
for (const fixture of documents.get("review-fixtures.json").records) {
  assert(fixture.evidence_refs.every((id) => sources.has(id)),
    `${fixture.id}: unresolved evidence reference`);
}
for (const coverage of documents.get("coverage.json").records) {
  assert(coverage.normalized_record_ids.every((id) => stableRows.has(id)),
    `${coverage.id}: unresolved normalized coverage reference`);
}

const excludedRows = [
  ...manifest.exclusions.historical_period_rows,
  ...manifest.exclusions.account_reward_rows,
  ...manifest.exclusions.excluded_constant_rows,
  ...manifest.exclusions.presentation_rows,
];
const excludedIds = new Set(excludedRows.map((row) => row.id));
assert(excludedIds.size === 106
  && excludedRows.every((row) => row.reachability === "Excluded"),
"excluded-row denominator/state drift");
assert([...stableRows.keys()].every((id) => !excludedIds.has(id)),
  "historical/account/presentation row leaked into normalized identity");
assert(manifestRows.every(({ row }) =>
  allowedOwnership.has(row.ownership)
    && !excludedIds.has(row.id)),
"excluded ownership/identity leaked into active manifest");

const sharedRows = manifestRows.filter(
  ({ row }) => row.ownership === "Shared",
);
const reconciliation = documents.get("reconciliation.json").records;
assert(sharedRows.length === 316 && reconciliation.length === 316
  && reconciliation.every((row) =>
    row.conflict_state === "None"
      && row.peer_match_state === "AbsentFromCommittedPeerManifest"),
"shared ownership/reconciliation drift");

console.log(
  "Anomaly Arbitration release ownership audit passed: "
    + "392 manifest rows (76 owned/316 shared), 2103 normalized rows, "
    + "824 sources, 106 exclusions and exact group-8 stage closure.",
);
