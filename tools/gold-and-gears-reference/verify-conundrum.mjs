#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const decimalPattern =
  /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u;

execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-conundrum.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const levels = json("content-reference/gold-and-gears-v1/conundrum-levels.json");
assert(Array.isArray(levels) && levels.length === 12,
  "Conundrum level count drift");
assert(unique(levels.map(({ id }) => id)), "Conundrum IDs are not unique");
assert(unique(levels.map(({ rule_contribution_id: id }) => id)),
  "Conundrum contribution IDs are not unique");

const bySource = new Map(levels.map((level) => [level.source_id, level]));
for (const level of levels) {
  assert(level.schema_revision === "starclock.gold-and-gears-row.v1"
    && level.kind === "ConundrumLevel"
    && level.ownership === "GoldAndGears"
    && level.coverage_state === "DataReady"
    && level.evidence_quality === "ExactStructured",
  `${level.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof level[field] === "string" && level[field].trim() !== "",
      `${level.id} has empty ${field}`);
  assert(JSON.stringify(level.tags) === JSON.stringify([...level.tags].sort()),
    `${level.id} tags are not canonical`);
  assert(level.track_cap === 6 && level.total_conundrum_cap === 12
    && level.total_level_formula === "stats_level + auxiliary_level"
    && level.unlock_requirement.target === "gold-gears.area.405",
  `${level.id} cap or unlock boundary drift`);
  assert(level.source_refs[0].path
    === "ExcelOutput/RogueNousDifficultyLevel.json",
  `${level.id} primary source drift`);
  assert(level.source_refs.some(({ source_id: id }) =>
    id === "source.goal08.public.gold-gears-conundrum-composition"),
  `${level.id} omits composition evidence`);
  for (const source of level.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${level.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${level.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${level.id} project policy is not replaceable`);
  }
  assert(/^[0-9a-f]{64}$/u.test(level.source_description_sha256_en)
    && /^[0-9a-f]{64}$/u.test(level.source_description_sha256_zh_cn),
  `${level.id} description digest drift`);
  assert(level.source_parameters.every(({ index, value }, parameterIndex) =>
    index === parameterIndex + 1 && decimalPattern.test(value)),
  `${level.id} parameter transport drift`);
  assert(level.effect_contributions.length === 1,
    `${level.id} contribution count drift`);
  for (const contributionId of level.active_contribution_ids)
    assert(levels.some(({ rule_contribution_id: id }) => id === contributionId),
      `${level.id} unresolved active contribution ${contributionId}`);
}

const stats = levels.filter(({ track }) => track === "Stats");
const auxiliary = levels.filter(({ track }) => track === "Auxiliary");
assert(stats.length === 6 && auxiliary.length === 6,
  "Conundrum track count drift");
assert(JSON.stringify(stats.map(({ level }) => level))
  === JSON.stringify([1, 2, 3, 4, 5, 6]),
"Stats level ordering drift");
assert(JSON.stringify(auxiliary.map(({ level }) => level))
  === JSON.stringify([1, 2, 3, 4, 5, 6]),
"Auxiliary level ordering drift");
assert(stats.every(({ composition_mode }) =>
  composition_mode === "LatestContributionPerSourceTagAtOrBelowSelectedLevel"),
"Stats composition drift");
assert(auxiliary.every(({ composition_mode, level, active_contribution_ids }) =>
  composition_mode === "AllContributionsAtOrBelowSelectedLevel"
    && active_contribution_ids.length === level),
"Auxiliary cumulative composition drift");
assert(JSON.stringify(stats.map(({ active_contribution_ids: ids }) =>
  ids.map((id) => id.split(".").at(-1))))
  === JSON.stringify([
    ["1"], ["2"], ["2", "3"], ["3", "4"], ["3", "4", "5"],
    ["3", "5", "6"],
  ]),
"Stats replacement composition drift");

for (const sourceId of ["101", "102", "103", "104", "105", "106"]) {
  const level = bySource.get(sourceId);
  const contribution = level.effect_contributions[0];
  assert(level.mechanism_quality === "ExactStructuredWithPolicyFields"
    && level.quality_overrides.length === 1
    && level.source_refs.some(({ evidence_quality: quality }) =>
      quality === "ProjectPolicy")
    && contribution.mechanism_quality === "ProjectPolicy"
    && contribution.numeric_binding.policy_id
      === "conundrum-unreleased-numeric-bindings-v1"
    && contribution.numeric_binding.resolution_state === "UnresolvedFailClosed"
    && contribution.numeric_binding.authoritative_behavior
      === "RejectBattleCompilation",
  `${level.id} unpublished numeric boundary drift`);
}
assert(JSON.stringify(bySource.get("102").replaces_level_ids)
  === JSON.stringify(["gold-gears.conundrum-level.stats.1"])
  && JSON.stringify(bySource.get("104").replaces_level_ids)
    === JSON.stringify(["gold-gears.conundrum-level.stats.2"])
  && JSON.stringify(bySource.get("106").replaces_level_ids)
    === JSON.stringify(["gold-gears.conundrum-level.stats.4"]),
"Stats replacement links drift");

assert(bySource.get("201").effect_contributions[0].value === "1",
  "Auxiliary +1 Formation count drift");
assert(bySource.get("202").effect_contributions[0].encounter_binding_state
  === "DataReady"
  && bySource.get("202").effect_contributions[0].encounter_group_ids.length
    === 12,
"Auxiliary +2 encounter binding drift");
assert(bySource.get("203").effect_contributions[0].value === "20"
  && bySource.get("203").source_parameters[0].value === "20",
"Auxiliary +3 cost drift");
assert(JSON.stringify(bySource.get("204").source_parameters.map(({ value }) =>
  value)) === JSON.stringify(["1", "1", "100"]),
"Auxiliary +4 parameter drift");
const resourceChange = bySource.get("204").effect_contributions[0];
assert(resourceChange.countdown_delta === "-1"
  && resourceChange.dice_reroll_delta === "-1"
  && resourceChange.cosmic_fragment_delta === "-100",
"Auxiliary +4 resource delta drift");
assert(bySource.get("205").effect_contributions[0].pool_binding_state
  === "DataReady"
  && bySource.get("205").effect_contributions[0].selection_pool_id
    === "gold-gears.curio-pool.negative"
  && bySource.get("205").effect_contributions[0].unresolved_pool_behavior
    === "FailClosed",
"Auxiliary +5 Curio binding drift");
assert(bySource.get("206").effect_contributions[0].value === "-1"
  && bySource.get("206").effect_contributions[0].minimum_effective_count === "0",
"Auxiliary +6 blessing-count drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
const actual = levels.map(({ source_id: id }) => id).sort();
const required = manifest.categories.conundrum_levels.records
  .map(({ id }) => id).sort();
assert(JSON.stringify(actual) === JSON.stringify(required),
  "Conundrum manifest exact-once drift");

console.log(
  "Gold and Gears Conundrum verified (12 levels; 6+6 tracks; total cap 12; " +
  "exact composition and explicit fail-closed numeric bindings).",
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
