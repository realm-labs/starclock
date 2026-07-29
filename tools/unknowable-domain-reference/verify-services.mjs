#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-services.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const workbenches = json(
  "content-reference/unknowable-domain-v1/workbenches.json",
);
const functions = json(
  "content-reference/unknowable-domain-v1/workbench-functions.json",
);
const groups = json(
  "content-reference/unknowable-domain-v1/gamble-groups.json",
);
const units = json(
  "content-reference/unknowable-domain-v1/gamble-units.json",
);
const offers = json(
  "content-reference/unknowable-domain-v1/service-offer-rules.json",
);
assert(workbenches.length === 4, "Workbench denominator drift");
assert(functions.length === 5, "Workbench function denominator drift");
assert(groups.length === 10, "Gamble group denominator drift");
assert(units.length === 7, "Gamble unit denominator drift");
assert(offers.length === 15, "Service offer policy denominator drift");
const allRows = [...workbenches, ...functions, ...groups, ...units, ...offers];
assert(unique(allRows.map(({ id }) => id)), "duplicate service stable ID");
for (const row of allRows) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length >= 1
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
}

const functionIds = new Set(functions.map(({ id }) => id));
assert(workbenches.every(({ function_ids: ids, eligibility, lifecycle }) =>
  ids.length === 3
    && ids.every((id) => functionIds.has(id))
    && eligibility === "Unspecified"
    && lifecycle === "Unspecified"),
"Workbench binding/lifecycle drift");
assert(exactOnce(functions.map(({ function_type: type }) => type), [
  "MagicScepterShop",
  "MagicUnitShop",
  "MagicUnitCompose",
  "MagicUnitReforge",
  "MagicScepterLevelUp",
]), "Workbench function taxonomy drift");
assert(functions.filter(({ currency_id: id }) =>
  id === "cosmic-fragments").length === 3
  && functions.every(({ price }) => price === "Unspecified"),
"Workbench price/currency boundary drift");

assert(groups.filter(({ gamble_type: type }) => type === "SlotMachine")
  .length === 7
  && groups.filter(({ gamble_type: type }) => type === "FortuneWheel")
    .length === 3,
"Gamble group type split drift");
assert(groups.every(({ unit_ids: ids, unit_binding_resolution: resolution }) =>
  ids.length === 0 && resolution === "Unspecified"),
"Gamble group inferred an unavailable unit binding");
assert(exactOnce(groups.filter(({ gamble_type: type }) =>
  type === "FortuneWheel").map(({ group_level: level }) => level),
["Common", "Normal", "Grand"]),
"FortuneWheel level boundary drift");
assert(exactOnce(units.map(({ unit_type: type }) => type), [
  "MagicUnitRare",
  "MagicUnitRare",
  "MagicUnitCommon",
  "MagicUnitCommon",
  "MiracleCommon",
  "MiracleCommon",
  "MiracleCommon",
]), "Gamble unit type split drift");
assert(units.every(({ parameters, parameter_target_resolution: target,
  outcome_program: outcome }) =>
  parameters.length === 1
    && target === "Unspecified"
    && outcome.resolution === "Unspecified"
    && outcome.referenced_ids.length === 0),
"Gamble unit overclaims parameter/outcome semantics");

const expectedOfferIds = new Set([
  ...functions.map(({ offer_policy_id: id }) => id),
  ...groups.map(({ offer_policy_id: id }) => id),
]);
assert(expectedOfferIds.size === 15
  && offers.every(({ id }) => expectedOfferIds.has(id)),
"Service offer reference drift");
for (const offer of offers) {
  assert(offer.evidence_quality === "ProjectPolicy"
    && offer.candidate_set.length === 0
    && offer.candidate_set_resolution === "Unspecified"
    && offer.ordering === "StableSourceIdAscending"
    && offer.refresh === "Unspecified"
    && offer.price === "Unspecified"
    && offer.eligibility === "Unspecified"
    && offer.no_legal_candidate ===
      "ReturnNoLegalCandidateWithoutMutation"
    && offer.source_refs.some(({ evidence_quality: quality, note,
      replacement_condition: replacement }) =>
      quality === "ProjectPolicy" && note?.length > 0 && replacement?.length > 0),
  `${offer.id} offer policy drift`);
}

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
for (const [label, normalized, category] of [
  ["Workbench", workbenches, "workbenches"],
  ["Workbench function", functions, "workbench_functions"],
  ["Gamble group", groups, "gamble_groups"],
  ["Gamble unit", units, "gamble_units"],
]) assert(exactOnce(
  normalized.map(({ source_id: id }) => id),
  manifest.categories[category].records.map(({ id }) => id),
), `${label} manifest closure drift`);

console.log(
  "Unknowable Domain services verified (4 workbenches; 5 functions; 10 " +
  "gamble groups; 7 units; 15 replaceable offer policies; unavailable " +
  "prices/group bindings/refresh remain fail-closed).",
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
