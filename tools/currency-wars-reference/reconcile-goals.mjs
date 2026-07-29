#!/usr/bin/env node

import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import {
  createContext,
  sha256,
  slug,
  writeOrCheck,
} from "./lib/common.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const check = process.argv.includes("--check");
const context = await createContext(root);
const referenceRoot = path.join(root, "content-reference/currency-wars-v1");
const register = json(path.join(
  root,
  "content-manifests/currency-wars-v1/reconciliation-checkpoints.json",
));
const outputPath = path.join(
  root,
  "evidence/currency-wars-reference-v1/p4b3-reconciliation-audit.json",
);
const currencySources = json(path.join(referenceRoot, "sources.json"))
  .map(normalizeSource);
const currencyByKey = new Map(currencySources.map((row) =>
  [sourceKey(row), row]));
const currencyByPath = Object.groupBy(currencySources, ({ path: sourcePath }) =>
  sourcePath);
const selectorEntry = (await context.table("GuideRogueTab"))
  .find(({ locator }) => locator === "2");
assert(selectorEntry, "Currency Wars selector receipt drift");
const selectorRef = context.sourceRef(selectorEntry);

assert(register.schema_revision
  === "starclock.currency-wars-reconciliation-checkpoints.v1"
  && register.checkpoints.length === 4,
"reconciliation checkpoint register drift");
assert(JSON.stringify(register.join_key)
  === JSON.stringify(["source_path", "row_locator", "evidence_sha256"]),
"reconciliation join-key drift");

const receipts = [];
const checkpointResults = [];
for (const checkpoint of register.checkpoints) {
  verifyCommit(checkpoint);
  const manifestBytes = gitShow(checkpoint.commit, checkpoint.manifest_path);
  assert(sha256(manifestBytes) === checkpoint.manifest_sha256,
    `${checkpoint.goal}: manifest digest drift`);
  const manifest = JSON.parse(manifestBytes);
  assert(manifest.goal_id === checkpoint.goal,
    `${checkpoint.goal}: manifest identity drift`);
  const manifestRows = Object.values(manifest.categories)
    .flatMap(({ records }) => records);
  assert(manifestRows.length === checkpoint.records,
    `${checkpoint.goal}: manifest denominator drift`);

  const otherSources = JSON.parse(gitShow(
    checkpoint.commit,
    checkpoint.sources_path,
  )).map(normalizeSource);
  assert(otherSources.length === checkpoint.source_receipts,
    `${checkpoint.goal}: source-receipt denominator drift`);
  const otherByKey = new Map(otherSources.map((row) =>
    [sourceKey(row), row]));
  const exactKeys = [...otherByKey.keys()]
    .filter((key) => currencyByKey.has(key))
    .sort(compare);
  const otherByPath = Object.groupBy(otherSources, ({ path: sourcePath }) =>
    sourcePath);
  const sharedPaths = Object.keys(otherByPath)
    .filter((sourcePath) => Object.hasOwn(currencyByPath, sourcePath))
    .sort(compare);
  let sameLocatorCount = 0;
  for (const sourcePath of sharedPaths) {
    const locators = new Set(currencyByPath[sourcePath]
      .map(({ locator }) => locator));
    sameLocatorCount += otherByPath[sourcePath]
      .filter(({ locator }) => locators.has(locator)).length;
  }

  const manifestReference = {
    source_id:
      `source.goal12.reconciliation.${slug(checkpoint.goal)}.manifest`,
    repository: "starclock",
    revision: checkpoint.commit,
    path: checkpoint.manifest_path,
    locator: "complete-manifest",
    sha256: checkpoint.manifest_sha256,
    access_date: "2026-07-30",
    game_version: "4.4",
    evidence_quality: "ExactStructured",
    mechanism_quality: "CommittedManifestComparison",
  };
  if (exactKeys.length === 0) {
    receipts.push(receipt({
      checkpoint,
      token: "currency-wars-only",
      sourcePath: selectorRef.path,
      locator: selectorRef.locator,
      evidenceSha: selectorRef.sha256,
      sourceRefs: [selectorRef, manifestReference],
      outcome: "CurrencyWarsOnly",
      checkpointOwnership: "NoExactJoinKey",
      currencyOwnership: "CurrencyWars",
      note:
        `Complete comparison found zero shared source-path/locator/digest keys across ${currencySources.length} Currency Wars and ${otherSources.length} ${checkpoint.goal} receipts; ${sharedPaths.length} shared path names grant no ownership.`,
    }));
  } else {
    for (const key of exactKeys) {
      const current = currencyByKey.get(key);
      receipts.push(receipt({
        checkpoint,
        token: sha256(key).slice(0, 16),
        sourcePath: current.path,
        locator: current.locator,
        evidenceSha: current.sha256,
        sourceRefs: [current.source_refs[0], manifestReference],
        outcome: "MatchedShared",
        checkpointOwnership: "SharedEvidence",
        currencyOwnership: current.ownership,
        note:
          "Both committed packs record the identical source path, row locator and evidence digest; semantic ownership remains mode-local unless separately proven shared.",
      }));
    }
  }
  checkpointResults.push({
    goal: checkpoint.goal,
    commit: checkpoint.commit,
    tree: checkpoint.tree,
    manifest_sha256: checkpoint.manifest_sha256,
    manifest_records: manifestRows.length,
    source_receipts: otherSources.length,
    exact_join_key_overlaps: exactKeys.length,
    shared_path_names: sharedPaths,
    shared_path_count: sharedPaths.length,
    same_path_and_locator_count: sameLocatorCount,
    outcome: exactKeys.length === 0 ? "CurrencyWarsOnly" : "MatchedShared",
    remote_verification: checkpoint.remote_ancestor
      ? {
        kind: "DirectAncestor",
        remote_ancestor: checkpoint.remote_ancestor,
      }
      : {
        kind: "ManifestDigestWitness",
        ...checkpoint.remote_witness,
      },
    ...(checkpoint.replacement_condition
      ? { replacement_condition: checkpoint.replacement_condition }
      : {}),
  });
}

