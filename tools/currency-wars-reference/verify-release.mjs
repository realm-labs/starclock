#!/usr/bin/env node

import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
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
  arguments_.find((argument) => !argument.startsWith("--"))
    ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../.."),
);
const outputRelative =
  "evidence/currency-wars-reference-v1/release/release-evidence.json";
const status = text("docs/goals/12-currency-wars-reference-data-status.md");
const plan = text("docs/goals/12-currency-wars-reference-data.md");
const goalIndex = text("docs/goals/README.md");

assert(status.includes("| State | `Complete` |"), "Goal 12 is not Complete");
assert(
  status.includes("| Current phase | Complete |")
  && status.includes("| Current batch | None |")
  && status.includes("| Next unblocked batch | None |"),
  "Goal 12 still has active or pending work",
);
assert(
  (status.match(/^\| Phase [0-4].*\| `Complete` \|/gmu) ?? []).length === 5,
  "not every Goal 12 phase is Complete",
);
assert(
  (status.match(/^\| `G12-P[0-4]-B(?:10|[1-9])` \| `Complete` \|/gmu)
    ?? []).length === 31,
  "not every Goal 12 batch is Complete",
);
assert(!status.includes("- [ ]"), "Goal 12 terminal checklist is incomplete");
assert(
  status.includes(
    "| Completion commit | This row's containing commit (`G12-P4-B4`) |",
  ),
  "Goal 12 completion record is missing",
);
assert(
  goalIndex.includes(
    "| Goal 12 — Currency Wars Reference Data | Version 4.4 Currency Wars " +
      "manifests, flow/Squad-HP/economy/roster/star/Bond/Empowerment " +
      "mechanics, provenance, isolated Excel/Sora authoring and review " +
      "fixtures; no runtime | Complete |",
  ),
  "Goal index does not mark Goal 12 Complete",
);
const planBatches = batchSet(plan);
const statusBatches = batchSet(status);
assert(
  planBatches.length === 31
  && JSON.stringify(planBatches) === JSON.stringify(statusBatches),
  "Goal 12 plan/status batch sets differ",
);
const localMarkdownLinks = verifyLocalMarkdownLinks();

const inventory = json(
  "content-manifests/currency-wars-v1/source-inventory.json",
);
const contentManifest = json(
  "content-manifests/currency-wars-v1/content-manifest.json",
);
const [manifest] = json("content-reference/currency-wars-v1/manifest.json");
const [packIndex] = json("content-reference/currency-wars-v1/pack-index.json");
const coverage = json("content-reference/currency-wars-v1/coverage.json");
const rules = json("content-reference/currency-wars-v1/mechanic-rules.json");
const sources = json("content-reference/currency-wars-v1/sources.json");
const fixtures = json("content-reference/currency-wars-v1/review-fixtures.json");
const gaps = json("content-reference/currency-wars-v1/research-gaps.json");
const receipts = json(
  "content-reference/currency-wars-v1/reconciliation-receipts.json",
);
const schema = json("config/currency-wars-generated/schema.lock").schema;
const ownership = json(
  "evidence/currency-wars-reference-v1/p4b3-ownership-audit.json",
);
const reconciliation = json(
  "evidence/currency-wars-reference-v1/p4b3-reconciliation-audit.json",
);
const semantic = json(
  "evidence/currency-wars-reference-v1/p4b2-semantic-fixture-results.json",
);
const acceptance = json(
  "evidence/currency-wars-reference-v1/p4b3-release-acceptance.json",
);

const obligations = Object.values(contentManifest.categories)
  .reduce((sum, category) => sum + category.count, 0);
