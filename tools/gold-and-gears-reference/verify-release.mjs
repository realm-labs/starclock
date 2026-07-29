#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const args = process.argv.slice(hasRoot ? 3 : 2);
assert(
  args.every((argument) => ["--bless", "--require-clean"].includes(argument)),
  "usage: verify-release.mjs [root] [--bless] [--require-clean]",
);
const bless = args.includes("--bless");
const requireClean = args.includes("--require-clean");
const artifactOnly = process.env.STARCLOCK_ARTIFACT_CHECK_ONLY === "1";

const policyPath = "policy/gold-and-gears-reference.json";
const policy = json(policyPath);
assert(
  policy.schema_revision === "starclock.gold-and-gears-reference-policy.v1" &&
    policy.goal_id === "gold-and-gears-reference-v1" &&
    policy.state === "Candidate",
  "Gold and Gears release policy identity differs",
);
assert(
  policy.runtime_boundary.delivery_lane === "CandidateReferenceOnly" &&
    policy.runtime_boundary.runtime_loading === false &&
    policy.runtime_boundary.runtime_lowering === false &&
    policy.runtime_boundary.runtime_handlers === 0 &&
    policy.runtime_boundary.playable_profile === false,
  "Gold and Gears runtime boundary differs",
);

const status = text("docs/goals/08-gold-and-gears-reference-data-status.md");
assert(status.includes("| State | `Complete` |"), "Goal 08 state is not Complete");
assert(status.includes("| Active phase | Complete |"), "Goal 08 still has an active phase");
assert(status.includes("| Active batch | None |"), "Goal 08 still has an active batch");
assert(status.includes("| Next unblocked batch | None |"), "Goal 08 has a next batch");
assert(
  (status.match(/^\| Phase [0-4].*\| `Complete` \|/gmu) ?? []).length === 5,
  "not every Goal 08 phase is Complete",
);
assert(
  (status.match(/^\| `G08-P[0-4]-B[0-9]+` \| `Complete` \|/gmu) ?? [])
    .length === 28,
  "not every Goal 08 batch is Complete",
);
assert(!status.includes("- [ ]"), "Goal 08 terminal checklist is incomplete");
assert(
  status.includes(
    "| Completion commit | This row's containing commit (`G08-P4-B4`) |",
  ),
  "Goal 08 completion record is missing",
);
assert(
  text("docs/goals/README.md").includes(
    "| Goal 08 — Gold and Gears Reference Data | Version 4.4 Gold and Gears " +
      "manifests, unique-mode mechanics, provenance, isolated Excel/Sora " +
      "authoring and review fixtures; no runtime | Complete |",
  ),
  "Goal index does not mark Goal 08 Complete",
);

if (!artifactOnly) {
  for (const script of [
    "verify-pack.mjs",
    "verify-sora-schema.mjs",
    "verify-semantic-fixtures.mjs",
    "audit-release.mjs",
    "verify-release-acceptance.mjs",
  ])
    run("node", [`tools/gold-and-gears-reference/${script}`, "."]);
}

const inventory = json(
  "content-manifests/gold-and-gears-v1/source-inventory.json",
);
const contentManifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const packManifest = json("content-reference/gold-and-gears-v1/manifest.json");
const packIndex = json("content-reference/gold-and-gears-v1/pack-index.json");
const coverage = json("content-reference/gold-and-gears-v1/coverage.json");
const rules = json("content-reference/gold-and-gears-v1/mechanic-rules.json");
const sources = json("content-reference/gold-and-gears-v1/sources.json");
const fixtures = json("content-reference/gold-and-gears-v1/review-fixtures.json");
const gaps = json("content-reference/gold-and-gears-v1/research-gaps.json");
const schema = json("config/gold-and-gears-generated/schema.lock").schema;
const audit = json("evidence/gold-and-gears-reference-v1/release-audit.json");
const semantic = json(
  "evidence/gold-and-gears-reference-v1/semantic-fixture-results.json",
);
const acceptance = json(
  "evidence/gold-and-gears-reference-v1/release-acceptance.json",
);
const visual = json(
  "evidence/gold-and-gears-reference-v1/release-visual-review.json",
);
const denominators = policy.terminal_denominators;

const sourceObligations = Object.values(contentManifest.categories)
  .reduce((sum, category) => sum + category.count, 0);
const dataReady = coverage.reduce((sum, row) => sum + row.data_ready, 0);
assert(
  inventory.counts.total === denominators.source_inventory_files &&
    Object.keys(contentManifest.categories).length ===
      denominators.manifest_categories &&
    sourceObligations === denominators.source_obligations &&
    dataReady === denominators.source_obligations,
  "source inventory, manifest or coverage denominator differs",
);
assert(
  packManifest.normalized_file_count === denominators.normalized_files &&
    schema.tables.length === denominators.workbook_tables &&
    acceptance.authoring.rows === denominators.workbook_rows &&
    rules.length === denominators.mechanic_rules &&
    sources.length === denominators.provenance_rows &&
    fixtures.length === denominators.semantic_fixture_families &&
    gaps.length === denominators.research_gaps &&
    gaps.filter(({ blocking }) => blocking).length ===
      denominators.blocking_research_gaps,
  "normalized, authoring or evidence denominator differs",
);
assert(
  audit.result === "pass" &&
    semantic.result === "pass" &&
    acceptance.result === "pass" &&
    visual.result === "pass",
  "one or more release evidence layers do not pass",
);
assert(
  contentManifest.counts.ownership.GoldAndGears ===
      policy.ownership.gold_and_gears_source_obligations &&
    contentManifest.counts.ownership.Shared ===
      policy.ownership.shared_source_obligations,
  "source-obligation ownership differs",
);

