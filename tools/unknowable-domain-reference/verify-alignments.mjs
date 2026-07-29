#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-alignments.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const alignments = json(
  "content-reference/unknowable-domain-v1/alignments.json",
);
assert(alignments.length === 4, "Alignment denominator drift");
assert(alignments.map(({ source_id: sourceId }) => sourceId).sort().join(",")
  === "Break,Dot,Follow,Ultimate",
"Alignment selector boundary drift");
assert(unique(alignments.map(({ id }) => id)),
  "Alignment stable IDs are not unique");

const allScepters = new Set();
for (const row of alignments) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.kind === "ExtrapolationAlignment"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length === 2
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && source.mechanism_quality === "DirectStructured"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
  assert(row.scepter_candidate_ids.length === 6,
    `${row.id} Scepter candidate count drift`);
  assert(row.component_candidate_ids.length === 0
    && row.component_pool_resolution === "Unspecified",
  `${row.id} inferred an unavailable Component pool`);
  assert(row.selection_cardinality === "Unspecified",
    `${row.id} overclaims selection cardinality`);
  assert(row.rule_contribution_ids.length === 0
    && row.contribution_resolution === "DeferredToScepterAndComponentRules",
  `${row.id} overclaims a direct rule contribution`);
  for (const scepterId of row.scepter_candidate_ids)
    assert(!allScepters.has(scepterId) && allScepters.add(scepterId),
      `${scepterId} belongs to multiple Alignment pools`);
}
assert(allScepters.size === 24, "Alignment Scepter pool closure drift");
const defaultAreas = alignments.flatMap(({ default_area_ids: ids }) => ids);
assert(defaultAreas.length === 13 && unique(defaultAreas),
  "Alignment default-area binding drift");
assert(alignments.find(({ source_id: id }) => id === "Ultimate").unlock_id === ""
  && alignments.filter(({ source_id: id }) => id !== "Ultimate")
    .every(({ unlock_id: unlockId }) => unlockId !== ""),
"Alignment unlock boundary drift");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
const frozen = manifest.categories.alignments.records
  .map(({ id }) => id).sort();
assert(JSON.stringify(alignments.map(({ source_id: id }) => id).sort())
  === JSON.stringify(frozen),
"Alignment manifest exact-once drift");
const areas = json("content-reference/unknowable-domain-v1/areas.json");
const areaIds = new Set(areas.map(({ id }) => id));
assert(defaultAreas.every((id) => areaIds.has(id)),
  "Alignment references an unknown area");

console.log(
  "Unknowable Domain Alignments verified (4 selectors; 24 exact Scepter " +
  "bindings; 13 default areas; Component pools remain fail-closed).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
