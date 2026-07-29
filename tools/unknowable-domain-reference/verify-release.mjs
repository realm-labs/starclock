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

const policyPath = "policy/unknowable-domain-reference.json";
const policy = json(policyPath);
assert(
  policy.schema_revision === "starclock.unknowable-domain-reference-policy.v1" &&
    policy.goal_id === "unknowable-domain-reference-v1" &&
    policy.state === "Candidate",
  "Unknowable Domain release policy identity differs",
);
assert(
  policy.runtime_boundary.delivery_lane === "CandidateReferenceOnly" &&
    policy.runtime_boundary.runtime_loading === false &&
    policy.runtime_boundary.runtime_lowering === false &&
    policy.runtime_boundary.runtime_handlers === 0 &&
    policy.runtime_boundary.playable_profile === false &&
    policy.runtime_boundary.standard_or_production_bundle_mutation === false,
  "Unknowable Domain runtime boundary differs",
);

const status = text("docs/goals/10-unknowable-domain-reference-data-status.md");
assert(status.includes("| State | `Complete` |"), "Goal 10 state is not Complete");
assert(status.includes("| Active phase | Complete |"), "Goal 10 still has an active phase");
assert(status.includes("| Active batch | None |"), "Goal 10 still has an active batch");
assert(status.includes("| Next unblocked batch | None |"), "Goal 10 has a next batch");
assert(
  (status.match(/^\| Phase [0-4].*\| `Complete` \|/gmu) ?? []).length === 5,
  "not every Goal 10 phase is Complete",
);
assert(
  (status.match(/^\| `G10-P[0-4]-B[0-9]+` \| `Complete` \|/gmu) ?? [])
    .length === 28,
  "not every Goal 10 batch is Complete",
);
assert(!status.includes("- [ ]"), "Goal 10 terminal checklist is incomplete");
assert(
  status.includes(
    "| Completion commit | This row's containing commit (`G10-P4-B4`) |",
  ),
  "Goal 10 completion record is missing",
);
assert(
  text("docs/goals/README.md").includes(
    "| Goal 10 — Unknowable Domain Reference Data | Version 4.4 Unknowable " +
      "Domain manifests, stage/Alignment/Scepter/Component mechanics, " +
      "provenance, isolated Excel/Sora authoring and review fixtures; no " +
      "runtime | Complete |",
  ),
  "Goal index does not mark Goal 10 Complete",
);

if (!artifactOnly) {
  for (const script of [
    "verify-pack.mjs",
    "verify-sora-schema.mjs",
    "verify-semantic-fixtures.mjs",
    "audit-release.mjs",
    "verify-release-acceptance.mjs",
  ]) {
    run("node", [`tools/unknowable-domain-reference/${script}`, "."]);
  }
}

const inventory = json(
  "content-manifests/unknowable-domain-v1/source-inventory.json",
);
const contentManifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
const [packManifest] = json("content-reference/unknowable-domain-v1/manifest.json");
const [packIndex] = json("content-reference/unknowable-domain-v1/pack-index.json");
const coverage = json("content-reference/unknowable-domain-v1/coverage.json");
const rules = json("content-reference/unknowable-domain-v1/mechanic-rules.json");
const sources = json("content-reference/unknowable-domain-v1/sources.json");
const fixtures = json("content-reference/unknowable-domain-v1/review-fixtures.json");
const gaps = json("content-reference/unknowable-domain-v1/research-gaps.json");
const receipts = json(
  "content-reference/unknowable-domain-v1/reconciliation-receipts.json",
);
const schema = json("config/unknowable-domain-generated/schema.lock").schema;
const audit = json("evidence/unknowable-domain-reference-v1/release-audit.json");
const semantic = json(
  "evidence/unknowable-domain-reference-v1/semantic-fixture-results.json",
);
const acceptance = json(
  "evidence/unknowable-domain-reference-v1/release-acceptance.json",
);
const visual = json("evidence/unknowable-domain-reference-v1/visual-review.json");
const denominators = policy.terminal_denominators;

const sourceObligations = Object.values(contentManifest.categories)
  .reduce((sum, category) => sum + category.count, 0);
const dataReady = coverage.filter(({ state }) => state === "DataReady").length;
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
    packIndex.file_digests.length === denominators.pre_index_files &&
    schema.tables.length === denominators.workbook_tables &&
    acceptance.authoring.rows === denominators.workbook_rows &&
    rules.length === denominators.mechanic_rules &&
    sources.length === denominators.provenance_rows &&
    fixtures.length === denominators.semantic_fixture_families &&
    gaps.length === denominators.research_gaps &&
    gaps.filter(({ blocking }) => blocking).length ===
      denominators.blocking_research_gaps &&
    receipts.length === denominators.reconciliation_receipts &&
    semantic.summary.approximation_boundaries ===
      denominators.project_policy_boundaries,
  "normalized, authoring or evidence denominator differs",
);
assert(
  audit.result === "pass" &&
    semantic.result === "pass" &&
    acceptance.result === "pass" &&
    visual.defects.length === 0 &&
    Object.values(visual.checks).every(Boolean),
  "one or more release evidence layers do not pass",
);
assert(
  contentManifest.counts.ownership.UnknowableDomain ===
      policy.ownership.unknowable_domain_source_obligations &&
    contentManifest.counts.ownership.Shared ===
      policy.ownership.shared_source_obligations,
  "source-obligation ownership differs",
);

