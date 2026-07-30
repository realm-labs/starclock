#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../..",
);
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "evidence/reference-integration-v1/merged-mode-audit.json",
);
const snapshotPolicy = json("policy/release-snapshots.json");
const contentIndex = text("content-reference/README.md");
const documentationIndex = text("docs/content-reference/README.md");

const modes = [
  {
    goal: "G08",
    goalId: "gold-and-gears-reference-v1",
    mode: "gold-and-gears",
    commit: "b7044fcca0ae20a9f51e89459ebf0b1b3b2c3a09",
    tree: "d5430413258a35a6a988973a9f57966f4daeee7d",
    manifest:
      "content-manifests/gold-and-gears-v1/content-manifest.json",
    evidence:
      "evidence/gold-and-gears-reference-v1/release/release-evidence.json",
    status: "docs/goals/08-gold-and-gears-reference-data-status.md",
    expectedResult: "complete",
    expectedRecords: 7_913,
    normalizedDigestField: "normalized_pack_sha256",
    bundleDigestField: "candidate_bundle_sha256",
  },
  {
    goal: "G09",
    goalId: "swarm-disaster-reference-v1",
    mode: "swarm-disaster",
    commit: "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    tree: "7116ca632ca03f13c8d8b0243338d88bef846093",
    manifest:
      "content-manifests/swarm-disaster-v1/content-manifest.json",
    evidence: "evidence/swarm-disaster-reference-v1/release-evidence.json",
    status: "docs/goals/09-swarm-disaster-reference-data-status.md",
    expectedResult: "CandidateReferenceComplete",
    expectedRecords: 6_963,
    normalizedDigestField: "normalized_pack_sha256",
    bundleDigestField: "swarm_candidate_bundle_sha256",
  },
  {
    goal: "G10",
    goalId: "unknowable-domain-reference-v1",
    mode: "unknowable-domain",
    commit: "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    tree: "7ac9d6242de3b024c6b0149f3f28230edc78da56",
    manifest:
      "content-manifests/unknowable-domain-v1/content-manifest.json",
    evidence:
      "evidence/unknowable-domain-reference-v1/release/release-evidence.json",
    status: "docs/goals/10-unknowable-domain-reference-data-status.md",
    expectedResult: "complete",
    expectedRecords: 5_377,
    normalizedDigestField: "normalized_pack_sha256",
    bundleDigestField: "candidate_bundle_sha256",
  },
  {
    goal: "G11",
    goalId: "divergent-universe-reference-v1",
    mode: "divergent-universe",
    commit: "3071d2c2fa7764c133931756769c9efe7f9dabd2",
    tree: "1cb45215d06549dbc7bb4f38ab331ae4e2e872e2",
    manifest:
      "content-manifests/divergent-universe-v1/content-manifest.json",
    evidence:
      "evidence/divergent-universe-reference-v1/release/release-evidence.json",
    status: "docs/goals/11-divergent-universe-reference-data-status.md",
    expectedResult: "complete",
    expectedRecords: 6_215,
    normalizedDigestField: "normalized_pack_sha256",
    bundleDigestField: "candidate_bundle_sha256",
  },
  {
    goal: "G12",
    goalId: "currency-wars-reference-v1",
    mode: "currency-wars",
    commit: "7d672177524a6b43cfd0ff3a5cb62ce7aa6e4981",
    tree: "3aa951a8708fc8333497df16055aced197dcaa5e",
    manifest:
      "content-manifests/currency-wars-v1/content-manifest.json",
    evidence:
      "evidence/currency-wars-reference-v1/release/release-evidence.json",
    status: "docs/goals/12-currency-wars-reference-data-status.md",
    expectedResult: "Complete",
    expectedRecords: 19_250,
    normalizedDigestField: "normalized_pack_sha256",
    bundleDigestField: "candidate_bundle_sha256",
  },
  {
    goal: "G13",
    goalId: "anomaly-arbitration-reference-v1",
    mode: "anomaly-arbitration",
    commit: "6b3ed8a13e962c95c5654e780d4b64f4f71ffc2c",
    tree: "46859cec1a7a3639bc898fe724da89f8058d3fa1",
    manifest:
      "content-manifests/anomaly-arbitration-v1/content-manifest.json",
    evidence:
      "evidence/anomaly-arbitration-reference-v1/release/release-evidence.json",
    status: "docs/goals/13-anomaly-arbitration-reference-data-status.md",
    expectedResult: "Passed",
    expectedRecords: 392,
    normalizedDigestField: "pack_index_sha256",
    bundleDigestField: "sora_bundle_sha256",
  },
];

const modeReports = modes.map(verifyMode);
const releaseEvidence = Object.fromEntries(
  modes.map((mode) => [mode.goal, json(mode.evidence)]),
);
const pairwiseReports = verifyPublishedReconciliations(releaseEvidence);
const anomalyFinal = reconcileFinalAnomalyPeers();

