#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";

const hasRoot = Boolean(process.argv[2] && !process.argv[2].startsWith("--"));
const root = path.resolve(hasRoot ? process.argv[2] : ".");
const options = process.argv.slice(hasRoot ? 3 : 2);
assert(options.every((option) => option === "--bless"),
  "usage: verify-release-audits.mjs [root] [--bless]");
const bless = options.includes("--bless");
const policy = json("policy/goal07-release-audits.json");
const expected = policy.denominators;
assert(policy.schema_revision === "starclock.goal07-release-audits.v1",
  "unexpected Goal 07 release-audit policy revision");
assert(Object.values(policy.contracts).every((value) => value === true),
  "Goal 07 release-audit contract is incomplete");

for (const command of policy.required_verifiers)
  execFileSync(command[0], command.slice(1), { cwd: root, stdio: "inherit" });

const manifest = json(
  "content-manifests/standard-universe-mechanics-complete-v1/content-partitions.json",
);
const progress = json(
  "evidence/standard-universe-mechanics-complete-v1/content-progress.json",
);
const register = json(
  "content-manifests/standard-universe-mechanics-complete-v1/evidence-and-approximation-register.json",
);
const receiptRoot = "evidence/standard-universe-mechanics-complete-v1/partitions";
assert(manifest.partitions.length === expected.generated_content_batches,
  "generated content-batch denominator drift");
assert(progress.result === "complete"
  && progress.completed_partitions === expected.generated_content_batches
  && progress.pending_partitions === 0,
"generated content progress is not complete");

const dimensions = {
  records: { manifest: "record_ids", expected: expected.content_records },
  rules: { manifest: "rule_ids", expected: expected.rules },
  fixtures: { manifest: "fixture_ids", expected: expected.semantic_fixtures },
  enemy_variants: { manifest: "enemy_variant_ids", expected: expected.enemy_variants },
  encounter_members: {
    manifest: "encounter_member_ids",
    expected: expected.encounter_members,
  },
};
const coverage = Object.fromEntries(
  Object.keys(dimensions).map((name) => [name, []]),
);
let nativeReviews = 0;
let admittedNativeHandlers = 0;
let externalDecisionRecords = 0;

for (const partition of manifest.partitions) {
  const receipt = json(`${receiptRoot}/${partition.id}.json`);
  assert(receipt.partition_id === partition.id
    && receipt.goal_id === policy.goal_id
    && receipt.state === "Complete",
  `${partition.id}: completion receipt identity/state differs`);
  assert(receipt.execution?.result === "pass",
    `${partition.id}: partition execution is not passing`);
  assert(receipt.authoring?.workbooks?.length > 0
    && receipt.authoring?.sora_bundle?.path
    && receipt.authoring?.sora_golden?.path,
  `${partition.id}: Excel/Sora authoring evidence is incomplete`);

  for (const [name, dimension] of Object.entries(dimensions)) {
    const assigned = partition[dimension.manifest];
    const entries = receipt[name];
    assert(Array.isArray(entries), `${partition.id}: ${name} receipt section missing`);
    exactIds(`${partition.id}/${name}`, assigned, entries.map(({ id }) => id));
    for (const entry of entries) {
      assert(nonEmpty(entry.runtime_disposition),
        `${partition.id}/${name}/${entry.id}: runtime disposition missing`);
      assert(entry.workbook_evidence?.length > 0
        && entry.provenance_evidence?.length > 0,
      `${partition.id}/${name}/${entry.id}: workbook/provenance evidence missing`);
      coverage[name].push(entry.id);
      if (name === "records" && entry.runtime_disposition === "ExternalDecision")
        externalDecisionRecords += 1;
    }
  }

  exactIds(
    `${partition.id}/native reviews`,
    partition.native_review_candidate_rule_ids,
    receipt.native_handler_reviews.map(({ id }) => id),
  );
  for (const review of receipt.native_handler_reviews) {
    assert(["IrSufficient", "Admitted"].includes(review.outcome)
      && nonEmpty(review.decision)
      && review.evidence?.length > 0,
    `${partition.id}/${review.id}: native review is not terminal`);
    nativeReviews += 1;
    if (review.outcome === "Admitted") admittedNativeHandlers += 1;
  }
}

for (const [name, dimension] of Object.entries(dimensions)) {
  assert(coverage[name].length === dimension.expected,
    `${name}: receipt denominator drift`);
  assert(new Set(coverage[name]).size === dimension.expected,
    `${name}: assignments are not exact-once`);
}
assert(nativeReviews === expected.native_review_candidates,
  "native review-candidate denominator drift");
