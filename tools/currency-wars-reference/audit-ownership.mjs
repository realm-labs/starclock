#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { canonical, sha256 } from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const manifestPath = path.join(
  root,
  "content-manifests/currency-wars-v1/content-manifest.json",
);
const schemaPath = path.join(
  root,
  "content-manifests/currency-wars-v1/normalized-schema.json",
);
const referenceRoot = path.join(root, "content-reference/currency-wars-v1");
const outputPath = path.join(
  root,
  "evidence/currency-wars-reference-v1/p4b1-ownership-audit.json",
);
const reconciliationPath = path.join(
  referenceRoot,
  "reconciliation-receipts.json",
);
if (!fs.existsSync(reconciliationPath) && !check)
  fs.writeFileSync(reconciliationPath, "[]\n");
const manifest = json(manifestPath);
const schema = json(schemaPath);
const allowedOwnership = new Set(schema.common_envelope.ownership.enum);
const allowedCoverage = new Set(schema.common_envelope.coverage_state.enum);
const allowedQuality = new Set(schema.common_envelope.evidence_quality.enum);
const idPattern = new RegExp(schema.common_envelope.id.pattern, "u");
const rowsByFile = new Map();
const allRows = [];

for (const contract of schema.files) {
  const target = path.join(referenceRoot, contract.file);
  if (!fs.existsSync(target)) {
    throw new Error(`missing normalized file ${contract.file}`);
  }
  const rows = json(target);
  assert(Array.isArray(rows), `${contract.file} must be an array`);
  rowsByFile.set(contract.file, rows);
  for (const row of rows) {
    for (const field of schema.common_envelope.required_fields)
      assert(Object.hasOwn(row, field), `${contract.file}/${row.id}: ${field}`);
    for (const field of contract.required_domain_fields)
      assert(Object.hasOwn(row, field), `${contract.file}/${row.id}: ${field}`);
    assert(/^CurrencyWars[A-Za-z0-9]+$/u.test(row.kind),
      `${contract.file}/${row.id}: record kind drift`);
    assert(row.schema_revision === schema.common_envelope.schema_revision.value,
      `${contract.file}/${row.id}: schema revision drift`);
    assert(idPattern.test(row.id), `${contract.file}/${row.id}: invalid ID`);
    assert(allowedOwnership.has(row.ownership),
      `${contract.file}/${row.id}: invalid ownership`);
    assert(allowedCoverage.has(row.coverage_state),
      `${contract.file}/${row.id}: invalid coverage state`);
    assert(allowedQuality.has(row.evidence_quality),
      `${contract.file}/${row.id}: invalid evidence quality`);
    for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"]) {
      assert(typeof row[field] === "string" && row[field].trim(),
        `${contract.file}/${row.id}: empty ${field}`);
      assert(row[field] === row[field].normalize("NFC"),
        `${contract.file}/${row.id}: non-NFC ${field}`);
    }
    assert(row.summary_en !== row.summary_zh_cn,
      `${contract.file}/${row.id}: summaries are not independent`);
    assert(Array.isArray(row.tags)
      && new Set(row.tags).size === row.tags.length
      && JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${contract.file}/${row.id}: tag order/uniqueness drift`);
    assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
      `${contract.file}/${row.id}: missing provenance`);
    allRows.push({ file: contract.file, row });
  }
}

const ids = allRows.map(({ row }) => row.id);
assert(new Set(ids).size === ids.length, "global normalized ID collision");
const idSet = new Set(ids);
const sources = rowsByFile.get("sources.json");
const sourceKeys = new Set(sources.map((row) => sourceKey(row)));
assert(sourceKeys.size === sources.length, "duplicate canonical source receipt");
let referenceCount = 0;
let policyReferenceCount = 0;
const repositoryCounts = {};
for (const { file, row } of allRows) {
  for (const reference of row.source_refs) {
    referenceCount++;
    for (const field of [
      "repository",
      "revision",
      "path",
      "locator",
      "sha256",
      "access_date",
      "game_version",
      "evidence_quality",
      "mechanism_quality",
    ])
      assert(typeof reference[field] === "string" && reference[field],
        `${file}/${row.id}: incomplete source reference ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(reference.sha256),
      `${file}/${row.id}: invalid evidence digest`);
    assert(reference.game_version === "4.4",
      `${file}/${row.id}: non-4.4 evidence`);
    assert(allowedQuality.has(reference.evidence_quality),
      `${file}/${row.id}: invalid reference quality`);
    assert(sourceKeys.has(sourceKey(reference)),
      `${file}/${row.id}: unresolved source receipt`);
    assert(!/(^|\/)Rogue(?:Persona|Tourn)[^/]*\.json$/u.test(reference.path),
      `${file}/${row.id}: other-mode source leak ${reference.path}`);
    if (reference.repository.includes("turnbasedgamedata"))
      assert(reference.revision
        === "fd978d6ef09f941fba644c731ab54abd6f7c3568",
      `${file}/${row.id}: structured snapshot drift`);
    else if (reference.repository.includes("StarRailRes"))
      assert(reference.revision
        === "7b349e39ee0f6f3bf814567995829b99c95e7a93",
      `${file}/${row.id}: identity snapshot drift`);
    else
      assert(reference.repository === "starclock",
        `${file}/${row.id}: unapproved evidence repository`);
    if (reference.evidence_quality === "ProjectPolicy") {
      policyReferenceCount++;
      assert(reference.mechanism_quality === "PolicyBound"
        && typeof reference.note === "string"
        && reference.note
        && typeof reference.replacement_condition === "string"
        && reference.replacement_condition,
      `${file}/${row.id}: unbounded project policy`);
    }
  }
}