const coveredPairs = new Set(pairwiseReports.flatMap(({ goal, peers }) =>
  peers.map((peer) => pairKey(goal, peer))));
for (const peer of anomalyFinal.peer_goals.filter((goal) => goal !== "G07"))
  coveredPairs.add(pairKey("G13", peer));
const expectedPairs = [];
for (let left = 0; left < modes.length; left += 1) {
  for (let right = left + 1; right < modes.length; right += 1)
    expectedPairs.push(pairKey(modes[left].goal, modes[right].goal));
}
assert(
  expectedPairs.every((pair) => coveredPairs.has(pair))
    && coveredPairs.size === expectedPairs.length,
  "pairwise reconciliation coverage is incomplete",
);

const report = {
  schema_revision: "starclock.merged-reference-integration-audit.v1",
  observed_on: "2026-07-30",
  result: "Pass",
  scope:
    "Final merged Goal 08-13 Candidate reference snapshots and their "
      + "triangular pairwise reconciliation chain.",
  mode_count: modes.length,
  manifest_record_count: modeReports.reduce(
    (sum, mode) => sum + mode.manifest_record_count,
    0,
  ),
  final_snapshots_unchanged: true,
  immutable_release_snapshots_registered: true,
  runtime_loading_enabled_modes: 0,
  pairwise_mode_pair_count: expectedPairs.length,
  pairwise_mode_pairs: expectedPairs.sort(),
  conflict_count: 0,
  merge_coordination_required: false,
  source_cache_eol_policy:
    "Goal 01 frozen source receipts use canonical UTF-8 CRLF hashes; "
      + "checkout-only LF/CRLF differences are not content drift.",
  modes: modeReports,
  published_reconciliations: pairwiseReports,
  anomaly_arbitration_final_peer_reconciliation: anomalyFinal,
  conclusions: [
    "No mode-owned manifest or release evidence changed while merging.",
    "All 15 Goal 08-13 mode pairs are covered by the ordered reconciliation chain.",
    "No reconciliation reports a factual evidence conflict.",
    "All six packages remain Candidate reference data with runtime loading disabled.",
    "Historical release evidence remains unchanged; this report records post-merge compatibility.",
  ],
};
assert(report.manifest_record_count === 46_110,
  "merged manifest denominator drift");

const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  assert(fs.existsSync(output), "merged reference integration audit is missing");
  assert(fs.readFileSync(output, "utf8") === encoded,
    "merged reference integration audit drift");
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, encoded);
}

console.log(
  "Merged reference integration verified: 6 Candidate modes, "
    + "46,110 manifest obligations, 15/15 mode pairs, 0 conflicts and "
    + "0 runtime-enabled modes.",
);

function verifyMode(mode) {
  assert(gitObjectExists(mode.commit), `${mode.goal}: completion commit missing`);
  assert(isAncestor(mode.commit, "HEAD"),
    `${mode.goal}: completion commit is not merged`);
  assert(git(["show", "-s", "--format=%T", mode.commit]).trim() === mode.tree,
    `${mode.goal}: completion tree drift`);
  const manifestSnapshot = gitBuffer(mode.commit, mode.manifest);
  const evidenceSnapshot = gitBuffer(mode.commit, mode.evidence);
  assert(
    Buffer.compare(manifestSnapshot, fs.readFileSync(path.join(root, mode.manifest)))
      === 0,
    `${mode.goal}: merged manifest differs from completion snapshot`,
  );
  assert(
    Buffer.compare(evidenceSnapshot, fs.readFileSync(path.join(root, mode.evidence)))
      === 0,
    `${mode.goal}: merged release evidence differs from completion snapshot`,
  );

  const manifest = JSON.parse(manifestSnapshot);
  const evidence = JSON.parse(evidenceSnapshot);
  const records = flattenCategories(manifest);
  assert(records.length === mode.expectedRecords,
    `${mode.goal}: manifest record denominator drift`);
  assert(evidence.goal_id === mode.goalId && evidence.result === mode.expectedResult,
    `${mode.goal}: release identity/result drift`);
  assert(
    evidence.digests.content_manifest_sha256 === sha256(manifestSnapshot),
    `${mode.goal}: release evidence does not bind the completion manifest`,
  );
  verifyRuntimeBoundary(mode.goal, evidence);

  const snapshot = snapshotPolicy.goals.find(
    ({ goal_id: goalId }) => goalId === mode.goalId,
  );
  assert(snapshot
    && snapshot.completion_commit === mode.commit
    && snapshot.completion_tree === mode.tree
    && snapshot.status_path === mode.status
    && snapshot.release_evidence_path === mode.evidence,
  `${mode.goal}: immutable release snapshot registration differs`);
  assert(contentIndex.includes(`\`${mode.mode}-v1/\``),
    `${mode.goal}: content root index entry is missing`);
  assert(documentationIndex.includes(path.basename(mode.status)),
    `${mode.goal}: documentation index ledger link is missing`);

  return {
    goal: mode.goal,
    goal_id: mode.goalId,
    mode: mode.mode,
    completion_commit: mode.commit,
    completion_tree: mode.tree,
    manifest_path: mode.manifest,
    manifest_sha256: sha256(manifestSnapshot),
    manifest_record_count: records.length,
    release_evidence_path: mode.evidence,
    release_evidence_sha256: sha256(evidenceSnapshot),
    release_result: evidence.result,
    normalized_pack_sha256:
      evidence.digests[mode.normalizedDigestField],
    candidate_bundle_sha256: evidence.digests[mode.bundleDigestField],
    current_tree_matches_completion_snapshot: true,
    runtime_loading: "Disabled",
  };
}

