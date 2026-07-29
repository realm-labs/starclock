#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "evidence/anomaly-arbitration-reference-v1/peer-reconciliation.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
)));

const peers = [
  {
    goal: "G07",
    mode: "standard-universe",
    commit: "3e03a9ac9a1723584a9eb1430fd01761d4f28cbf",
    branch: "origin/master",
    manifest:
      "content-manifests/standard-universe-v1/content-manifest.json",
    execution_state: "Complete",
    publication_state: "RemoteMerged",
    observed_worktree_dirty_entries: 0,
  },
  {
    goal: "G08",
    mode: "gold-and-gears",
    commit: "2688624c34a564d87076cadb405c8da506efd373",
    branch: "codex/goal08-gold-gears-reference",
    manifest: "content-manifests/gold-and-gears-v1/content-manifest.json",
    execution_state: "Complete",
    publication_state: "CommittedLocalBranchNoRemoteRef",
    observed_worktree_dirty_entries: 0,
  },
  {
    goal: "G09",
    mode: "swarm-disaster",
    commit: "d258c94dfb6426017fee9216f6ae2bc0f6e257d0",
    branch: "origin/codex/goal09-swarm-disaster-reference",
    manifest: "content-manifests/swarm-disaster-v1/content-manifest.json",
    execution_state: "Complete",
    publication_state: "RemoteBacked",
    observed_worktree_dirty_entries: 0,
  },
  {
    goal: "G10",
    mode: "unknowable-domain",
    commit: "6d8d3b1e834bbf29d1d5787f4fe12d9b75e66b29",
    branch: "origin/codex/goal10-unknowable-domain-reference",
    manifest: "content-manifests/unknowable-domain-v1/content-manifest.json",
    execution_state: "Complete",
    publication_state: "RemoteBacked",
    observed_worktree_dirty_entries: 0,
  },
  {
    goal: "G11",
    mode: "divergent-universe",
    commit: "d3928d69a5e6b2622b8bea41f370f2bf328ff072",
    branch: "origin/codex/goal11-divergent-universe-reference",
    manifest: "content-manifests/divergent-universe-v1/content-manifest.json",
    execution_state: "InProgressAtP4B2",
    publication_state: "RemoteBackedCommittedPrefix",
    observed_worktree_dirty_entries: 108,
  },
  {
    goal: "G12",
    mode: "currency-wars",
    commit: "798ffac6fa46917cee45b8ff187fbd8e87ad4a78",
    branch: "origin/codex/goal12-currency-wars-reference",
    manifest: "content-manifests/currency-wars-v1/content-manifest.json",
    execution_state: "InProgressAtP2B4",
    publication_state: "RemoteBackedCommittedPrefix",
    observed_worktree_dirty_entries: 2,
  },
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function git(args) {
  return execFileSync("git", args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
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
  if (hash === -1)
    return { source_path: source, row_locator: "" };
  return {
    source_path: source.slice(0, hash),
    row_locator: source.slice(hash + 1),
  };
}

function flattenCategories(document) {
  return Object.entries(document.categories ?? {}).flatMap(
    ([category, value]) => (value.records ?? []).map((row) => ({
      category,
      id: row.id,
      evidence_sha256: row.evidence_sha256,
      ...normalizeSource(row),
    })),
  );
}

const sharedRows = Object.entries(manifest.categories).flatMap(
  ([category, value]) => (value.records ?? [])
    .filter((row) => row.ownership === "Shared")
    .map((row) => ({
      manifest_record_id: `${category}:${row.id}`,
      source_path: row.source_path,
      row_locator: row.row_locator,
      evidence_sha256: row.evidence_sha256,
      evidence_summary: row.selector,
    })),
);
assert(sharedRows.length === 316, "shared-row denominator drift");

const peerReports = [];
const peerRecords = [];
for (const peer of peers) {
  assert(git(["cat-file", "-t", peer.commit]).trim() === "commit",
    `${peer.goal} commit is unavailable`);
  const document = JSON.parse(git([
    "show",
    `${peer.commit}:${peer.manifest}`,
  ]));
  const records = flattenCategories(document);
  peerRecords.push(...records.map((row) => ({ ...row, goal: peer.goal })));
  peerReports.push({
    ...peer,
    manifest_schema: document.schema_revision ?? document.schema,
    committed_manifest_record_count: records.length,
    authority:
      "committed blob only; uncommitted worktree state was not inspected",
  });
}

const reconciliations = sharedRows.map((row) => {
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
    `${row.manifest_record_id}: peer evidence digest conflict`);
  return {
    ...row,
    result: exact.length > 0
      ? "ExactCommittedPeerMatch"
      : containerOnly.length > 0
        ? "CommittedPeerContainerOnly"
        : "AbsentFromCommittedPeerManifest",
    exact_peer_records: exact.map((peer) => ({
      goal: peer.goal,
      category: peer.category,
      id: peer.id,
    })),
    peer_container_records: containerOnly.map((peer) => ({
      goal: peer.goal,
      category: peer.category,
      id: peer.id,
      source_path: peer.source_path,
      row_locator: peer.row_locator || "(container-level obligation)",
      evidence_sha256: peer.evidence_sha256,
    })),
  };
});

const resultCounts = Object.fromEntries(
  [
    "ExactCommittedPeerMatch",
    "CommittedPeerContainerOnly",
    "AbsentFromCommittedPeerManifest",
  ].map((result) => [
    result,
    reconciliations.filter((row) => row.result === result).length,
  ]),
);
assert(resultCounts.ExactCommittedPeerMatch === 0
  && resultCounts.CommittedPeerContainerOnly === 5
  && resultCounts.AbsentFromCommittedPeerManifest === 311,
"peer reconciliation classification drift");

const report = {
  schema_revision:
    "starclock.anomaly-arbitration-peer-reconciliation.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  observed_at: "2026-07-30",
  game_version: "4.4",
  source_revision: manifest.snapshot.source_revision,
  authority:
    "Goal 07-12 committed manifest blobs at explicit commits; local "
      + "uncommitted worktree rows are excluded",
  comparison_key:
    "source_path + row_locator + evidence_sha256; matching source "
      + "containers without a peer row locator are informational only",
  shared_record_count: reconciliations.length,
  peer_goal_count: peerReports.length,
  exact_match_count: resultCounts.ExactCommittedPeerMatch,
  container_only_count: resultCounts.CommittedPeerContainerOnly,
  absent_count: resultCounts.AbsentFromCommittedPeerManifest,
  conflict_count: 0,
  merge_coordination_required: false,
  peers: peerReports,
  reconciliations,
};
const encoded = `${JSON.stringify(report, null, 2)}\n`;
if (check) {
  const current = await readFile(output, "utf8");
  assert(current === encoded, "peer reconciliation report drift");
} else {
  await writeFile(output, encoded);
}
console.log(
  "Anomaly Arbitration peer reconciliation passed: "
    + "316 shared rows, 6 committed Goal 07-12 manifests, "
    + "5 container-only observations, 311 absent and 0 conflicts.",
);