const manifestRows = [];
const manifestKeys = new Set();
const manifestOwnership = {};
const reachability = {};
for (const [category, group] of Object.entries(manifest.categories)) {
  assert(group.id === category && group.count === group.records.length,
    `${category}: manifest group count drift`);
  for (const record of group.records) {
    const key = `${category}\0${record.id}`;
    assert(!manifestKeys.has(key), `${category}/${record.id}: duplicate`);
    manifestKeys.add(key);
    assert(["CurrencyWars", "Shared", "EvidenceOnly"].includes(record.ownership),
      `${category}/${record.id}: invalid manifest ownership`);
    manifestOwnership[record.ownership] =
      (manifestOwnership[record.ownership] ?? 0) + 1;
    reachability[record.reachability] =
      (reachability[record.reachability] ?? 0) + 1;
    manifestRows.push({ category, record });
  }
}
assert(manifest.enabled_selector.guide_type === "GridFight"
  && manifest.enabled_selector.guide_tab_id === 1003
  && manifest.enabled_selector.guide_data_id === 301,
"enabled selector drift");

const coverage = rowsByFile.get("coverage.json");
assert(coverage.length === manifestRows.length
  && coverage.length === 19250, "coverage denominator drift");
const coverageByKey = new Map(coverage.map((row) => [
  `${row.manifest_category}\0${row.manifest_record_id}`,
  row,
]));
assert(coverageByKey.size === coverage.length, "duplicate coverage key");
for (const { category, record } of manifestRows) {
  const row = coverageByKey.get(`${category}\0${record.id}`);
  assert(row, `${category}/${record.id}: missing coverage`);
  const expectedState =
    record.ownership === "EvidenceOnly" ? "Excluded" : "DataReady";
  assert(row.state === expectedState,
    `${category}/${record.id}: coverage disposition drift`);
  assert(row.normalized_record_ids.length > 0,
    `${category}/${record.id}: no normalized accounting`);
  for (const id of row.normalized_record_ids)
    assert(idSet.has(id), `${category}/${record.id}: unresolved normalized ID`);
  if (record.ownership === "EvidenceOnly")
    assert(row.normalized_record_ids.every((id) =>
      id.startsWith("currency-wars.source.")),
    `${category}/${record.id}: excluded content promotion`);
}