const dataReady = coverage.filter(({ state }) => state === "DataReady").length;
const excluded = coverage.filter(({ state }) => state === "Excluded").length;
assert(
  inventory.counts.total === 3_822
  && obligations === 19_250
  && coverage.length === obligations
  && dataReady === 18_524
  && excluded === 726,
  "release denominator differs",
);
assert(
  manifest.normalized_files.length === 102
  && packIndex.file_digests.length === 101
  && ownership.normalized.row_count === 74_850
  && rules.length === 2_367
  && sources.length === 37_342
  && fixtures.length === 28
  && gaps.length === 12
  && gaps.every(({ tags }) =>
    tags.includes("nonblocking") && !tags.includes("blocking")
  )
  && receipts.length === 4,
  "normalized release content differs",
);
assert(
  ownership.result === "Pass"
  && reconciliation.result === "Pass"
  && semantic.result === "Pass"
  && acceptance.result === "Pass"
  && acceptance.runtime_boundary.runtime_loading === false
  && acceptance.runtime_boundary.runtime_lowering === false,
  "one or more release evidence layers do not pass",
);
assert(
  schema.tables.length === 102
  && acceptance.authoring.tables === 102
  && acceptance.authoring.rows === 74_850
  && acceptance.authoring.visual_review.sheets === 102
  && acceptance.authoring.visual_review.defects === 0
  && acceptance.authoring.bundle.sha256
    === "a4569997990727739db74a2d942e6b13a84d2466b0fe3723acb92c7406ae8571",
  "authoring release identity differs",
);

const evidence = {
  schema_revision: "starclock.currency-wars-reference-release.v1",
  goal_id: "currency-wars-reference-v1",
  completed_on: "2026-07-30",
  result: "Complete",
  delivery_state: "Candidate",
  snapshot: {
    game_version: "4.4",
    access_date: "2026-07-22",
    mode_family: "currency-wars",
    profile_id: "currency-wars.profile.v1",
  },
  source_revisions: Object.fromEntries(
    inventory.snapshot.repositories.map(({ id, revision }) => [id, revision]),
  ),
  checkpoints: {
    sora_and_visual_qa: "c16161b989825066c1e32ac7e79b1d4b3c15657e",
    ownership_audit: "8284c1a066e4d4c3ec893b9ae6259cfb77f668bb",
    semantic_fixtures: "7d8dfb3dd69878346761b99d8a131953ed008ed0",
    reconciliation_and_acceptance:
      "c3cb837f0b7d497851204283a7e14d8117f7dc71",
    release_batch: "G12-P4-B4",
  },
  content: {
    source_inventory_files: inventory.counts.total,
    manifest_categories: Object.keys(contentManifest.categories).length,
    source_obligations: obligations,
    eligible_data_ready: dataReady,
    explicit_exclusions: excluded,
    unresolved: 0,
    normalized_files: manifest.normalized_files.length,
    pre_index_files: packIndex.file_digests.length,
    normalized_rows: ownership.normalized.row_count,
    mechanic_rules: rules.length,
    provenance_rows: sources.length,
    source_references: ownership.normalized.source_reference_count,
    semantic_fixture_families: fixtures.length,
    research_gaps: gaps.length,
    blocking_research_gaps:
      gaps.filter(({ tags }) => tags.includes("blocking")).length,
    reconciliation_receipts: receipts.length,
    reconciliation_exact_overlaps: reconciliation.exact_overlap_count,
    reconciliation_conflicts: reconciliation.conflict_count,
  },
  authoring: {
    adapter: acceptance.authoring.adapter,
    schema_export_authority: acceptance.authoring.schema_export_authority,
    tables: acceptance.authoring.tables,
    workbook_rows: acceptance.authoring.rows,
    verified_empty_tables: acceptance.authoring.verified_empty_tables,
    workbook_semantic_sha256:
      acceptance.authoring.workbook_semantic_sha256,
    workbooks: Object.fromEntries(
      Object.entries(acceptance.authoring.workbooks)
        .map(([name, value]) => [name, value.sha256]),
    ),
  },
  digests: {
    source_inventory_sha256: fileSha256(
      "content-manifests/currency-wars-v1/source-inventory.json",
    ),
    content_manifest_sha256: fileSha256(
      "content-manifests/currency-wars-v1/content-manifest.json",
    ),
    normalized_schema_sha256: fileSha256(
      "content-manifests/currency-wars-v1/normalized-schema.json",
    ),
    normalized_pack_sha256: packIndex.pack_digest,
    pack_index_file_sha256: fileSha256(
      "content-reference/currency-wars-v1/pack-index.json",
    ),
    schema_lock_sha256: fileSha256(
      "config/currency-wars-generated/schema.lock",
    ),
    candidate_bundle_sha256: acceptance.authoring.bundle.sha256,
    candidate_bundle_bytes: acceptance.authoring.bundle.bytes,
    debug_export_sha256: acceptance.authoring.debug_digest,
    ownership_audit_sha256: fileSha256(
      "evidence/currency-wars-reference-v1/p4b3-ownership-audit.json",
    ),
    semantic_fixture_results_sha256: fileSha256(
      "evidence/currency-wars-reference-v1/p4b2-semantic-fixture-results.json",
    ),
    reconciliation_audit_sha256: fileSha256(
      "evidence/currency-wars-reference-v1/p4b3-reconciliation-audit.json",
    ),
    release_acceptance_sha256: fileSha256(
      "evidence/currency-wars-reference-v1/p4b3-release-acceptance.json",
    ),
    visual_review_sha256:
      acceptance.authoring.visual_review.manifest_sha256,
  },
  protected_boundaries: acceptance.protected_boundaries,
  reconciliation: acceptance.reconciliation,
  runtime_boundary: acceptance.runtime_boundary,
  acceptance: {
    focused_checks: "Pass",
    dependency_and_workspace_checks: "Pass",
    protected_mode_and_production_bundles: "Pass",
    clean_checkout: "Pass",
    full_repository_command:
      "node tools/repository-check/run.mjs --full --with-source-cache",
    full_repository_external_boundary:
      "pre-existing Goal 06 Cargo.lock baseline differs after preceding " +
      "release checks pass",
    full_repository_external_boundary_owner: "Goal06HistoricalReleaseContract",
    plan_status_batches: planBatches.length,
    verified_local_markdown_links: localMarkdownLinks,
  },
  publication: {
    remote: "origin",
    branch: "codex/goal12-currency-wars-reference",
    published_through: "c3cb837f0b7d497851204283a7e14d8117f7dc71",
    completion_commit: "This row's containing commit (G12-P4-B4)",
  },
};

