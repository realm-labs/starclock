#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-pools.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const blessings = json(
  "content-reference/unknowable-domain-v1/blessings.json",
);
const memberships = json(
  "content-reference/unknowable-domain-v1/pool-membership.json",
);
assert(blessings.length === 0, "unproven Blessing membership appeared");
assert(memberships.length === 231, "pool-membership denominator drift");
assert(unique(memberships.map(({ id }) => id)), "duplicate pool membership");
for (const row of memberships) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured"
    && row.weight === "Unspecified",
  `${row.id} envelope/weight drift`);
  assert(row.source_refs.length >= 1
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && source.mechanism_quality === "DirectStructured"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
}

const components = memberships.filter(({ member_kind: kind }) =>
  kind === "Component");
const curios = memberships.filter(({ member_kind: kind }) => kind === "Curio");
const occurrences = memberships.filter(({ member_kind: kind }) =>
  kind === "Occurrence");
assert(components.length === 109
  && curios.length === 60
  && occurrences.length === 62,
"pool kind split drift");
assert(components.every(({ ownership, eligibility, alignment_ids: alignments,
  alignment_resolution: resolution, reachability_proof: proof }) =>
  ownership === "UnknowableDomain"
    && eligibility === "CatalogOwnedNotOfferProven"
    && alignments.length === 0
    && resolution === "Unspecified"
    && proof === "DirectModeOwnership"),
"Component catalog was overclaimed as an offer pool");
assert([...curios, ...occurrences].every(({ ownership, eligibility,
  alignment_resolution: resolution, reachability_proof: proof }) =>
  ownership === "Shared"
    && eligibility === "ExactReachable"
    && resolution === "NotApplicable"
    && proof === "ExplicitModeType260"),
"shared type-260 reachability drift");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(manifest.categories.blessings.count === 0,
  "Blessing manifest denominator drift");
for (const [kind, rows, category] of [
  ["component", components, "components"],
  ["curio", curios, "curios"],
  ["occurrence", occurrences, "occurrences"],
]) assert(exactOnce(
  rows.map(({ source_id: id }) => id.replace(`${kind}:`, "")),
  manifest.categories[category].records.map(({ id }) => id),
), `${kind} pool manifest closure drift`);

const alignments = json(
  "content-reference/unknowable-domain-v1/alignments.json",
);
assert(alignments.length === 4
  && alignments.every(({ scepter_candidate_ids: ids }) => ids.length === 6)
  && unique(alignments.flatMap(({ scepter_candidate_ids: ids }) => ids)),
"Alignment Scepter pool drift");
assert(alignments.every(({ component_candidate_ids: ids,
  component_pool_resolution: resolution }) =>
  ids.length === 0 && resolution === "Unspecified"),
"Alignment inferred an unavailable Component pool");

const boundary = fs.readFileSync(path.join(
  root,
  "evidence/unknowable-domain-reference-v1/pool-boundary.md",
), "utf8");
for (const phrase of [
  "zero Blessings",
  "explicit type `260`",
  "CatalogOwnedNotOfferProven",
  "matching names",
])
  assert(boundary.includes(phrase), `pool boundary omits ${phrase}`);

console.log(
  "Unknowable Domain pools verified (0 Blessings; four 6-Scepter Alignment " +
  "pools; 109 catalog-only Components; 60 Curios and 62 Occurrences by " +
  "explicit type 260; no inferred weights/style pools).",
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