assert(allRows.every(({ row }) =>
  !hasTruthyRuntimeFlag(row)), "runtime lowering/enabling leak");
const packIndex = rowsByFile.get("pack-index.json");
assert(packIndex.length > 0
  && packIndex.every(({ pack_digest: digest }) =>
    digest === packIndex[0].pack_digest),
"pack-index digest drift");
const forbiddenTags = new Set([
  "beta",
  "leak",
  "leaked",
  "preview",
  "test-server",
  "unreleased",
]);
assert(allRows.every(({ row }) =>
  row.tags.every((tag) => !forbiddenTags.has(tag.toLowerCase()))),
"unreleased evidence tag detected");

const report = {
  batch: "G12-P4-B1",
  result: "Pass",
  snapshot: {
    game_version: "4.4",
    turnbasedgamedata_revision:
      "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    starrailres_revision:
      "7b349e39ee0f6f3bf814567995829b99c95e7a93",
  },
  selector: {
    guide_type: manifest.enabled_selector.guide_type,
    guide_tab_id: manifest.enabled_selector.guide_tab_id,
    guide_data_id: manifest.enabled_selector.guide_data_id,
  },
  manifest: {
    category_count: Object.keys(manifest.categories).length,
    obligation_count: manifestRows.length,
    ownership_counts: sortedObject(manifestOwnership),
    reachability_counts: sortedObject(reachability),
    sha256: sha256(fs.readFileSync(manifestPath)),
  },
  coverage: {
    exact_once_count: coverage.length,
    data_ready: coverage.filter(({ state }) => state === "DataReady").length,
    excluded: coverage.filter(({ state }) => state === "Excluded").length,
    unresolved: coverage.filter(({ state }) =>
      !["DataReady", "Excluded"].includes(state)).length,
  },
  pack: {
    digest: packIndex[0].pack_digest,
    index_chunk_count: packIndex.length,
  },
  normalized: {
    contract_file_count: schema.files.length,
    present_file_count: rowsByFile.size,
    nonempty_file_count: [...rowsByFile.values()]
      .filter((rows) => rows.length > 0).length,
    empty_file_count: [...rowsByFile.values()]
      .filter((rows) => rows.length === 0).length,
    row_count: allRows.length,
    globally_unique_id_count: idSet.size,
    bilingual_rows_audited: allRows.length,
    source_receipt_count: sources.length,
    source_reference_count: referenceCount,
    policy_reference_count: policyReferenceCount,
    unresolved_source_references: 0,
    other_mode_source_leaks: 0,
    runtime_enabled_or_lowered_rows: 0,
    unreleased_evidence_tags: 0,
    schema_sha256: sha256(fs.readFileSync(schemaPath)),
  },
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(fs.readFileSync(outputPath, "utf8") === encoded,
    "P4-B1 ownership audit drift");
} else {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
}
console.log(
  `Currency Wars ownership audit ${check ? "verified" : "generated"}: ` +
  `${coverage.length} exact-once obligations, ${allRows.length} bilingual ` +
  `rows and ${referenceCount} resolved source references.`,
);

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function sourceKey(value) {
  return canonical([
    value.repository,
    value.revision,
    value.path,
    value.locator,
    value.sha256,
    value.evidence_quality,
  ]);
}
function hasTruthyRuntimeFlag(value) {
  if (Array.isArray(value)) return value.some(hasTruthyRuntimeFlag);
  if (!value || typeof value !== "object") return false;
  return Object.entries(value).some(([key, child]) =>
    (["runtime_enabled", "runtime_lowered"].includes(key) && child === true)
      || hasTruthyRuntimeFlag(child));
}
function sortedObject(value) {
  return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) =>
      left.localeCompare(right)),
  );
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