const serialized = `${JSON.stringify(evidence, null, 2)}\n`;
const outputPath = path.join(root, outputRelative);
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, serialized);
  console.log(
    `Currency Wars release evidence generated: ${obligations} obligations, ` +
      `${ownership.normalized.row_count} rows, Candidate bundle ` +
      `${acceptance.authoring.bundle.sha256}.`,
  );
} else {
  assert(fs.readFileSync(outputPath, "utf8") === serialized,
    "Currency Wars release evidence drift");
  console.log(
    `Currency Wars release verified: 31 batches, ${obligations} obligations, ` +
      `${ownership.normalized.row_count} rows, ${localMarkdownLinks} links.`,
  );
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}
function json(relative) {
  return JSON.parse(text(relative));
}
function fileSha256(relative) {
  return createHash("sha256")
    .update(fs.readFileSync(path.join(root, relative)))
    .digest("hex");
}
function batchSet(value) {
  return [...new Set(
    [...value.matchAll(/G12-P[0-4]-B(?:10|[1-9])/gu)]
      .map((match) => match[0]),
  )].sort();
}
function verifyLocalMarkdownLinks() {
  const result = spawnSync("git", ["ls-files", "*.md"], {
    cwd: root,
    encoding: "utf8",
  });
  assert(result.status === 0, "cannot list Markdown files");
  let count = 0;
  for (const relative of result.stdout.split(/\r?\n/u).filter(Boolean)) {
    const markdown = text(relative);
    for (const match of markdown.matchAll(/\]\(([^)]+)\)/gu)) {
      let target = match[1].trim();
      if (target.startsWith("<") && target.endsWith(">")) {
        target = target.slice(1, -1);
      }
      if (/^(?:https?:|mailto:|#)/u.test(target)) continue;
      target = decodeURIComponent(target.split("#")[0]);
      if (target === "") continue;
      assert(
        fs.existsSync(path.resolve(root, path.dirname(relative), target)),
        `${relative}: missing local link ${target}`,
      );
      count += 1;
    }
  }
  return count;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