assert(admittedNativeHandlers === expected.admitted_native_handlers,
  "an unapproved native handler was admitted");

const registeredExternal = new Set(register.project_policy_records.map(({ id }) => id));
const completedExternal = new Set();
for (const partition of manifest.partitions) {
  const receipt = json(`${receiptRoot}/${partition.id}.json`);
  for (const entry of receipt.records)
    if (entry.runtime_disposition === "ExternalDecision") completedExternal.add(entry.id);
}
assert(externalDecisionRecords === expected.external_decision_records
  && registeredExternal.size === expected.external_decision_records,
"external-decision denominator drift");
exactIds("registered external decisions", [...registeredExternal], [...completedExternal]);

const reviews = loadEnemySourceReviews();
assert(reviews.length === expected.enemy_variants
  && new Set(reviews.map(({ enemy_variant_id }) => enemy_variant_id)).size
    === expected.enemy_variants,
"enemy source reviews do not exactly cover 86 variants");
exactIds("enemy source reviews", coverage.enemy_variants,
  reviews.map(({ enemy_variant_id }) => enemy_variant_id));
for (const review of reviews) {
  assert(review.mechanic_status === "ExecutableMechanismCorrect",
    `${review.enemy_variant_id}: mechanic evidence is not executable/exact`);
  assert(review.numeric_status === "ApprovedPerVariantInputs"
    && nonEmpty(review.numeric_policy_id),
  `${review.enemy_variant_id}: numeric evidence/policy is not terminal`);
  assert(["ExactPublic", "ApprovedNumericApproximation"].includes(
    review.accuracy_disposition,
  ), `${review.enemy_variant_id}: numeric disposition is not terminal`);
  assert(review.evidence_paths.length > 0,
    `${review.enemy_variant_id}: no numeric evidence path`);
  for (const entry of review.evidence_paths)
    assert(exists(entry.path) && sha256(entry.path) === entry.sha256,
      `${review.enemy_variant_id}: numeric evidence hash differs`);
}

const candidates = new Set(register.enemy_numeric_approximations.map(({ id }) => id));
assert(candidates.size === expected.numeric_approximation_candidates,
  "Phase 0 numeric candidate denominator drift");
const candidateReviews = reviews.filter(({ enemy_variant_id }) =>
  candidates.has(enemy_variant_id));
assert(candidateReviews.length === candidates.size,
  "a Phase 0 numeric candidate lacks a final source review");
const upgraded = candidateReviews.filter(({ accuracy_disposition }) =>
  accuracy_disposition === "ExactPublic").length;
const approved = candidateReviews.filter(({ accuracy_disposition }) =>
  accuracy_disposition === "ApprovedNumericApproximation").length;
assert(upgraded === expected.candidates_upgraded_to_exact_public
  && approved === expected.approved_numeric_approximations,
"numeric candidate terminal-disposition counts drift");
assert(reviews
  .filter(({ accuracy_disposition }) => accuracy_disposition
    === "ApprovedNumericApproximation")
  .every(({ enemy_variant_id }) => candidates.has(enemy_variant_id)),
"an unregistered numeric approximation entered production");

const dependency = verifyDependencyDelta();
const releaseCoverage = json("content-reference/standard-universe-v1/coverage.json");
const sources = json("content-reference/standard-universe-v1/sources.json");
const production = json("config/production-golden.json");
const nativePolicy = json("policy/native-handler-audit.json");
const sourcePolicy = json("policy/repository-checks.json");
assert(releaseCoverage.required === expected.content_records
  && releaseCoverage.data_ready === expected.content_records
  && releaseCoverage.coverage_percent === "100",
"release coverage denominator drift");
assert(sources.length === expected.provenance_rows
  && sources.every((source) => nonEmpty(source.license_note)),
"provenance/license inventory drift");
assert(production.table_count === expected.production_config_tables
  && production.identity_count === expected.production_content_identities,
"production Excel/Sora denominator drift");
assert(nativePolicy.admitted_handlers.length === expected.admitted_native_handlers,
  "native-handler policy differs from completion receipts");

