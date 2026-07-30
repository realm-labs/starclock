#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const write = arguments_.includes("--write");
assert(
  arguments_.every((argument) =>
    argument === "--write" || !argument.startsWith("--")
  ),
  "usage: verify-release.mjs [root] [--write]",
);
const root = path.resolve(
  arguments_.find((argument) => !argument.startsWith("--")) ??
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const evidenceRelative =
  "evidence/divergent-universe-reference-v1/release/release-evidence.json";
const status = text("docs/goals/11-divergent-universe-reference-data-status.md");
const plan = text("docs/goals/11-divergent-universe-reference-data.md");

assert(status.includes("| State | `Complete` |"), "Goal 11 is not Complete");
assert(
  status.includes("| Active phase | Complete |") &&
    status.includes("| Active batch | None |") &&
    status.includes("| Next unblocked batch | None |"),
  "Goal 11 still has active or pending work",
);
assert(
  (status.match(/^\| Phase [0-4].*\| `Complete` \|/gmu) ?? []).length === 5,
  "not every Goal 11 phase is Complete",
);
assert(
  (status.match(/^\| `G11-P[0-4]-B[0-9]+` \| `Complete` \|/gmu) ?? [])
    .length === 29,
  "not every Goal 11 batch is Complete",
);
assert(!status.includes("- [ ]"), "Goal 11 terminal checklist is incomplete");
assert(
  status.includes(
    "| Completion commit | This row's containing commit (`G11-P4-B4`) |",
  ),
  "Goal 11 completion record is missing",
);
assert(
  goalIndexMarksComplete(
    text("docs/goals/README.md"),
    "Goal 11 — Divergent Universe Reference Data",
  ),
  "Goal index does not mark Goal 11 Complete",
);
const planBatches = batchSet(plan);
const statusBatches = batchSet(status);
assert(
  planBatches.length === 29 &&
    JSON.stringify(planBatches) === JSON.stringify(statusBatches),
  "Goal 11 plan/status batch sets differ",
);
const localMarkdownLinks = verifyLocalMarkdownLinks();
const evidenceMarkdownLinks = write
  ? localMarkdownLinks
  : retainedMarkdownLinkCount(evidenceRelative);

const inventory = json(
  "content-manifests/divergent-universe-v1/source-inventory.json",
);
const contentManifest = json(
  "content-manifests/divergent-universe-v1/content-manifest.json",
);
const [manifest] = json("content-reference/divergent-universe-v1/manifest.json");
const [packIndex] = json(
  "content-reference/divergent-universe-v1/pack-index.json",
);
const coverage = json("content-reference/divergent-universe-v1/coverage.json");
const rules = json("content-reference/divergent-universe-v1/mechanic-rules.json");
const sources = json("content-reference/divergent-universe-v1/sources.json");
const fixtures = json(
  "content-reference/divergent-universe-v1/review-fixtures.json",
);
const gaps = json("content-reference/divergent-universe-v1/research-gaps.json");
const receipts = json(
  "content-reference/divergent-universe-v1/reconciliation-receipts.json",
);
const schema = json("config/divergent-universe-generated/schema.lock").schema;
const audit = json(
  "evidence/divergent-universe-reference-v1/release-audit.json",
);
const semantic = json(
  "evidence/divergent-universe-reference-v1/semantic-fixture-results.json",
);
const reconciliation = json(
  "evidence/divergent-universe-reference-v1/reconciliation-checkpoints.json",
);
const acceptance = json(
  "evidence/divergent-universe-reference-v1/release-acceptance.json",
);
const visual = json(
  "evidence/divergent-universe-reference-v1/visual-review.json",
);
const sourceObligations = Object.values(contentManifest.categories).reduce(
  (sum, category) => sum + category.count,
  0,
);
const dataReady = coverage.filter(({ state }) => state === "DataReady").length;

assert(
  inventory.counts.total === 2_684 &&
    Object.keys(contentManifest.categories).length === 50 &&
    sourceObligations === 6_215 &&
    coverage.length === sourceObligations &&
    dataReady === sourceObligations,
  "source inventory, manifest or coverage denominator differs",
);
assert(
  manifest.candidate_quality === true &&
    manifest.runtime_loading === "ForbiddenReferenceOnly" &&
    manifest.frozen_source_obligations === sourceObligations &&
    manifest.data_ready_source_obligations === sourceObligations &&
    manifest.coverage_percent === "100" &&
    manifest.blocking_research_gap_count === 0,
  "Candidate manifest differs",
);
assert(
  manifest.normalized_files.length === 80 &&
    packIndex.file_digests.length === 79 &&
    audit.normalized.common_rows === 27_091 &&
    rules.length === 669 &&
    sources.length === 7_624 &&
    fixtures.length === 25 &&
    gaps.length === 25 &&
    gaps.every(({ blocking }) => blocking === false) &&
    receipts.length === 102,
  "normalized or evidence denominator differs",
);
assert(
  schema.tables.length === 80 &&
    acceptance.authoring.rows === 27_091 &&
    acceptance.authoring.verified_empty_tables === 2 &&
    audit.result === "pass" &&
    semantic.result === "pass" &&
    reconciliation.result === "pass" &&
    acceptance.result === "pass" &&
    visual.defects.length === 0 &&
    Object.values(visual.checks).every(Boolean),
  "one or more release evidence layers do not pass",
);
assert(
  semantic.summary.required_families === 25 &&
    semantic.summary.ordered_operations === 75 &&
    semantic.summary.assertions === 75 &&
    semantic.summary.replacement_conditions_verified === 54 &&
    semantic.summary.runtime_executions === 0,
  "semantic fixture result differs",
);
assert(
  reconciliation.summary.exact_shared_source_records === receipts.length &&
    reconciliation.summary.same_locator_different_digest === 181 &&
    reconciliation.summary.conflicts === 0,
  "reconciliation result differs",
);

const evidence = {
  schema_revision: "starclock.divergent-universe-reference-release.v1",
  goal_id: "divergent-universe-reference-v1",
  completed_on: "2026-07-29",
  result: "complete",
  delivery_state: "Candidate",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    mode_family: "simulated-universe-divergent-universe",
    profile_id: "divergent-universe.profile.v1",
  },
  source_revisions: Object.fromEntries(
    inventory.snapshot.repositories.map(({ id, revision }) => [id, revision]),
  ),
  checkpoints: {
    sora_and_visual_qa: "d43b1886b62cfb4e8d26f5712a5d2d4ec112e350",
    release_audit: "f36269d6079f53806a0239b6a3734198c8572005",
    semantic_fixtures: "d3928d69a5e6b2622b8bea41f370f2bf328ff072",
    release_acceptance: "5efce0dbe3ba7431c049eba50ab30f6798963c15",
    release_batch: "G11-P4-B4",
  },
  content: {
    source_inventory_files: inventory.counts.total,
    manifest_categories: Object.keys(contentManifest.categories).length,
    source_obligations: sourceObligations,
    data_ready: dataReady,
    normalized_files: manifest.normalized_files.length,
    pre_index_files: packIndex.file_digests.length,
    normalized_rows: audit.normalized.common_rows,
    ownership: audit.normalized.ownership,
    mechanic_rules: rules.length,
    provenance_rows: sources.length,
    semantic_fixture_families: fixtures.length,
    semantic_operations: semantic.summary.ordered_operations,
    semantic_assertions: semantic.summary.assertions,
    research_gaps: gaps.length,
    replacement_conditions_verified:
      semantic.summary.replacement_conditions_verified,
    blocking_research_gaps: gaps.filter(({ blocking }) => blocking).length,
    reconciliation_receipts: receipts.length,
  },
  authoring: {
    adapter: "openpyxl==3.1.5",
    schema_export_authority: "sora-cli==0.3.0",
    tables: schema.tables.length,
    workbook_rows: acceptance.authoring.rows,
    verified_empty_tables: acceptance.authoring.verified_empty_tables,
    workbook_semantic_sha256:
      acceptance.authoring.workbook_semantic_sha256,
    workbooks: Object.fromEntries(
      Object.entries(acceptance.authoring.workbooks).map(
        ([name, value]) => [name, value.sha256],
      ),
    ),
  },
  digests: {
    source_inventory_sha256: sha256(
      "content-manifests/divergent-universe-v1/source-inventory.json",
    ),
    content_manifest_sha256: sha256(
      "content-manifests/divergent-universe-v1/content-manifest.json",
    ),
    normalized_schema_sha256: sha256(
      "content-manifests/divergent-universe-v1/normalized-schema.json",
    ),
    normalized_pack_sha256: packIndex.pack_digest,
    pack_index_file_sha256: sha256(
      "content-reference/divergent-universe-v1/pack-index.json",
    ),
    schema_lock_sha256: sha256(
      "config/divergent-universe-generated/schema.lock",
    ),
    candidate_bundle_sha256: acceptance.authoring.bundle.sha256,
    candidate_bundle_bytes: acceptance.authoring.bundle.bytes,
    debug_export_sha256: acceptance.authoring.debug_digest,
    reconciliation_checkpoints_sha256:
      acceptance.reconciliation.checkpoint_evidence_sha256,
    release_audit_sha256: sha256(
      "evidence/divergent-universe-reference-v1/release-audit.json",
    ),
    semantic_fixture_results_sha256: sha256(
      "evidence/divergent-universe-reference-v1/semantic-fixture-results.json",
    ),
    release_acceptance_sha256: sha256(
      "evidence/divergent-universe-reference-v1/release-acceptance.json",
    ),
    visual_review_sha256: sha256(
      "evidence/divergent-universe-reference-v1/visual-review.json",
    ),
  },
  protected_boundaries: acceptance.protected_boundaries,
  reconciliation: acceptance.reconciliation,
  runtime_boundary: {
    delivery_lane: "CandidateReferenceOnly",
    runtime_loading: false,
    runtime_lowering: false,
    runtime_handlers: 0,
    playable_profile: false,
    standard_or_production_bundle_mutation: false,
  },
  acceptance: {
    focused_checks: "pass",
    dependency_and_workspace_checks: "pass",
    immutable_release_snapshots: "pass",
    clean_checkout: "pass",
    clean_checkout_staged_tree: "e79d12c706959ae934138d02ee85d21916efb974",
    full_repository_command:
      "node tools/repository-check/run.mjs --full --with-source-cache",
    full_repository_external_boundary:
      "pre-existing Goal 06 Cargo.lock baseline differs after all preceding checks pass",
    full_repository_external_boundary_owner: "Goal06HistoricalReleaseContract",
    plan_status_batches: planBatches.length,
    verified_local_markdown_links: evidenceMarkdownLinks,
  },
  publication: {
    remote: "origin",
    branch: "codex/goal11-divergent-universe-reference",
    published_through: "5efce0dbe3ba7431c049eba50ab30f6798963c15",
    completion_commit: "This row's containing commit (G11-P4-B4)",
  },
};
assert(
  evidence.digests.normalized_pack_sha256 ===
    "74234f3f689db6ba897d13865e079a3404ab707d3ddd978d646390e7b50bad02",
  "normalized Candidate pack digest differs",
);
assert(
  evidence.digests.candidate_bundle_sha256 ===
    "3221d0965292de6bbbd834338c2ff088821200ea22a4b7e7c65afc996444c5cf",
  "Candidate Sora bundle digest differs",
);
assert(
  evidence.authoring.workbook_semantic_sha256 ===
    "b083e897f1938603dc69dec0c07090b4215d6af7a5c49a19448e23a07d7aba0d",
  "workbook semantic digest differs",
);

