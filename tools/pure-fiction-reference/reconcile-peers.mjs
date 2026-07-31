#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const peerArguments = process.argv.filter((arg) => arg.startsWith("--peer="))
  .map((arg) => arg.slice(7));
if (peerArguments.length !== 3)
  throw new Error("provide three --peer=<name>@<root>: memory-of-chaos, apocalyptic-shadow, fate-star-rail-night");
const paths = {
  "memory-of-chaos": "content-manifests/memory-of-chaos-v1/content-manifest.json",
  "apocalyptic-shadow": "content-manifests/apocalyptic-shadow-v1/content-manifest.json",
  "fate-star-rail-night": "content-manifests/fate-star-rail-night-v1/content-manifest.json",
};
function flattenManifest(document) {
  if (Array.isArray(document.obligations)) return document.obligations;
  const records = [];
  for (const value of Object.values(document.categories ?? {})) {
    if (Array.isArray(value)) records.push(...value);
    else if (Array.isArray(value?.records)) records.push(...value.records);
  }
  return records;
}
function normalize(record) {
  return { id: record.id ?? record.obligation_id, source_path: record.source_path,
    locator: record.source_locator ?? record.row_locator ?? record.locator,
    digest: record.evidence_digest ?? record.evidence_sha256 ?? record.source_sha256 };
}
const peers = [];
for (const argument of peerArguments) {
  const separator = argument.indexOf("@");
  const name = argument.slice(0, separator);
  const peerRoot = argument.slice(separator + 1);
  if (!paths[name] || !peerRoot) throw new Error(`invalid peer ${argument}`);
  const bytes = await readFile(path.join(peerRoot, paths[name]));
  const document = JSON.parse(bytes);
  const records = flattenManifest(document).map(normalize)
    .filter((row) => row.source_path && row.locator && row.digest);
  peers.push({ name, root: peerRoot,
    commit: execFileSync("git", ["-C", peerRoot, "rev-parse", "HEAD"],
      { encoding: "utf8" }).trim(),
    manifest_path: paths[name], manifest_sha256: createHash("sha256").update(bytes).digest("hex"),
    records });
}
const ours = JSON.parse(await readFile(path.join(root,
  "content-reference/pure-fiction-v1/reconciliation.json"))).records;
const overlaps = [];
const conflicts = [];
for (const row of ours) for (const peer of peers) {
  const samePath = peer.records.filter((candidate) => candidate.source_path === row.source_path);
  for (const candidate of samePath) {
    const exactLocator = candidate.locator === row.source_locator;
    const exactDigest = candidate.digest === row.evidence_digest;
    if (exactLocator && !exactDigest) conflicts.push({ pure_fiction_id: row.id,
      peer: peer.name, peer_id: candidate.id, source_path: row.source_path,
      locator: row.source_locator, pure_fiction_digest: row.evidence_digest,
      peer_digest: candidate.digest });
    if ((exactLocator && exactDigest) || (!exactLocator && exactDigest))
      overlaps.push({ pure_fiction_id: row.id, peer: peer.name, peer_id: candidate.id,
        source_path: row.source_path, pure_fiction_locator: row.source_locator,
        peer_locator: candidate.locator, digest: row.evidence_digest,
        outcome: exactLocator ? "ExactTripleMatch" : "PathDigestMatchLocatorVariance" });
  }
}
if (conflicts.length) throw new Error(`${conflicts.length} peer reconciliation conflicts: ${JSON.stringify(conflicts[0])}`);
const uniqueOverlaps = [...new Map(overlaps.map((row) =>
  [`${row.pure_fiction_id}\0${row.peer}\0${row.peer_id}`, row])).values()]
  .sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b)));
const report = { schema_revision: "starclock.pure-fiction-peer-reconciliation.v1",
  goal_id: "pure-fiction-reference-v1", batch: "G15-P4-B3",
  reconciliation_key: ["source_path", "stable_row_locator", "evidence_digest"],
  pure_fiction_shared_receipts: ours.length,
  peers: peers.map(({ name, commit, manifest_path, manifest_sha256, records }) =>
    ({ name, commit, manifest_path, manifest_sha256, comparable_records: records.length })),
  overlap_count: uniqueOverlaps.length,
  exact_triple_match_count: uniqueOverlaps.filter((row) =>
    row.outcome === "ExactTripleMatch").length,
  locator_variance_match_count: uniqueOverlaps.filter((row) =>
    row.outcome === "PathDigestMatchLocatorVariance").length,
  conflict_count: 0, peer_artifact_mutation_count: 0,
  overlaps: uniqueOverlaps, result: "Passed" };
const output = path.join(root, "evidence/pure-fiction-v1/peer-reconciliation.json");
await mkdir(path.dirname(output), { recursive: true });
await writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
console.log(`Pure Fiction peer reconciliation: ${ours.length} shared receipts, `
  + `${uniqueOverlaps.length} peer overlaps, zero conflicts/mutations.`);