const report = {
  schema_revision: "starclock.goal07-release-audits-evidence.v1",
  goal_id: policy.goal_id,
  batch: policy.batch,
  result: "coverage-provenance-bilingual-workbook-sora-dependency-license-native-source-and-approximation-audits-pass",
  coverage: {
    generated_content_batches: manifest.partitions.length,
    content_records: coverage.records.length,
    rules: coverage.rules.length,
    semantic_fixtures: coverage.fixtures.length,
    enemy_variants: coverage.enemy_variants.length,
    encounter_members: coverage.encounter_members.length,
    exact_once_assignment_gaps: 0,
    pending_partitions: 0,
  },
  provenance_bilingual_license: {
    content_rows: releaseCoverage.required,
    data_ready_rows: releaseCoverage.data_ready,
    coverage_percent: releaseCoverage.coverage_percent,
    provenance_rows: sources.length,
    missing_license_notes: 0,
    release_pack_sha256: treeSha256("content-reference/standard-universe-v1"),
  },
  workbook_sora_drift: {
    production_tables: production.table_count,
    production_content_identities: production.identity_count,
    production_output_digest: production.output_digest,
    production_bundle_sha256: sha256("config/generated/config.sora"),
    universe_bundle_sha256: sha256("config/universe-generated/config.sora"),
    drift: 0,
  },
  dependency_license: dependency,
  native_handler: {
    reviewed_candidates: nativeReviews,
    admitted_handlers: admittedNativeHandlers,
    registry_revision: nativePolicy.registry_revision,
  },
  source_structure: {
    maximum_handwritten_lines:
      sourcePolicy.rust_source.maximum_handwritten_lines,
    maximum_facade_lines: sourcePolicy.rust_source.maximum_facade_lines,
    line_limit_exceptions:
      sourcePolicy.rust_source.line_limit_exceptions.length,
    generated_or_vendor_exclusions:
      sourcePolicy.rust_source.excluded_roots.length,
  },
  approximation: {
    registered_external_decisions: completedExternal.size,
    numeric_candidates: candidates.size,
    upgraded_to_exact_public: upgraded,
    approved_numeric_approximations: approved,
    unresolved_numeric_candidates: 0,
    mechanism_correct_enemy_variants: reviews.length,
    mechanic_approximations: 0,
    source_review_sha256: treeSha256(
      "evidence/standard-universe-mechanics-complete-v1/source-reviews",
    ),
  },
  policy_sha256: sha256("policy/goal07-release-audits.json"),
};
const relative =
  "evidence/standard-universe-mechanics-complete-v1/audits/release-audits.json";
const output = `${JSON.stringify(report, null, 2)}\n`;
if (bless) {
  fs.mkdirSync(path.dirname(absolute(relative)), { recursive: true });
  fs.writeFileSync(absolute(relative), output);
} else {
  assert(exists(relative), `${relative} is missing; run with --bless`);
  assert(text(relative).replaceAll("\r\n", "\n") === output,
    `${relative} is stale; run with --bless`);
}
console.log(
  `Goal 07 release audits verified (${manifest.partitions.length} receipts, ` +
  `${sources.length} provenance rows, ${nativeReviews} native reviews, ` +
  `${candidates.size} numeric candidates, zero unresolved gaps).`,
);

function loadEnemySourceReviews() {
  const directory = absolute(
    "evidence/standard-universe-mechanics-complete-v1/source-reviews",
  );
  const files = fs.readdirSync(directory)
    .filter((file) => /^G07-P5-M15-S\d+\.json$/u.test(file))
    .sort();
  const output = [];
  for (const file of files) {
    const review = JSON.parse(fs.readFileSync(path.join(directory, file), "utf8"));
    if (review.schema_revision === "starclock.goal07-enemy-source-review.v1") {
      const evidence = new Map();
      for (const row of review.numeric_evidence) {
        const relative = row.source_url_or_committed_evidence_path;
        assert(nonEmpty(relative) && /^[0-9a-f]{64}$/u.test(row.evidence_hash),
          `${review.partition_id}: malformed numeric evidence row`);
        evidence.set(relative, row.evidence_hash);
      }
      output.push({
        enemy_variant_id: review.enemy_variant_id,
        accuracy_disposition: review.accuracy_disposition,
        numeric_policy_id: review.numeric_policy_id,
        numeric_status: review.numeric_status,
        mechanic_status: review.mechanic_status,
        evidence_paths: [...evidence].map(([entryPath, digest]) => ({
          path: entryPath,
          sha256: digest,
        })),
      });
      continue;
    }
    assert(review.schema_revision === "starclock.goal07-enemy-source-review.v2",
      `${file}: unsupported enemy source-review revision`);
    for (const variant of review.variants) {
      output.push({
        enemy_variant_id: variant.enemy_variant_id,
        accuracy_disposition: variant.accuracy_disposition,
        numeric_policy_id: variant.numeric_policy_id,
        numeric_status: review.numeric_status,
        mechanic_status: review.mechanic_status,
        evidence_paths: [{
          path: variant.numeric_evidence_path,
          sha256: variant.numeric_evidence_sha256,
        }],
      });
    }
  }
  return output;
}

