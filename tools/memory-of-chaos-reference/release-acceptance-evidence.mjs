#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { assert, root, writeText } from "./lib.mjs";

const check = process.argv.includes("--check");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
async function fileDigest(relativePath) { return sha256(await readFile(path.join(root, relativePath))); }
async function treeDigest(relativePath) {
  const directory = path.join(root, relativePath);
  const names = (await readdir(directory)).sort((left, right) => left.localeCompare(right, "en"));
  const hash = createHash("sha256");
  for (const name of names) {
    hash.update(name);
    hash.update("\0");
    hash.update(await readFile(path.join(directory, name)));
    hash.update("\0");
  }
  return hash.digest("hex");
}

const pack = JSON.parse(await readFile(path.join(
  root,
  "content-reference/memory-of-chaos-v1/pack-index.json",
), "utf8")).records[0];
const reconciliation = JSON.parse(await readFile(path.join(
  root,
  "content-reference/memory-of-chaos-v1/reconciliation-receipts.json",
), "utf8")).records;
assert(reconciliation.length === 305 && !reconciliation.some(({ semantic_result: result }) => result === "Conflict"),
  "reconciliation acceptance drift");

const evidence = {
  schema_revision: "starclock.memory-of-chaos-release-acceptance.v1",
  goal_id: "memory-of-chaos-reference-v1",
  lane: "Candidate",
  result: "Pass",
  runtime_profile: "Unreleased",
  source_revisions: {
    turnbasedgamedata: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    StarRailRes: "7b349e39ee0f6f3bf814567995829b99c95e7a93",
    shared_enemy_goal: "60ca52ed98c5c83d867d33bff7f88c69e0b389de",
  },
  coverage: {
    required: pack.manifest_required,
    accounted: pack.manifest_accounted,
    data_ready: pack.manifest_data_ready,
    fixture_families: pack.semantic_fixture_family_count,
    research_gaps: pack.research_gap_count,
    blocking_research_gaps: pack.blocking_research_gap_count,
  },
  reconciliation: {
    receipts: reconciliation.length,
    matches: reconciliation.filter(({ semantic_result: result }) => result === "Match").length,
    compatible_projections: reconciliation.filter(({ semantic_result: result }) => result === "CompatibleProjection").length,
    conflicts: reconciliation.filter(({ semantic_result: result }) => result === "Conflict").length,
  },
  artifacts: {
    canonical_pack_sha256: pack.canonical_pack_sha256,
    manifest_sha256: pack.manifest_sha256,
    schema_lock_sha256: await fileDigest("config/memory-of-chaos-generated/schema.lock"),
    workbook_sha256: {
      MemoryOfChaos: await fileDigest("config/memory-of-chaos/data/MemoryOfChaos.xlsx"),
      MemoryOfChaosBindings: await fileDigest("config/memory-of-chaos/data/MemoryOfChaosBindings.xlsx"),
      MemoryOfChaosReview: await fileDigest("config/memory-of-chaos/data/MemoryOfChaosReview.xlsx"),
    },
    bundle_sha256: await fileDigest("config/memory-of-chaos-generated/config.sora"),
    debug_tree_sha256: await treeDigest("config/memory-of-chaos-generated/debug-json"),
    coverage_audit_sha256: await fileDigest(
      "evidence/memory-of-chaos-reference-v1/release-audits/coverage-ownership-audit.json"),
    semantic_fixture_results_sha256: await fileDigest(
      "evidence/memory-of-chaos-reference-v1/release-audits/semantic-fixture-results.json"),
  },
  acceptance: {
    unified_verifier: "fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/verify-candidate-release.mjs",
    clean_prospective_tree: "Pass",
    clean_verifier: "fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/verify-candidate-release.mjs --clean-checkout",
    full_gate_with_source_cache: "Pass",
    full_gate: "fnm exec --using 24.15.0 node tools/repository-check/run.mjs --full --with-source-cache",
  },
};
assert(evidence.coverage.required === 477 && evidence.coverage.data_ready === 477
  && evidence.coverage.fixture_families === 18 && evidence.coverage.blocking_research_gaps === 0,
"release acceptance coverage drift");
assert(evidence.reconciliation.matches === 303 && evidence.reconciliation.compatible_projections === 2,
  "release acceptance reconciliation drift");
await writeText(
  "evidence/memory-of-chaos-reference-v1/release-audits/release-acceptance.json",
  `${JSON.stringify(evidence, null, 2)}\n`,
  check,
);
console.log(`Goal 17 release-acceptance evidence ${check ? "verified" : "generated"}: ${pack.canonical_pack_sha256}.`);
