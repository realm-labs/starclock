#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const index = args.indexOf("--partition");
assert(index >= 0 && typeof args[index + 1] === "string",
  "usage: verify-content-partition.mjs --partition <batch> [--expect-pending]");
const partitionId = args[index + 1];
const expectPending = args.includes("--expect-pending");
assert(args.every((value, offset) =>
  value === "--expect-pending" || value === "--partition" || offset === index + 1),
"unsupported content-partition verifier argument");

const gate = json("policy/goal07-shared-capability-gate.json");
const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const audit = json(
  "content-manifests/standard-universe-mechanics-complete-v1/retained-audit.json",
);
const partition = manifest.partitions.find(({ id }) => id === partitionId);
assert(partition, `unknown Goal 07 partition ${partitionId}`);
const receiptRelative = `${gate.partition_receipt.root}/${partitionId}.json`;
const receiptExists = exists(receiptRelative);
if (expectPending) {
  assert(!receiptExists, `${partitionId}: expected no completion receipt`);
  console.log(`Goal 07 partition ${partitionId} is structurally valid and Pending.`);
  process.exit(0);
}
assert(receiptExists, `${partitionId}: completion receipt is missing`);
for (const dependency of partition.dependencies)
  assert(exists(`${gate.partition_receipt.root}/${dependency}.json`),
    `${partitionId}: dependency ${dependency} has no completion receipt`);

const receipt = json(receiptRelative);
assert(receipt.schema_revision === gate.partition_receipt.schema_revision,
  `${partitionId}: receipt schema differs`);
assert(receipt.goal_id === gate.goal_id && receipt.partition_id === partitionId,
  `${partitionId}: receipt identity differs`);
assert(receipt.state === "Complete", `${partitionId}: receipt is not Complete`);
for (const section of gate.partition_receipt.required_sections)
  assert(Object.hasOwn(receipt, section), `${partitionId}: missing ${section} section`);

verifyAuthoring(receipt.authoring);
verifyEntries("records", partition.record_ids, receipt.records, audit.records);
verifyRules(partition.rule_ids, receipt.rules, audit.rules);
verifyFixtures(partition.fixture_ids, receipt.fixtures, audit.fixtures);
verifyEntries(
  "enemy_variants",
  partition.enemy_variant_ids,
  receipt.enemy_variants,
  audit.enemy_variants,
);
verifyEntries(
  "encounter_members",
  partition.encounter_member_ids,
  receipt.encounter_members,
  audit.encounter_members,
);
verifyNativeReviews(receipt.native_handler_reviews);
verifyExecution(receipt.execution);
console.log(
  `Goal 07 partition ${partitionId} verified ` +
  `(${partition.record_ids.length} records, ${partition.rule_ids.length} rules, ` +
  `${partition.fixture_ids.length} fixtures).`,
);