const evidence = {
  schema_revision: "starclock.unknowable-domain-reference-release.v1",
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
    sora_and_visual_qa: "b5d84f0d982c05e52108b2e7cc4b89957d7f0982",
    release_audit: "218bc1a1c7879dab8bb16372dacdd67fec2ec280",
    semantic_fixtures: "dfa9b5b9153657e6b46908a0d45b43007b3cd434",
    release_acceptance: "d79ac52bdc0c888b15a7165f780b0b52b2f555c6",
    release_batch: "G10-P4-B4",
  },
  content: {
    source_inventory_files: inventory.counts.total,
    manifest_categories: Object.keys(contentManifest.categories).length,
    source_obligations: sourceObligations,
    data_ready: dataReady,
    ownership: contentManifest.counts.ownership,
    normalized_files: packManifest.normalized_file_count,
    pre_index_files: packIndex.file_digests.length,
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
    workbook_semantic_sha256: acceptance.authoring.workbook_semantic_sha256,
    workbooks: Object.fromEntries(Object.entries(
      acceptance.authoring.workbooks,
    ).map(([name, value]) => [name, value.sha256])),
  },
  digests: {
    source_inventory_sha256:
      sha256("content-manifests/unknowable-domain-v1/source-inventory.json"),
    content_manifest_sha256:
      sha256("content-manifests/unknowable-domain-v1/content-manifest.json"),
    normalized_schema_sha256:
      sha256("content-manifests/unknowable-domain-v1/normalized-schema.json"),
    normalized_pack_sha256: packIndex.pack_digest,
    normalized_component_sha256: packIndex.component_digest,
    pack_index_file_sha256:
      sha256("content-reference/unknowable-domain-v1/pack-index.json"),
    schema_lock_sha256: sha256("config/unknowable-domain-generated/schema.lock"),
    candidate_bundle_sha256: acceptance.authoring.bundle.sha256,
    debug_export_index_sha256: acceptance.authoring.debug_digest,
    debug_export_bytes_sha256: visual.debug_export.sha256,
    reconciliation_checkpoints_sha256:
      acceptance.reconciliation.checkpoint_evidence_sha256,
    release_audit_sha256:
      sha256("evidence/unknowable-domain-reference-v1/release-audit.json"),
    semantic_fixture_results_sha256: sha256(
      "evidence/unknowable-domain-reference-v1/semantic-fixture-results.json",
    ),
    release_acceptance_sha256: sha256(
      "evidence/unknowable-domain-reference-v1/release-acceptance.json",
    ),
    release_visual_review_sha256:
      sha256("evidence/unknowable-domain-reference-v1/visual-review.json"),
  },
  protected_boundaries: acceptance.protected_boundaries,
  reconciliation: acceptance.reconciliation,
  runtime_boundary: policy.runtime_boundary,
  acceptance: {
    focused_checks: "pass",
    dependency_and_workspace_checks: "pass",
    immutable_release_snapshots: "pass",
    clean_checkout: "pass",
    clean_checkout_staged_tree: "fa01efe774440861e5c7068f9098bd9370cfa31f",
    full_repository_command:
      "node tools/repository-check/run.mjs --full --with-source-cache",
    full_repository_external_boundary:
      "pre-existing Goal 06 Cargo.lock baseline differs after all preceding checks pass",
    full_repository_external_boundary_owner: "Goal06HistoricalReleaseContract",
  },
};
assert(
  evidence.digests.normalized_pack_sha256 ===
    "f48f264fb55221e2494156c5ab7911719d703ec47f492c9c0e2d7fd2c8123b28",
  "normalized Candidate pack digest differs",
);
assert(
  evidence.digests.candidate_bundle_sha256 ===
    "05114105b6d905c2858865df08d7ab551cb0fb056b3871b959897a4a590451ec",
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
  assert(
    fileExists(policy.release_evidence),
    "Goal 10 release evidence is missing; run with --bless",
  );
  assert(
    text(policy.release_evidence).replaceAll("\r\n", "\n") === output,
    "Goal 10 release evidence is stale; run with --bless",
  );
}
if (requireClean) {
  assert(
    capture("git", ["status", "--porcelain"]) === "",
    "Goal 10 worktree is not clean",
  );
}
console.log(
  `Goal 10 Candidate reference release verified (${sourceObligations}/` +
    `${sourceObligations} DataReady; ${schema.tables.length} tables; ` +
    `${fixtures.length} semantic families${requireClean ? ", clean" : ""}).`,
);

function run(command, commandArgs) {
  execFileSync(command, commandArgs, {
    cwd: root,
    stdio: "inherit",
    maxBuffer: 64 * 1024 * 1024,
  });
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
