#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const write = args.includes("--write");
const root = path.resolve(
  args.find((argument) => !argument.startsWith("--")) ?? ".",
);
const localSourcesPath = path.join(
  root,
  "content-reference",
  "divergent-universe-v1",
  "sources.json",
);
const outputPath = path.join(
  root,
  "evidence",
  "divergent-universe-reference-v1",
  "reconciliation-checkpoints.json",
);
const definitions = [
  {
    goal: "Goal08",
    goal_id: "gold-and-gears-reference-v1",
    commit: "2688624c34a564d87076cadb405c8da506efd373",
    content_commit: "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    source_transport: "LocalCommittedReleaseRegistration",
    sources_path: "content-reference/gold-and-gears-v1/sources.json",
    sources_sha256:
      "d4df241d6a4cdfc2168be29749ca92e24b49721a85e1422720c6d85b376452e9",
    source_records: 9_082,
    exact_matches: 53,
    same_locator_different_digest: 69,
    status_path: "docs/goals/08-gold-and-gears-reference-data-status.md",
  },
  {
    goal: "Goal09",
    goal_id: "swarm-disaster-reference-v1",
    commit: "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    content_commit: "b8da6744a63cd92554b45f8e780d79a1be131f50",
    source_transport: "RemoteBranch",
    remote_ref: "origin/codex/goal09-swarm-disaster-reference",
    sources_path: "content-reference/swarm-disaster-v1/sources.json",
    sources_sha256:
      "e7fdeef0405cdbf746ba0c8cf18ae25975efdd84d75235728d3d43dfb7a05884",
    source_records: 8_139,
    exact_matches: 45,
    same_locator_different_digest: 57,
    status_path: "docs/goals/09-swarm-disaster-reference-data-status.md",
  },
  {
    goal: "Goal10",
    goal_id: "unknowable-domain-reference-v1",
    commit: "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    content_commit: "a2e64e1ddf40dd5e4570e576650be0472044794d",
    source_transport: "RemoteBranch",
    remote_ref: "origin/codex/goal10-unknowable-domain-reference",
    sources_path: "content-reference/unknowable-domain-v1/sources.json",
    sources_sha256:
      "6b118bdd12e14c67b6edf1ff209efd39ce65ec0c120a76141e0e3e5854d15494",
    source_records: 4_473,
    exact_matches: 4,
    same_locator_different_digest: 55,
    status_path: "docs/goals/10-unknowable-domain-reference-data-status.md",
  },
];

const localSources = json(localSourcesPath).filter(
  ({ mechanism_quality: quality }) => quality !== "ReconciliationCheckpoint",
);
assert(localSources.length === 7_620, "Goal 11 source denominator differs");
const localByTriple = uniqueMap(localSources, tripleKey, "Goal 11 source triple");
const localByLocator = uniqueMap(
  localSources,
  locatorKey,
  "Goal 11 source locator",
);
const checkpoints = definitions.map(buildCheckpoint);
const report = {
  schema_revision: "starclock.divergent-universe-reconciliation-checkpoints.v1",
  goal_id: "divergent-universe-reference-v1",
  frozen_at: "2026-07-29",
  purpose:
    "Compact source-path, row-locator and evidence-digest reconciliation " +
    "against completed Goal 08/09/10 facts; foreign content is not imported.",
  join_key: ["source_path", "row_locator", "evidence_sha256"],
  result: "pass",
  goal11_source_records: localSources.length,
  goal11_source_triples_sha256: sha256(
    canonical([...localByTriple.keys()].sort()),
  ),
  summary: {
    checkpoints: checkpoints.length,
    exact_shared_source_records: checkpoints.reduce(
      (sum, checkpoint) => sum + checkpoint.exact_match_count,
      0,
    ),
    same_locator_different_digest: checkpoints.reduce(
      (sum, checkpoint) =>
        sum + checkpoint.same_locator_different_digest_count,
      0,
    ),
    conflicts: 0,
  },
  checkpoints,
};
assert(
  report.summary.exact_shared_source_records === 102 &&
    report.summary.same_locator_different_digest === 181,
  "aggregate reconciliation denominator differs",
);
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (write) {
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  fs.writeFileSync(outputPath, encoded);
} else {
  assert(fs.existsSync(outputPath), "checkpoint evidence is missing; run --write");
  assert(
    fs.readFileSync(outputPath, "utf8") === encoded,
    "checkpoint evidence drifted",
  );
}
console.log(
  `Divergent Universe reconciliation checkpoints verified (` +
    `${checkpoints.map(({ goal, exact_match_count: count }) =>
      `${goal}=${count}`).join("; ")} exact shared-source records; ` +
    `${report.summary.same_locator_different_digest} non-join digest ` +
    "representations; zero conflicts).",
);