function verifyDependencyDelta() {
  const baseline = policy.dependency_baseline_commit;
  const baselineLock = capture("git", ["show", `${baseline}:Cargo.lock`]);
  const baselineRegistry = registryPackages(baselineLock);
  const currentRegistry = registryPackages(text("Cargo.lock"));
  exactIds("registry package identities", baselineRegistry, currentRegistry);
  assert(currentRegistry.length === expected.reviewed_registry_packages,
    "reviewed registry-package denominator drift");

  const changedManifests = lines(capture("git", [
    "diff", "--name-only", baseline, "--", "Cargo.toml",
    ":(glob)crates/*/Cargo.toml",
  ]));
  const expectedManifests = [...new Set(
    policy.dependency_delta.reviewed_workspace_edges.map(({ manifest }) => manifest),
  )].sort();
  exactIds("reviewed changed manifests", expectedManifests,
    changedManifests.sort());
  for (const edge of policy.dependency_delta.reviewed_workspace_edges) {
    const current = text(edge.manifest);
    const before = capture("git", ["show", `${baseline}:${edge.manifest}`]);
    const marker = `${edge.to} = { path = "../${edge.to}" }`;
    assert(!before.includes(marker) && current.includes(marker),
      `${edge.from} -> ${edge.to}: reviewed workspace edge differs`);
    assert(nonEmpty(edge.reason) && nonEmpty(edge.license_disposition),
      `${edge.from} -> ${edge.to}: review rationale/license missing`);
  }
  assert(policy.dependency_delta.new_registry_packages.length === 0,
    "Goal 07 dependency policy admits a registry package");
  return {
    baseline_commit: baseline,
    reviewed_registry_packages: currentRegistry.length,
    new_registry_packages: [],
    reviewed_workspace_edges: policy.dependency_delta.reviewed_workspace_edges,
    baseline_cargo_lock_sha256: crypto.createHash("sha256")
      .update(baselineLock).digest("hex"),
    current_cargo_lock_sha256: sha256("Cargo.lock"),
  };
}

function registryPackages(lock) {
  return lock.split("[[package]]").slice(1).flatMap((block) => {
    const source = block.match(/^source = "([^"]+)"/mu)?.[1];
    if (!source?.startsWith("registry+")) return [];
    const name = block.match(/^name = "([^"]+)"/mu)?.[1];
    const version = block.match(/^version = "([^"]+)"/mu)?.[1];
    assert(name && version, "Cargo.lock registry package identity malformed");
    return [`${name}@${version}`];
  }).sort();
}
function exactIds(label, expectedIds, actualIds) {
  const expectedSorted = [...expectedIds].sort();
  const actualSorted = [...actualIds].sort();
  assert(new Set(actualSorted).size === actualSorted.length,
    `${label}: duplicate identity`);
  assert(JSON.stringify(expectedSorted) === JSON.stringify(actualSorted),
    `${label}: exact set differs`);
}
function treeSha256(relative) {
  const digest = crypto.createHash("sha256");
  for (const file of walk(absolute(relative))) {
    digest.update(path.relative(root, file).replaceAll("\\", "/"));
    digest.update("\0");
    digest.update(fs.readFileSync(file));
    digest.update("\0");
  }
  return digest.digest("hex");
}
function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true })
    .sort((left, right) => left.name.localeCompare(right.name))
    .flatMap((entry) => {
      const file = path.join(directory, entry.name);
      return entry.isDirectory() ? walk(file) : [file];
    });
}
function capture(command, args) {
  return execFileSync(command, args, { cwd: root, encoding: "utf8" }).trim();
}
function lines(value) { return value.split(/\r?\n/u).filter(Boolean); }
function sha256(relative) {
  return crypto.createHash("sha256")
    .update(fs.readFileSync(absolute(relative))).digest("hex");
}
function absolute(relative) { return path.join(root, relative); }
function exists(relative) {
  return fs.statSync(absolute(relative), { throwIfNoEntry: false })?.isFile();
}
function text(relative) { return fs.readFileSync(absolute(relative), "utf8"); }
function json(relative) { return JSON.parse(text(relative)); }
function nonEmpty(value) {
  return typeof value === "string" && value.trim().length > 0;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
