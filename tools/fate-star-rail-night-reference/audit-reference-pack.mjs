#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const manifest = json("content-manifests/fate-star-rail-night-v1/content-manifest.json");
const coverage = json("content-reference/fate-star-rail-night-v1/coverage.json");
const pack = json("content-reference/fate-star-rail-night-v1/pack-index.json");
const sources = json("content-reference/fate-star-rail-night-v1/sources.json");
const pools = json("content-reference/fate-star-rail-night-v1/pool-audits.json");
const normalizedFiles = pack.files
  .filter(({ records }) => records > 0)
  .map(({ path }) => json(`content-reference/fate-star-rail-night-v1/${path}`))
  .filter((payload) => Array.isArray(payload.records));
const records = normalizedFiles.flatMap(({ records: values }) => values);

assert(manifest.obligations.length === 1_904, "manifest denominator drift");
assert(new Set(manifest.obligations.map(({ obligation_id }) => obligation_id)).size === 1_904, "duplicate manifest obligation");
assert(coverage.rows.length === 1_904 && coverage.counts.required === 1_904 && coverage.counts.accounted === 1_904, "coverage denominator drift");
assert(new Set(coverage.rows.map(({ obligation_id }) => obligation_id)).size === 1_904, "duplicate coverage row");
const obligations = new Set(manifest.obligations.map(({ obligation_id }) => obligation_id));
assert(coverage.rows.every(({ obligation_id }) => obligations.has(obligation_id)), "coverage outside manifest");
assert(coverage.counts.data_ready === 1_491 && coverage.counts.evidence_only === 413 && coverage.counts.policy_bound === 13 && coverage.counts.unresolved === 0, "coverage disposition drift");
assert(pack.counts.normalized_records === 2_018 && records.length === 2_018, "normalized record denominator drift");
assert(new Set(records.map(({ stable_id }) => stable_id)).size === 2_018, "duplicate normalized stable ID");
for (const row of records) {
  assert(typeof row.name_zh === "string" && row.name_zh.length > 0, `${row.stable_id}: missing Chinese name`);
  assert(typeof row.name_en === "string" && row.name_en.length > 0, `${row.stable_id}: missing English name`);
  assert(typeof row.summary_zh === "string" && row.summary_zh.length > 0, `${row.stable_id}: missing Chinese summary`);
  assert(typeof row.summary_en === "string" && row.summary_en.length > 0, `${row.stable_id}: missing English summary`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0, `${row.stable_id}: missing provenance`);
  assert(row.source_refs.every(({ path, locator, sha256 }) => path && locator && /^[0-9a-f]{64}$/u.test(sha256)), `${row.stable_id}: invalid provenance receipt`);
  if (row.disposition === "EvidenceOnly") assert(row.enabled === false, `${row.stable_id}: evidence-only row enabled`);
}
assert(pools.records.length === 6 && pools.records.every((row) => row.mechanic_payload.required === "0" && row.mechanic_payload.accounted === "0" && row.mechanic_payload.data_ready === "0"), "exact-zero pool proof drift");
assert(sources.sources.length === 1_914 && new Set(sources.sources.map(({ source_id }) => source_id)).size === 1_914, "source receipt denominator drift");
assert(manifest.obligations.every(({ source_path }) => !source_path.startsWith("Config/Activity/RtBattle/") && !source_path.startsWith("Config/Gameplays/GridFight/")), "named mode exclusion leaked into manifest");
let runtimeMatches = "";
try {
  runtimeMatches = execFileSync("git", ["grep", "-n", "fate-star-rail-night", "--", "crates"], { cwd: root, encoding: "utf8" });
} catch (error) {
  if (error.status !== 1) throw error;
  runtimeMatches = error.stdout ?? "";
}
assert(runtimeMatches.trim() === "", "Fate reference data leaked into runtime crates");
console.log("Fate reference audit passed: 1,904 exact obligations, 2,018 unique normalized rows, 1,914 source receipts, six exact-zero pools and zero runtime/exclusion leaks.");

function json(relative) { return JSON.parse(readFileSync(resolve(root, relative), "utf8")); }
function assert(condition, message) { if (!condition) throw new Error(message); }