const output = `${JSON.stringify(evidence, null, 2)}\n`;
const outputPath = path.join(root, evidenceRelative);
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, output);
} else {
  assert(fs.existsSync(outputPath), "release evidence is missing; run --write");
  assert(
    fs.readFileSync(outputPath, "utf8") === output,
    "release evidence drifted",
  );
}
console.log(
  `Goal 11 Candidate reference release ${write ? "written" : "verified"} ` +
    `(${sourceObligations}/${sourceObligations} DataReady; ` +
    `${schema.tables.length} tables; ${fixtures.length} semantic families).`,
);

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

function batchSet(value) {
  return [...new Set(value.match(/G11-P[0-4]-B[0-9]+/gu) ?? [])].sort();
}

function verifyLocalMarkdownLinks() {
  const markdownFiles = execFileSync(
    "git",
    ["ls-files", "*.md"],
    { cwd: root, encoding: "utf8" },
  ).trim().split(/\r?\n/u).filter(Boolean);
  let checked = 0;
  for (const relative of markdownFiles) {
    const body = text(relative);
    for (const match of body.matchAll(/!?\[[^\]]*\]\(([^)\s]+)\)/gu)) {
      let target = match[1];
      if (
        target.startsWith("#") ||
        /^[a-z][a-z0-9+.-]*:/iu.test(target)
      ) {
        continue;
      }
      if (target.startsWith("<") && target.endsWith(">")) {
        target = target.slice(1, -1);
      }
      const fileTarget = decodeURIComponent(target.split("#", 1)[0]);
      if (fileTarget.length === 0) continue;
      const resolved = path.resolve(root, path.dirname(relative), fileTarget);
      assert(
        resolved.startsWith(`${root}${path.sep}`) &&
          fs.statSync(resolved, { throwIfNoEntry: false }),
        `${relative}: broken local Markdown link ${target}`,
      );
      checked += 1;
    }
  }
  return checked;
}

function goalIndexMarksComplete(index, goalLabel) {
  const row = index
    .split(/\r?\n/u)
    .find((line) => line.startsWith(`| ${goalLabel} |`));
  const state = row?.split("|")[3]?.trim();
  return /^Complete(?:; .+)?$/u.test(state ?? "");
}

function retainedMarkdownLinkCount(relative) {
  assert(
    fs.existsSync(path.join(root, relative)),
    "release evidence is missing; run --write",
  );
  const count = json(relative).acceptance?.verified_local_markdown_links;
  assert(
    Number.isSafeInteger(count) && count >= 0,
    "release evidence Markdown-link count is invalid",
  );
  return count;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