function verifyAuthoring(authoring) {
  assert(authoring && Array.isArray(authoring.workbooks) && authoring.workbooks.length > 0,
    `${partitionId}: no authoritative workbook evidence`);
  for (const workbook of authoring.workbooks) {
    assert(typeof workbook.path === "string" && workbook.path.endsWith(".xlsx"),
      `${partitionId}: authoring path is not xlsx`);
    assert(exists(workbook.path), `${partitionId}: workbook is missing ${workbook.path}`);
    assert(Array.isArray(workbook.tables) && workbook.tables.length > 0,
      `${partitionId}: workbook ${workbook.path} has no owned table rows`);
  }
  assert(Array.isArray(authoring.openpyxl_commands)
    && authoring.openpyxl_commands.length > 0
    && authoring.openpyxl_commands.every((command) =>
      typeof command === "string" && command.includes("openpyxl")),
  `${partitionId}: openpyxl authoring command is absent`);
  assert(fileDigest(authoring.sora_bundle), `${partitionId}: Sora bundle evidence differs`);
  assert(fileDigest(authoring.sora_golden), `${partitionId}: Sora golden evidence differs`);
}
function verifyEntries(label, expectedIds, entries, sourceEntries) {
  assert(Array.isArray(entries), `${partitionId}: ${label} is not an array`);
  exactIds(label, expectedIds, entries);
  const source = new Map(sourceEntries.map((entry) => [entry.id, entry]));
  for (const entry of entries) {
    const planned = source.get(entry.id);
    assert(planned, `${partitionId}: ${label} ${entry.id} is absent from retained audit`);
    verifyDisposition(entry, planned, label);
  }
}
function verifyRules(expectedIds, entries, sourceEntries) {
  verifyEntries("rules", expectedIds, entries, sourceEntries);
  for (const entry of entries) {
    assert(["RuleIr", "SharedPrimitive", "NativeHandler"].includes(entry.implementation_kind),
      `${partitionId}: rule ${entry.id} has no executable implementation`);
    assert(Array.isArray(entry.definition_keys) && entry.definition_keys.length > 0,
      `${partitionId}: rule ${entry.id} has no formal definition key`);
    assert(Array.isArray(entry.execution_evidence) && entry.execution_evidence.length > 0,
      `${partitionId}: rule ${entry.id} has no runtime execution evidence`);
    verifyEvidence(entry.execution_evidence, `rule ${entry.id}`);
  }
}
function verifyFixtures(expectedIds, entries, sourceEntries) {
  verifyEntries("fixtures", expectedIds, entries, sourceEntries);
  for (const entry of entries) {
    assert(["RustTest", "CliGolden", "ReplayGolden", "ScenarioGolden"].includes(
      entry.execution_kind,
    ), `${partitionId}: fixture ${entry.id} is not executable`);
    assert(typeof entry.test_path === "string" && exists(entry.test_path),
      `${partitionId}: fixture ${entry.id} test path is missing`);
    assert(nonEmpty(entry.test_marker)
      && text(entry.test_path).includes(entry.test_marker),
    `${partitionId}: fixture ${entry.id} test marker is missing`);
  }
}
function verifyDisposition(entry, planned, label) {
  assert(nonEmpty(entry.runtime_disposition)
    && !gate.partition_receipt.forbidden_runtime_claims.includes(
      entry.runtime_disposition,
    ), `${partitionId}: ${label} ${entry.id} retains a nonterminal runtime claim`);
  assert(entry.accuracy_disposition === planned.intended_accuracy_disposition,
    `${partitionId}: ${label} ${entry.id} accuracy disposition differs`);
  assert(Array.isArray(entry.workbook_evidence) && entry.workbook_evidence.length > 0,
    `${partitionId}: ${label} ${entry.id} has no workbook evidence`);
  assert(Array.isArray(entry.provenance_evidence)
    && entry.provenance_evidence.length > 0,
  `${partitionId}: ${label} ${entry.id} has no provenance evidence`);
  verifyEvidence(entry.workbook_evidence, `${label} ${entry.id}`);
  verifyEvidence(entry.provenance_evidence, `${label} ${entry.id}`);
}
function verifyNativeReviews(reviews) {
  assert(Array.isArray(reviews), `${partitionId}: native reviews are not an array`);
  exactIds(
    "native review candidates",
    partition.native_review_candidate_rule_ids,
    reviews,
  );
  const admitted = [];
  for (const review of reviews) {
    assert(["IrSufficient", "Admitted"].includes(review.outcome),
      `${partitionId}: native review ${review.id} is not terminal`);
    assert(nonEmpty(review.decision), `${partitionId}: native review has no decision`);
    verifyEvidence(review.evidence, `native review ${review.id}`);
    if (review.outcome === "Admitted") {
      assert(Number.isInteger(review.handler_id), `${partitionId}: admitted handler lacks ID`);
      admitted.push(review.handler_id);
    }
  }
  assert(admitted.length <= gate.native_handler_admission.maximum_new_handlers_per_partition,
    `${partitionId}: native-handler admission cap exceeded`);
}
function verifyExecution(execution) {
  assert(execution?.result === "pass", `${partitionId}: execution result is not pass`);
  assert(Array.isArray(execution.commands) && execution.commands.length > 0,
    `${partitionId}: no executed command evidence`);
  assert(execution.commands.every(nonEmpty), `${partitionId}: blank execution command`);
  assert(Array.isArray(execution.goldens) && execution.goldens.length > 0,
    `${partitionId}: no runtime golden evidence`);
  verifyEvidence(execution.goldens, "partition execution");
}
function verifyEvidence(entries, label) {
  assert(Array.isArray(entries) && entries.length > 0,
    `${partitionId}: ${label} evidence is empty`);
  for (const entry of entries) {
    assert(typeof entry.path === "string" && exists(entry.path),
      `${partitionId}: ${label} evidence path is missing`);
    if (entry.sha256 !== undefined)
      assert(fileDigest(entry), `${partitionId}: ${label} evidence digest differs`);
  }
}
function exactIds(label, expected, entries) {
  const actual = entries.map(({ id }) => id);
  assert(new Set(actual).size === actual.length, `${partitionId}: duplicate ${label}`);
  assert(JSON.stringify([...actual].sort()) === JSON.stringify([...expected].sort()),
    `${partitionId}: ${label} do not exactly cover the frozen assignment`);
}
function fileDigest(entry) {
  if (!entry || typeof entry.path !== "string" || !/^[0-9a-f]{64}$/u.test(entry.sha256)
    || !exists(entry.path))
    return false;
  if (sha256(entry.path) === entry.sha256) return true;
  if (!/^[0-9a-f]{40}$/u.test(entry.git_blob_sha1 ?? "")) return false;
  try {
    const bytes = execFileSync("git", ["cat-file", "blob", entry.git_blob_sha1], {
      cwd: root,
      encoding: "buffer",
    });
    return crypto.createHash("sha256").update(bytes).digest("hex") === entry.sha256;
  } catch {
    return false;
  }
}
function sha256(relative) {
  return crypto.createHash("sha256").update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) { return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile(); }
function text(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function nonEmpty(value) { return typeof value === "string" && value.trim().length > 0; }
function assert(condition, message) { if (!condition) throw new Error(message); }
