#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const manifest = json("content-manifests/fate-star-rail-night-v1/content-manifest.json");
const lock = json("content-manifests/fate-star-rail-night-v1/peer-reconciliation-lock.json");
const output = json("content-reference/fate-star-rail-night-v1/reconciliation.json");
const localTriples = new Map(manifest.obligations.map((row) => [
  tripleKey(row.source_path, row.locator, row.source_sha256),
  row.obligation_id,
]));
const localLocators = new Map(manifest.obligations.map((row) => [
  locatorKey(row.source_path, row.locator),
  { obligation_id: row.obligation_id, sha256: row.source_sha256 },
]));
for (const peer of lock.peers) {
  const bytes = execFileSync("git", ["show", `${peer.commit}:${peer.peer_manifest}`], { cwd: root });
  assert(sha256(bytes) === peer.peer_manifest_sha256, `${peer.peer_goal}: manifest digest drift`);
  const document = JSON.parse(bytes.toString("utf8"));
  const triples = collectTriples(document);
  const locators = collectLocators(document);
  const matches = [...triples].filter((key) => localTriples.has(key)).sort();
  const conflicts = [...locators].flatMap(([key, digests]) => {
    const local = localLocators.get(key);
    if (!local || digests.has(local.sha256)) return [];
    return [{ local_obligation_id: local.obligation_id, locator: key, local_sha256: local.sha256, peer_sha256: [...digests].sort() }];
  }).sort((left, right) => left.locator.localeCompare(right.locator));
  assert(matches.length === peer.match_count, `${peer.peer_goal}: exact match count drift`);
  assert(conflicts.length === peer.conflict_count, `${peer.peer_goal}: conflict count drift`);
  const receipt = output.receipts.find(({ peer_goal: goal }) => goal === peer.peer_goal);
  assert(receipt && receipt.peer_manifest_sha256 === peer.peer_manifest_sha256 && receipt.match_count === matches.length && receipt.conflict_count === conflicts.length && receipt.decision === peer.decision, `${peer.peer_goal}: output receipt drift`);
}
console.log(`Verified ${lock.peers.length} immutable concurrent peer manifests: exact matches=0, same-locator digest conflicts=0.`);

function collectTriples(value, output = new Set()) {
  visit(value, (path, locator, digest) => output.add(tripleKey(path, locator, digest)));
  return output;
}
function collectLocators(value, output = new Map()) {
  visit(value, (path, locator, digest) => {
    const key = locatorKey(path, locator);
    const digests = output.get(key) ?? new Set();
    digests.add(digest);
    output.set(key, digests);
  });
  return output;
}
function visit(value, callback) {
  if (Array.isArray(value)) for (const entry of value) visit(entry, callback);
  else if (value && typeof value === "object") {
    const path = value.source_path ?? value.path ?? value.relative_path;
    const locator = value.locator ?? value.source_locator ?? value.row_locator;
    const digest = value.source_sha256 ?? value.sha256 ?? value.evidence_digest ?? value.evidence_sha256;
    if (typeof path === "string" && typeof locator === "string" && typeof digest === "string" && /^[0-9a-f]{64}$/u.test(digest)) callback(path, locator, digest);
    for (const child of Object.values(value)) visit(child, callback);
  }
}
function tripleKey(path, locator, digest) { return `${path}\u001f${locator}\u001f${digest}`; }
function locatorKey(path, locator) { return `${path}\u001f${locator}`; }
function sha256(value) { return createHash("sha256").update(value).digest("hex"); }
function json(relative) { return JSON.parse(readFileSync(resolve(root, relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