function buildCheckpoint(definition) {
  assertCommit(definition.commit);
  assertCommit(definition.content_commit);
  capture("git", [
    "merge-base",
    "--is-ancestor",
    definition.content_commit,
    definition.commit,
  ]);
  if (definition.remote_ref) {
    capture("git", [
      "merge-base",
      "--is-ancestor",
      definition.commit,
      definition.remote_ref,
    ]);
  }
  const status = gitBlob(definition.commit, definition.status_path);
  assert(
    status.includes("| State | `Complete` |"),
    `${definition.goal}: completed status is missing`,
  );
  const bytes = gitBlob(definition.commit, definition.sources_path);
  assert(
    sha256(bytes) === definition.sources_sha256,
    `${definition.goal}: sources digest differs`,
  );
  const foreignSources = JSON.parse(bytes);
  assert(
    foreignSources.length === definition.source_records,
    `${definition.goal}: source denominator differs`,
  );
  const normalized = foreignSources.map(normalizeForeignSource);
  const foreignByTriple = uniqueMap(
    normalized,
    tripleKey,
    `${definition.goal} source triple`,
  );
  const foreignByLocator = uniqueMap(
    normalized,
    locatorKey,
    `${definition.goal} source locator`,
  );
  const exactMatches = [];
  for (const source of localSources) {
    const foreign = foreignByTriple.get(tripleKey(source));
    if (!foreign) continue;
    exactMatches.push({
      source_path: source.path,
      row_locator: source.locator,
      evidence_sha256: source.sha256,
      goal11_source_id: source.source_id,
      checkpoint_source_id: foreign.source_id,
      repository: source.repository,
      revision: source.revision,
      evidence_quality: source.evidence_quality,
    });
  }
  const digestRepresentations = [];
  for (const source of localSources) {
    const foreign = foreignByLocator.get(locatorKey(source));
    if (!foreign || foreign.sha256 === source.sha256) continue;
    digestRepresentations.push({
      source_path: source.path,
      row_locator: source.locator,
      goal11_evidence_sha256: source.sha256,
      checkpoint_evidence_sha256: foreign.sha256,
      goal11_source_id: source.source_id,
      checkpoint_source_id: foreign.source_id,
      shared_repository: source.repository === foreign.repository,
      shared_revision: source.revision === foreign.revision,
      disposition:
        "NotAJoin; retain both evidence representations without overwrite.",
    });
  }
  exactMatches.sort(compareReconciliation);
  digestRepresentations.sort(compareReconciliation);
  assert(
    exactMatches.length === definition.exact_matches,
    `${definition.goal}: exact-match denominator differs`,
  );
  assert(
    digestRepresentations.length ===
      definition.same_locator_different_digest,
    `${definition.goal}: digest-representation denominator differs`,
  );
  assert(
    digestRepresentations.every(
      ({ shared_repository: repository, shared_revision: revision }) =>
        repository && revision,
    ),
    `${definition.goal}: non-join representation crosses source revisions`,
  );
  return {
    goal: definition.goal,
    goal_id: definition.goal_id,
    commit: definition.commit,
    content_commit: definition.content_commit,
    source_transport: definition.source_transport,
    ...(definition.remote_ref ? { remote_ref: definition.remote_ref } : {}),
    sources_path: definition.sources_path,
    sources_sha256: definition.sources_sha256,
    source_record_count: foreignSources.length,
    source_triples_sha256: sha256(
      canonical([...foreignByTriple.keys()].sort()),
    ),
    exact_match_count: exactMatches.length,
    exact_matches_sha256: sha256(canonical(exactMatches)),
    exact_matches: exactMatches,
    same_locator_different_digest_count: digestRepresentations.length,
    same_locator_different_digest_sha256: sha256(
      canonical(digestRepresentations),
    ),
    same_locator_different_digest: digestRepresentations,
    conflicts: 0,
  };
}

function normalizeForeignSource(source) {
  const result = {
    source_id: source.source_id,
    repository: source.repository ?? source.repository_or_url,
    revision: source.revision ?? source.revision_or_access_date,
    path: source.path ?? source.relative_path_or_page,
    locator: source.locator ?? source.row_locator,
    sha256: source.sha256 ?? source.evidence_sha256,
    evidence_quality: source.evidence_quality,
  };
  for (const [field, value] of Object.entries(result)) {
    assert(
      typeof value === "string" && value.length > 0,
      `${source.source_id}: normalized ${field} is empty`,
    );
  }
  return result;
}

function compareReconciliation(left, right) {
  return left.source_path.localeCompare(right.source_path) ||
    left.row_locator.localeCompare(right.row_locator) ||
    (left.evidence_sha256 ?? left.goal11_evidence_sha256).localeCompare(
      right.evidence_sha256 ?? right.goal11_evidence_sha256,
    );
}

function tripleKey(source) {
  return `${source.path}\0${source.locator}\0${source.sha256}`;
}

function locatorKey(source) {
  return `${source.path}\0${source.locator}`;
}

function uniqueMap(values, keyOf, label) {
  const result = new Map();
  for (const value of values) {
    const key = keyOf(value);
    assert(!result.has(key), `duplicate ${label}: ${key}`);
    result.set(key, value);
  }
  return result;
}

function assertCommit(commit) {
  capture("git", ["cat-file", "-e", `${commit}^{commit}`]);
}

function gitBlob(commit, relative) {
  return capture("git", ["show", `${commit}:${relative}`]);
}

function capture(command, commandArgs) {
  return execFileSync(command, commandArgs, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
