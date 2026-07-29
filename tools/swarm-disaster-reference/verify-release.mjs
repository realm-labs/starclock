#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const arguments_ = process.argv.slice(2);
const write = arguments_.includes("--write");
assert(
  arguments_.every((argument) =>
    argument === "--write" || !argument.startsWith("--")),
  "usage: verify-release.mjs [root] [--write]",
);
const root = path.resolve(
  arguments_.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const evidenceRelative =
  "evidence/swarm-disaster-reference-v1/release-evidence.json";
const status = text("docs/goals/09-swarm-disaster-reference-data-status.md");

assert(status.includes("| State | `Complete` |"),
  "Goal 09 state is not Complete");
assert(status.includes("| Active batch | None |")
  && status.includes("| Next unblocked batch | None |"),
"Goal 09 still has active work");
assert(
  (status.match(/^\| Phase [0-4].*\| `Complete` \|/gmu) ?? []).length === 5,
  "not every Goal 09 phase is Complete",
);
assert(
  (status.match(/^\| `G09-P[0-4]-B\d+` \| `Complete` \|/gmu) ?? []).length
    === 29,
  "not every Goal 09 batch is Complete",
);
assert(!status.includes("- [ ]"), "Goal 09 terminal checklist is incomplete");
assert(
  status.includes(
    "| Completion commit | This row's containing commit (`G09-P4-B4`) |",
  ),
  "Goal 09 completion record is missing",
);
assert(
  text("docs/goals/README.md").includes(
    "| Goal 09 — Swarm Disaster Reference Data | Version 4.4 Swarm Disaster " +
    "manifests, map/dice/progression mechanics, provenance, isolated " +
    "Excel/Sora authoring and review fixtures; no runtime | Complete |",
  ),
  "Goal index does not mark Goal 09 Complete",
);
for (const marker of [
  "Candidate reference release",
  "ForbiddenReferenceOnly",
  "run-clean-checkout.mjs",
])
  assert(
    text("content-reference/swarm-disaster-v1/README.md").includes(marker),
    `normalized reference README omits ${marker}`,
  );

const commits = {
  foundation: "3abe9a80adbba314d780f8c3c963c37af69aed92",
  authoring: "6ffd6cf7d3c6d23141f6ce425170d9d59d94dbaa",
  sora_and_visual: "820a3423db0e3b54b83695381741809376a6b64f",
  release_audit: "b8da6744a63cd92554b45f8e780d79a1be131f50",
  semantic_fixtures: "93261e6b437c5334fed608275e2e1ea8dad76250",
  integration_acceptance: "fc3f8a92f0c6f865e2d9fec14e4fbc44ad69196f",
};
for (const commit of Object.values(commits))
  runGit(["cat-file", "-e", `${commit}^{commit}`]);

const manifest = json("content-reference/swarm-disaster-v1/manifest.json");
const packIndex = json("content-reference/swarm-disaster-v1/pack-index.json");
const releaseAudit = json(
  "evidence/swarm-disaster-reference-v1/release-audit.json",
);
const semanticAudit = json(
  "evidence/swarm-disaster-reference-v1/semantic-fixture-audit.json",
);
const integration = json(
  "evidence/swarm-disaster-reference-v1/integration-acceptance.json",
);
const visual = json(
  "evidence/swarm-disaster-reference-v1/visual-review.json",
);
const schemaLock = json("config/swarm-disaster-generated/schema.lock");
const gaps = json("content-reference/swarm-disaster-v1/research-gaps.json");
const receipts = json(
  "content-reference/swarm-disaster-v1/reconciliation-receipts.json",
);
const packDigest = packIndex[0]?.pack_sha256;
const workbookSemantic = workbookSemanticDigest();

assert(
  manifest.candidate_quality === true
    && manifest.runtime_loading === "ForbiddenReferenceOnly"
    && manifest.frozen_source_obligations === 6_963
    && manifest.data_ready_source_obligations === 6_963
    && manifest.coverage_percent === "100"
    && manifest.blocking_research_gap_count === 0,
  "Candidate manifest differs",
);
assert(
  releaseAudit.denominators.normalized_records === 27_820
    && releaseAudit.denominators.bilingual_records === 19_617
    && releaseAudit.checks.gold_rows_enabled_for_swarm === 0
    && releaseAudit.checks.unknowable_or_divergent_rows === 0,
  "release audit differs",
);
assert(
  semanticAudit.denominators.mechanic_families === 23
    && semanticAudit.denominators.ordered_operations_executed === 85
    && semanticAudit.denominators.expected_facts_executed === 108
    && semanticAudit.checks.blocking_boundaries === 0,
  "semantic audit differs",
);
assert(
  integration.result === "pass"
    && integration.reconciliation.matching_receipts === 609
    && integration.reconciliation.conflicts === 0
    && integration.isolation.protected_changes === 0
    && integration.artifacts.debug_tables === 65
    && integration.artifacts.debug_rows === 33_380,
  "integration acceptance differs",
);
assert(
  receipts.length === 609
    && gaps.length === 31
    && gaps.every(({ blocking }) => blocking === false),
  "receipt or nonblocking-gap denominator differs",
);
assert(
  schemaLock.schema.tables.length === 65
    && visual.workbooks.reduce(
      (sum, workbook) => sum + workbook.sheets.length,
      0,
    ) === 65,
  "Sora or visual table denominator differs",
);
assert(
  workbookSemantic
    === "63c7d1ede0b08b3205545316e94840105b270d23fcd338d9cd288068c2b75a92",
  "workbook semantic digest differs",
);

const evidence = {
  schema_revision: "starclock.swarm-disaster-reference-release.v1",
  goal_id: "swarm-disaster-reference-v1",
  result: "CandidateReferenceComplete",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    profile: "swarm-disaster.profile.v1",
  },
  commits,
  content: {
    source_inventory_files: 2_882,
    manifest_categories: 42,
    manifest_obligations: manifest.frozen_source_obligations,
    data_ready: manifest.data_ready_source_obligations,
    coverage_percent: manifest.coverage_percent,
    normalized_files: releaseAudit.denominators.normalized_files,
    normalized_records: releaseAudit.denominators.normalized_records,
    bilingual_records: releaseAudit.denominators.bilingual_records,
    provenance_rows: releaseAudit.denominators.source_records,
    nonblocking_boundaries: gaps.length,
    blocking_boundaries: 0,
  },
  semantics: {
    mechanic_families: semanticAudit.denominators.mechanic_families,
    mechanic_rules: semanticAudit.denominators.mechanic_rules,
    fixtures: semanticAudit.denominators.semantic_fixtures,
    ordered_operations: semanticAudit.denominators.ordered_operations_executed,
    expected_facts: semanticAudit.denominators.expected_facts_executed,
    affected_record_bindings:
      semanticAudit.denominators.affected_record_bindings,
    affected_fixture_bindings:
      semanticAudit.denominators.affected_fixture_bindings,
  },
  reconciliation: {
    goal08_commit: integration.reconciliation.goal08_commit,
    receipts: integration.reconciliation.matching_receipts,
    conflicts: integration.reconciliation.conflicts,
    ownership_pairs: integration.reconciliation.ownership_pairs,
  },
  authoring: {
    adapter: "openpyxl==3.1.5",
    schema_export_authority: "sora-cli==0.3.0",
    workbooks: 4,
    primary_tables: 64,
    repeated_field_child_tables: 1,
    workbook_rows: integration.artifacts.debug_rows,
    workbook_semantic_sha256: workbookSemantic,
    visual_review_sheets: 65,
    visual_review_contact_sheets: 10,
    visual_defects: 0,
  },
  digests: {
    source_inventory_sha256: manifest.source_manifest_sha256,
    content_manifest_sha256: manifest.content_manifest_sha256,
    normalized_pack_sha256: packDigest,
    schema_lock_sha256: integration.artifacts.schema_lock_sha256,
    swarm_candidate_bundle_sha256: integration.artifacts.bundle_sha256,
    swarm_candidate_bundle_bytes: integration.artifacts.bundle_bytes,
    debug_tree_sha256: integration.artifacts.debug_tree_sha256,
    release_audit_sha256: sha256File(
      "evidence/swarm-disaster-reference-v1/release-audit.json",
    ),
    semantic_fixture_audit_sha256: sha256File(
      "evidence/swarm-disaster-reference-v1/semantic-fixture-audit.json",
    ),
    integration_acceptance_sha256: sha256File(
      "evidence/swarm-disaster-reference-v1/integration-acceptance.json",
    ),
    visual_review_sha256: sha256File(
      "evidence/swarm-disaster-reference-v1/visual-review.json",
    ),
  },
  preserved: integration.preserved,
  runtime_boundary: {
    release_lane: "Candidate",
    bundle_role: "AuthoringAndReviewOnly",
    runtime_loading: "ForbiddenReferenceOnly",
    runtime_lowering: false,
    json_runtime_path: false,
    standard_gold_or_production_mutation: false,
  },
  acceptance: {
    source_cache_command:
      "node tools/swarm-disaster-reference/run-acceptance.mjs --with-source-cache",
    source_cache_commands: 47,
    clean_checkout_command:
      "node tools/swarm-disaster-reference/run-clean-checkout.mjs",
    clean_checkout_commands: 11,
    clean_checkout_fresh_target: true,
    clean_checkout_inherited_source_cache: false,
    repository_quick_gate: "pass",
    repository_full_gate_external_boundary:
      "Goal 06 Cargo.lock baseline differs before Goal 09 generated checks",
  },
  publication: {
    remote: "origin",
    branch: "codex/goal09-swarm-disaster-reference",
    published_through: commits.integration_acceptance,
    completion_commit: "This row's containing commit (G09-P4-B4)",
  },
};
const output = `${JSON.stringify(evidence, null, 2)}\n`;
const evidencePath = path.join(root, evidenceRelative);
if (write) {
  fs.mkdirSync(path.dirname(evidencePath), { recursive: true });
  fs.writeFileSync(evidencePath, output);
} else {
  assert(fs.existsSync(evidencePath),
    "release evidence is missing; run with --write");
  assert(fs.readFileSync(evidencePath, "utf8") === output,
    "release evidence has generated drift");
}
console.log(
  `Goal 09 Candidate release ${write ? "written" : "verified"} ` +
  `(${manifest.data_ready_source_obligations}/` +
  `${manifest.frozen_source_obligations} DataReady; ` +
  `${schemaLock.schema.tables.length} Sora tables; ` +
  `${semanticAudit.denominators.mechanic_families} fixture families).`,
);

function workbookSemanticDigest() {
  const python = process.env.STARCLOCK_PYTHON ?? "python3";
  const script = [
    "import sys",
    "from pathlib import Path",
    "sys.path.insert(0, str(Path('tools/swarm-disaster-reference').resolve()))",
    "from workbook_authoring import semantic_digest",
    "print(semantic_digest(Path('config/swarm-disaster/data').resolve()))",
  ].join("; ");
  const result = spawnSync(python, ["-c", script], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      PYTHONDONTWRITEBYTECODE: "1",
    },
  });
  assert(result.status === 0,
    `workbook semantic digest failed: ${result.stderr}`);
  return result.stdout.trim();
}

function runGit(arguments__) {
  execFileSync("git", arguments__, { cwd: root, stdio: "ignore" });
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function json(relative) {
  return JSON.parse(text(relative));
}

function sha256File(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
