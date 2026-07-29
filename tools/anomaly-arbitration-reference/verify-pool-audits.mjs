#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/pool-audits.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));
const families = [
  "blessings",
  "curios",
  "occurrences",
  "gameplay_services",
  "currencies",
  "random_content_pools",
];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    root,
    "tools/anomaly-arbitration-reference/import-pool-audits.mjs",
  ),
  "--check",
], { stdio: "inherit" });
const encoded = await readFile(output);
const document = JSON.parse(encoded);
assert(
  document.schema_revision
    === "starclock.anomaly-arbitration-normalized-file.v1"
  && document.goal_id === "anomaly-arbitration-reference-v1"
  && document.profile === "anomaly-arbitration-v1"
  && document.file === "pool-audits.json"
  && document.record_kind === "PoolAudit",
  "pool audit envelope drift",
);
assert(document.records.length === 6, "pool audit count drift");
for (const [index, record] of document.records.entries()) {
  const family = families[index];
  for (const field of schema.common_envelope.required_fields)
    assert(record[field] !== undefined, `${record.id} lacks ${field}`);
  assert(record.kind === "PoolAudit"
    && record.name_en && record.name_zh_cn
    && record.summary_en && record.summary_zh_cn
    && record.ownership === "AnomalyArbitration"
    && record.coverage_state === "DataReady"
    && record.evidence_quality === "ExactStructured"
    && record.mechanism_quality === "ExactRelationship"
    && record.runtime_executable === false,
  `${record.id} boundary drift`);
  assert(record.pool_family === family
    && record.active_member_count === 0
    && record.manifest_record_ids.length === 0
    && record.account_reward_locators_are_members === false,
  `${record.id} zero-family drift`);
  assert(manifest.categories[family].count === 0
    && manifest.categories[family].records.length === 0,
  `${family} manifest is no longer empty`);
  const proof = manifest.zero_pool_proofs[family];
  assert(record.selector_closure_sha256 === proof.selector_closure_sha256
    && record.replacement_condition === proof.replacement_condition
    && record.source_refs[0].sha256 === proof.selector_closure_sha256
    && /^[0-9a-f]{64}$/u.test(record.selector_closure_sha256),
  `${family} proof receipt drift`);
  assert(record.selector_scope.length === 8
    && record.closure_rule.includes("No explicit active selector"),
  `${family} selector closure drift`);
}
assert(new Set(document.records.map(
  ({ selector_closure_sha256: value }) => value,
)).size === 6, "zero families share a non-independent proof");

console.log(
  "Anomaly Arbitration pool audits verified: "
    + `pool-audits.json=${createHash("sha256").update(encoded).digest("hex")}`,
);