receipts.sort((left, right) => compare(left.id, right.id));
assert(receipts.length >= register.checkpoints.length,
  "reconciliation receipt denominator drift");
assert(new Set(receipts.map(({ id }) => id)).size === receipts.length,
  "reconciliation receipt ID collision");
assert(receipts.every(({ outcome }) =>
  ["MatchedShared", "CurrencyWarsOnly"].includes(outcome)),
"reconciliation conflict or divergent representation detected");
await writeOrCheck(context, new Map([
  ["reconciliation-receipts.json", receipts],
]), check);

const report = {
  batch: "G12-P4-B3",
  result: "Pass",
  join_key: register.join_key,
  currency_wars_source_receipts: currencySources.length,
  checkpoint_count: checkpointResults.length,
  receipt_count: receipts.length,
  exact_overlap_count: checkpointResults.reduce((sum, entry) =>
    sum + entry.exact_join_key_overlaps, 0),
  conflict_count: 0,
  divergent_representation_count: 0,
  checkpoints: checkpointResults,
  policy:
    "A shared path name, table prefix, ID range or matching name is not a shared-row claim; only the complete source path, row locator and evidence digest key can match.",
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(outputPath), "reconciliation audit is missing");
  assert(fs.readFileSync(outputPath, "utf8") === encoded,
    "reconciliation audit drift");
} else {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
}
console.log(
  `Currency Wars Goal 08/09/10/11 reconciliation ` +
  `${check ? "verified" : "generated"} (${receipts.length} receipts; ` +
  `${report.exact_overlap_count} exact overlaps; 0 conflicts; audit ` +
  `${sha256(encoded)}).`,
);

function receipt({
  checkpoint,
  token,
  sourcePath,
  locator,
  evidenceSha,
  sourceRefs,
  outcome,
  checkpointOwnership,
  currencyOwnership,
  note,
}) {
  return {
    ...context.envelope({
      id: `currency-wars.reconciliation.${slug(checkpoint.goal)}.${token}`,
      kind: "CurrencyWarsReconciliationReceipts",
      nameEn: `Reconciliation with ${checkpoint.goal}`,
      nameZh: `与 ${checkpoint.goal} 的对账`,
      summaryEn:
        `${outcome} after complete source-path, row-locator and evidence-digest comparison.`,
      summaryZh:
        `按完整来源路径、行定位与证据摘要对账后的结果为 ${outcome}。`,
      sourceRefs,
      ownership: "Shared",
      tags: ["ownership", "reconciliation", slug(outcome)],
    }),
    source_path: sourcePath,
    row_locator: locator,
    evidence_sha256: evidenceSha,
    checkpoint: {
      goal: checkpoint.goal,
      commit: checkpoint.commit,
      manifest_sha256: checkpoint.manifest_sha256,
    },
    checkpoint_goal: checkpoint.goal,
    checkpoint_commit: checkpoint.commit,
    checkpoint_ownership: checkpointOwnership,
    currency_wars_ownership: currencyOwnership,
    outcome,
    note,
  };
}
function verifyCommit(checkpoint) {
  const tree = execFileSync("git", [
    "show",
    "--format=%T",
    "--no-patch",
    checkpoint.commit,
  ], { cwd: root, encoding: "utf8" }).trim();
  assert(tree === checkpoint.tree, `${checkpoint.goal}: tree drift`);
  if (checkpoint.remote_ancestor) {
    assert(isAncestor(checkpoint.commit, checkpoint.remote_ancestor),
      `${checkpoint.goal}: remote ancestry drift`);
    return;
  }
  const witness = checkpoint.remote_witness;
  assert(witness && isAncestor(witness.commit, witness.remote_ancestor),
    `${checkpoint.goal}: remote witness ancestry drift`);
  const witnessManifest = JSON.parse(gitShow(witness.commit, witness.path));
  assert(witnessManifest.exclusions?.gold_checkpoint?.manifest_sha256
    === witness.manifest_sha256,
  `${checkpoint.goal}: remote manifest witness drift`);
}
function isAncestor(commit, ancestor) {
  return spawnSync("git", [
    "merge-base",
    "--is-ancestor",
    commit,
    ancestor,
  ], { cwd: root }).status === 0;
}
function gitShow(commit, file) {
  return execFileSync("git", ["show", `${commit}:${file}`], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  });
}
function normalizeSource(row) {
  const normalized = {
    ...row,
    path: row.path ?? row.relative_path_or_page,
    locator: row.locator ?? row.row_locator,
    sha256: row.sha256 ?? row.evidence_sha256,
  };
  assert(normalized.path && normalized.locator
    && /^[0-9a-f]{64}$/u.test(normalized.sha256),
  `${row.id}: incomplete source receipt`);
  return normalized;
}
function sourceKey(row) {
  return `${row.path}\0${row.locator}\0${row.sha256}`;
}
function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
function compare(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