const evidence = {
  schema_revision: "starclock.gold-and-gears-reference-release.v1",
  goal_id: policy.goal_id,
  completed_on: "2026-07-29",
  result: "complete",
  delivery_state: policy.state,
  policy_sha256: sha256(policyPath),
  snapshot: policy.snapshot,
  source_revisions: Object.fromEntries(
    inventory.snapshot.repositories.map(({ id, revision }) => [id, revision]),
  ),
  checkpoints: {
    sora_and_visual_qa: "070ab224",
    release_audit: "1dc125e6",
    semantic_fixtures: "6137398b",
    release_acceptance: "3b705263",
    release_batch: "G08-P4-B4",
  },
  content: {
    source_inventory_files: inventory.counts.total,
    manifest_categories: Object.keys(contentManifest.categories).length,
    source_obligations: sourceObligations,
    data_ready: dataReady,
    ownership: contentManifest.counts.ownership,
    normalized_files: packManifest.normalized_file_count,
    mechanic_rules: rules.length,
    provenance_rows: sources.length,
    semantic_fixture_families: fixtures.length,
    semantic_operations: semantic.summary.ordered_operations,
    semantic_assertions: semantic.summary.assertions,
    research_gaps: gaps.length,
    replacement_conditions_verified:
      semantic.summary.replacement_conditions_verified,
    blocking_research_gaps: gaps.filter(({ blocking }) => blocking).length,
  },
  authoring: {
    adapter: "openpyxl==3.1.5",
    schema_export_authority: "sora-cli==0.3.0",
    tables: schema.tables.length,
    workbook_rows: acceptance.authoring.rows,
    workbook_semantic_sha256:
      "b35d3560bccc7730b54a1ec15348e2750f4f373350c147507568cb1218a83fea",
    workbooks: Object.fromEntries(Object.entries(
      acceptance.authoring.workbooks,
    ).map(([name, value]) => [name, value.sha256])),
  },
  digests: {
    source_inventory_sha256:
      sha256("content-manifests/gold-and-gears-v1/source-inventory.json"),
    content_manifest_sha256:
      sha256("content-manifests/gold-and-gears-v1/content-manifest.json"),
    normalized_pack_sha256: packIndex.pack_sha256,
    pack_index_file_sha256:
      sha256("content-reference/gold-and-gears-v1/pack-index.json"),
    schema_lock_sha256: sha256("config/gold-and-gears-generated/schema.lock"),
    candidate_bundle_sha256: acceptance.authoring.bundle.sha256,
    debug_export_sha256: acceptance.authoring.debug_digest,
    release_audit_sha256:
      sha256("evidence/gold-and-gears-reference-v1/release-audit.json"),
    semantic_fixture_results_sha256: sha256(
      "evidence/gold-and-gears-reference-v1/semantic-fixture-results.json",
    ),
    release_acceptance_sha256: sha256(
      "evidence/gold-and-gears-reference-v1/release-acceptance.json",
    ),
    release_visual_review_sha256: sha256(
      "evidence/gold-and-gears-reference-v1/release-visual-review.json",
    ),
  },
  protected_boundaries: acceptance.protected_boundaries,
  runtime_boundary: policy.runtime_boundary,
  acceptance: {
    focused_checks: "pass",
    dependency_and_workspace_checks: "pass",
    immutable_release_snapshots: "pass",
    clean_checkout: "pass",
    clean_checkout_staged_tree: "281e19d881edaf8036b5cd25e0e1fd3f0487aabb",
    full_repository_command:
      "node tools/repository-check/run.mjs --full --with-source-cache",
    full_repository_external_boundary:
      "pre-existing Goal 06 Cargo.lock baseline differs; Goal 08 inputs are not reached",
    full_repository_external_boundary_owner: "Goal06HistoricalReleaseContract",
  },
};
assert(
  evidence.digests.normalized_pack_sha256 ===
    "ea2f3a35807b9a7dae39be2d67fb5de955bfad7852718eb1d3393affed5a5623",
  "normalized Candidate pack digest differs",
);
assert(
  evidence.digests.candidate_bundle_sha256 ===
    "97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b",
  "Candidate Sora bundle digest differs",
);
assert(
  evidence.runtime_boundary.delivery_lane === "CandidateReferenceOnly" &&
    evidence.runtime_boundary.standard_or_production_bundle_mutation === false,
  "Candidate runtime/configuration boundary differs",
);

const output = `${JSON.stringify(evidence, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(path.join(root, policy.release_evidence)), {
    recursive: true,
  });
  fs.writeFileSync(path.join(root, policy.release_evidence), output);
} else {
  assert(fileExists(policy.release_evidence),
    "Goal 08 release evidence is missing; run with --bless");
  assert(text(policy.release_evidence).replaceAll("\r\n", "\n") === output,
    "Goal 08 release evidence is stale; run with --bless");
}
if (requireClean)
  assert(capture("git", ["status", "--porcelain"]) === "",
    "Goal 08 worktree is not clean");
console.log(
  `Goal 08 Candidate reference release verified (${sourceObligations}/` +
    `${sourceObligations} DataReady; ${schema.tables.length} tables; ` +
    `${fixtures.length} semantic families${requireClean ? ", clean" : ""}).`,
);

function run(command, commandArgs) {
  execFileSync(command, commandArgs, { cwd: root, stdio: "inherit" });
}

function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, {
    cwd: root,
    encoding: "utf8",
  }).trim();
}

function fileExists(relative) {
  return fs.statSync(path.join(root, relative), {
    throwIfNoEntry: false,
  })?.isFile();
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function json(relative) {
  return JSON.parse(text(relative));
}

function sha256(relative) {
  return createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