function verifyPublishedReconciliations(evidence) {
  const reports = [
    {
      goal: "G09",
      peers: ["G08"],
      receipt_count: evidence.G09.reconciliation.receipts,
      exact_match_count: evidence.G09.reconciliation.receipts,
      conflict_count: evidence.G09.reconciliation.conflicts,
    },
    {
      goal: "G10",
      peers: ["G08", "G09"],
      receipt_count: evidence.G10.reconciliation.receipts,
      exact_match_count: evidence.G10.reconciliation.matched_shared,
      conflict_count: evidence.G10.reconciliation.conflicts,
    },
    {
      goal: "G11",
      peers: ["G08", "G09", "G10"],
      receipt_count: evidence.G11.reconciliation.receipts,
      exact_match_count: evidence.G11.reconciliation.receipts,
      non_join_digest_representation_count:
        evidence.G11.reconciliation.non_join_digest_representations,
      conflict_count: evidence.G11.reconciliation.conflicts,
    },
    {
      goal: "G12",
      peers: ["G08", "G09", "G10", "G11"],
      receipt_count: evidence.G12.reconciliation.receipts,
      exact_match_count: evidence.G12.reconciliation.exact_overlaps,
      conflict_count: evidence.G12.reconciliation.conflicts,
    },
  ];
  assert(
    reports.every(({ conflict_count: conflicts }) => conflicts === 0),
    "published reconciliation conflict detected",
  );

  const swarmCheckpoint = evidence.G09.reconciliation.goal08_commit;
  const goldManifest = modes.find(({ goal }) => goal === "G08").manifest;
  assert(
    Buffer.compare(
      gitBuffer(swarmCheckpoint, goldManifest),
      gitBuffer(modes[0].commit, goldManifest),
    ) === 0,
    "G09 reconciliation used a pre-final Goal 08 manifest that later changed",
  );
  return reports;
}

