#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-transformations.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const synthesis = json(
  "content-reference/unknowable-domain-v1/synthesis-rules.json",
);
const upgrades = json(
  "content-reference/unknowable-domain-v1/upgrade-rules.json",
);
const reforges = json(
  "content-reference/unknowable-domain-v1/reforge-rules.json",
);
const rows = [...synthesis, ...upgrades, ...reforges];
assert(synthesis.length === 1, "synthesis rule denominator drift");
assert(upgrades.length === 2, "upgrade rule denominator drift");
assert(reforges.length === 1, "reforge rule denominator drift");
assert(unique(rows.map(({ id }) => id)), "duplicate transformation stable ID");
for (const row of rows) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ProjectPolicy"
    && row.policy_id === "component-transformation-policy-v1",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length === 2
    && row.source_refs.some(({ evidence_quality: quality }) =>
      quality === "ExactStructured")
    && row.source_refs.some(({ evidence_quality: quality, note,
      replacement_condition: replacement }) =>
      quality === "ProjectPolicy" && note?.length > 0 && replacement?.length > 0),
  `${row.id} does not separate fact from policy`);
}

const compose = synthesis[0];
assert(compose.source_id === "8"
  && compose.function_type === "MagicUnitCompose"
  && compose.input_count === "Unspecified"
  && compose.output_pool.length === 0
  && compose.output_pool_resolution === "Unspecified"
  && compose.cost.amount === "Unspecified"
  && compose.fallback === "ReturnNoLegalCandidateWithoutMutation",
"synthesis policy overclaims unavailable mechanics");
assert(exactOnce(
  upgrades.map(({ input_level: input, output_level: output }) =>
    `${input}:${output}`),
  ["1:2", "2:3"],
), "Scepter upgrade progression drift");
assert(upgrades.every(({ source_id: id, cap, cost, fallback,
  ordered_operations: operations }) =>
  id.startsWith("10:")
    && cap === "3"
    && cost.currency_id === "cosmic-fragments"
    && cost.amount === "Unspecified"
    && fallback === "RejectWithoutMutation"
    && operations.join(",") ===
      "ValidateOwnedScepterLevel,ResolveUnspecifiedCost," +
      "AdvanceExactlyOneReleasedLevel"),
"Scepter upgrade policy drift");
const reforge = reforges[0];
assert(reforge.source_id === "9"
  && reforge.function_type === "MagicUnitReforge"
  && reforge.candidate_set.length === 0
  && reforge.candidate_set_resolution === "Unspecified"
  && reforge.exclude_input_identity === "Unspecified"
  && reforge.ordering === "StableComponentIdAscending"
  && reforge.cost.amount === "Unspecified"
  && reforge.fallback === "ReturnNoLegalCandidateWithoutMutation",
"reforge policy overclaims unavailable mechanics");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
const parentIds = new Set(
  manifest.categories.workbench_functions.records.map(({ id }) => id),
);
assert(["8", "9", "10"].every((id) => parentIds.has(id))
  && rows.every(({ source_id: id }) => parentIds.has(id.split(":")[0])),
"transformation parent function drift");
const scepterLevels = json(
  "content-reference/unknowable-domain-v1/scepter-levels.json",
);
assert(new Set(scepterLevels.map(({ level }) => level)).size === 3,
  "upgrade rules do not match released Scepter levels");

console.log(
  "Unknowable Domain transformations verified (functions 8/9/10; one " +
  "synthesis, two level upgrades, one reforge; unavailable counts/costs/" +
  "pools remain replaceable ProjectPolicy).",
);

function json(relative) {
  return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8"));
}
function unique(values) {
  return new Set(values).size === values.length;
}
function exactOnce(left, right) {
  const ordered = (values) => [...values].sort();
  return JSON.stringify(ordered(left)) === JSON.stringify(ordered(right));
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