function reconcileFinalAnomalyPeers() {
  const anomalyMode = modes.find(({ goal }) => goal === "G13");
  const manifest = JSON.parse(gitBuffer(
    anomalyMode.commit,
    anomalyMode.manifest,
  ));
  const sharedRows = Object.entries(manifest.categories).flatMap(
    ([category, value]) => (value.records ?? [])
      .filter((row) => row.ownership === "Shared")
      .map((row) => ({
        manifest_record_id: `${category}:${row.id}`,
        source_path: row.source_path,
        row_locator: row.row_locator,
        evidence_sha256: row.evidence_sha256,
      })),
  );
  assert(sharedRows.length === 316, "G13 shared-row denominator drift");

  const peerDefinitions = [
    {
      goal: "G07",
      commit: "c3acefe237cef1e277e28a017e60ecdbe119f5d0",
      manifest:
        "content-manifests/standard-universe-v1/content-manifest.json",
    },
    ...modes.filter(({ goal }) => goal !== "G13"),
  ];
  const peerRecords = [];
  const peers = [];
  for (const peer of peerDefinitions) {
    assert(gitObjectExists(peer.commit) && isAncestor(peer.commit, "HEAD"),
      `${peer.goal}: final peer snapshot is not merged`);
    const records = flattenCategories(
      JSON.parse(gitBuffer(peer.commit, peer.manifest)),
    );
    peerRecords.push(...records.map((row) => ({ ...row, goal: peer.goal })));
    peers.push({
      goal: peer.goal,
      completion_commit: peer.commit,
      manifest_path: peer.manifest,
      manifest_record_count: records.length,
    });
  }

  const classifications = sharedRows.map((row) => {
    const samePath = peerRecords.filter(
      (peer) => peer.source_path === row.source_path,
    );
    const sameLocator = samePath.filter(
      (peer) => peer.row_locator === row.row_locator,
    );
    const exact = sameLocator.filter(
      (peer) => peer.evidence_sha256 === row.evidence_sha256,
    );
    const conflicts = sameLocator.filter(
      (peer) => peer.evidence_sha256 !== row.evidence_sha256,
    );
    const containerOnly = samePath.filter(
      (peer) => peer.row_locator !== row.row_locator,
    );
    assert(conflicts.length === 0,
      `${row.manifest_record_id}: final peer evidence conflict`);
    return {
      manifest_record_id: row.manifest_record_id,
      result: exact.length > 0
        ? "ExactFinalPeerMatch"
        : containerOnly.length > 0
          ? "FinalPeerContainerOnly"
          : "AbsentFromFinalPeerManifests",
      exact_peer_records: exact.map(peerIdentity).sort(compareCanonical),
      container_peer_records:
        containerOnly.map(peerIdentity).sort(compareCanonical),
    };
  });
  const counts = Object.fromEntries(
    [
      "ExactFinalPeerMatch",
      "FinalPeerContainerOnly",
      "AbsentFromFinalPeerManifests",
    ].map((result) => [
      result,
      classifications.filter((row) => row.result === result).length,
    ]),
  );
  assert(
    counts.ExactFinalPeerMatch === 0
      && counts.FinalPeerContainerOnly === 5
      && counts.AbsentFromFinalPeerManifests === 311,
    "G13 final peer classification drift",
  );
  return {
    shared_record_count: sharedRows.length,
    peer_goals: peers.map(({ goal }) => goal),
    peers,
    exact_match_count: counts.ExactFinalPeerMatch,
    container_only_count: counts.FinalPeerContainerOnly,
    absent_count: counts.AbsentFromFinalPeerManifests,
    conflict_count: 0,
    classification_sha256: sha256(
      Buffer.from(canonical(classifications), "utf8"),
    ),
    container_observations: classifications
      .filter(({ result }) => result === "FinalPeerContainerOnly")
      .map((row) => ({
        manifest_record_id: row.manifest_record_id,
        peer_goals: [...new Set(
          row.container_peer_records.map(({ goal }) => goal),
        )].sort(),
        peer_record_count: row.container_peer_records.length,
      })),
    historical_goal13_release_evidence_preserved: true,
  };
}

function verifyRuntimeBoundary(goal, evidence) {
  if (goal === "G13") {
    assert(
      evidence.release_state === "CandidateReferenceData"
        && evidence.runtime_profile_state === "Unreleased"
        && evidence.counts.runtime_executable_rows === 0,
      `${goal}: runtime boundary drift`,
    );
    return;
  }
  const boundary = evidence.runtime_boundary;
  assert(boundary && boundary.runtime_lowering === false,
    `${goal}: runtime lowering is not disabled`);
  assert(
    boundary.runtime_loading === false
      || boundary.runtime_loading === "ForbiddenReferenceOnly",
    `${goal}: runtime loading is not disabled`,
  );
}

function flattenCategories(document) {
  return Object.entries(document.categories ?? {}).flatMap(
    ([category, value]) => (value.records ?? []).map((row) => ({
      category,
      id: String(row.id),
      evidence_sha256: row.evidence_sha256,
      ...normalizeSource(row),
    })),
  );
}

function normalizeSource(row) {
  if (row.source_path) {
    return {
      source_path: row.source_path,
      row_locator: row.row_locator ?? "",
    };
  }
  const source = row.source ?? "";
  const hash = source.indexOf("#");
  return hash === -1
    ? { source_path: source, row_locator: "" }
    : {
      source_path: source.slice(0, hash),
      row_locator: source.slice(hash + 1),
    };
}

function peerIdentity(peer) {
  return {
    goal: peer.goal,
    category: peer.category,
    id: peer.id,
  };
}

function pairKey(left, right) {
  return [left, right].sort().join("-");
}

function compareCanonical(left, right) {
  return canonical(left).localeCompare(canonical(right));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map(
      (key) => `${JSON.stringify(key)}:${canonical(value[key])}`,
    ).join(",")}}`;
  }
  return JSON.stringify(value);
}

function git(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  });
}

function gitBuffer(commit, relative) {
  return execFileSync("git", ["show", `${commit}:${relative}`], {
    cwd: root,
    maxBuffer: 256 * 1024 * 1024,
  });
}

function gitObjectExists(commit) {
  return spawnSync(
    "git",
    ["cat-file", "-e", `${commit}^{commit}`],
    { cwd: root },
  ).status === 0;
}

function isAncestor(commit, descendant) {
  return spawnSync(
    "git",
    ["merge-base", "--is-ancestor", commit, descendant],
    { cwd: root },
  ).status === 0;
}

function json(relative) {
  return JSON.parse(text(relative));
}

function text(relative) {
  return fs.readFileSync(path.join(root, relative), "utf8");
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
