#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/gold-and-gears-reference/import-paths.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);

const expected = new Map([
  ["paths.json", ["paths", 9]],
  ["resonances.json", ["resonances", 36]],
  ["path-boosts.json", ["path_boosts", 9]],
  ["resonance-extrapolations.json", ["resonance_extrapolations", 36]],
  ["resonance-interplays.json", ["resonance_interplays", 18]],
  ["bonuses.json", ["trailblaze_bonuses", 5]],
]);
const data = new Map([...expected].map(([file]) => [
  file,
  json(`content-reference/gold-and-gears-v1/${file}`),
]));
const allRows = [...data.values()].flat();
assert(unique(allRows.map(({ id }) => id)), "Path pack has duplicate IDs");
for (const [file, [, count]] of expected)
  assert(data.get(file).length === count, `${file} count drift`);
for (const row of allRows) {
  assert(row.schema_revision === "starclock.gold-and-gears-row.v1"
    && row.coverage_state === "DataReady",
  `${row.id} common envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field].trim() !== "",
      `${row.id} has empty ${field}`);
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
  assert(Array.isArray(row.source_refs) && row.source_refs.length > 0,
    `${row.id} has no provenance`);
  for (const source of row.source_refs) {
    for (const field of [
      "source_id", "repository", "revision", "path", "locator", "sha256",
      "access_date", "evidence_quality",
    ])
      assert(typeof source[field] === "string" && source[field] !== "",
        `${row.id} source ref omits ${field}`);
    assert(/^[0-9a-f]{64}$/u.test(source.sha256),
      `${row.id} source digest drift`);
    if (source.evidence_quality === "ProjectPolicy")
      assert(source.note?.length > 0 && source.replacement_condition?.length > 0,
        `${row.id} policy is not replaceable`);
  }
}

const paths = data.get("paths.json");
const pathIds = new Set(paths.map(({ id }) => id));
assert(paths.every(({ ownership }) => ownership === "Shared"),
  "Path ownership drift");
assert(unique(paths.map(({ source_id }) => source_id))
  && unique(paths.map(({ sort }) => sort)),
"Path source/sort drift");

const resonances = data.get("resonances.json");
assert(resonances.every(({ ownership, path_id: pathId }) =>
  ownership === "Shared" && pathIds.has(pathId)),
"Resonance ownership/path drift");
for (const pathRow of paths) {
  const owned = resonances.filter(({ path_id: id }) => id === pathRow.id);
  assert(owned.length === 4
    && owned.filter(({ resonance_kind: kind }) => kind === "Resonance").length
      === 1
    && owned.filter(({ resonance_kind: kind }) => kind === "Formation").length
      === 3,
  `${pathRow.id} Resonance/Formation closure drift`);
}

const expectedStats = new Map(Object.entries({
  1: ["ShieldGain", "ShieldTakenRatio"],
  2: ["EffectHitRate", "StatusProbabilityBase"],
  3: ["DamageOverTime", "DotDamageAddedRatio"],
  4: ["OutgoingHealing", "HealTakenRatio"],
  5: ["CriticalDamage", "CriticalDamageBase"],
  6: ["DamageDealt", "AllDamageTypeAddedRatio"],
  7: ["FollowUpAttackDamage", "FollowUpAttackDamageRatio"],
  8: ["BasicAttackDamage", "BasicAttackDamageRatio"],
  9: ["UltimateDamage", "UltimateDamageRatio"],
}));
const boosts = data.get("path-boosts.json");
for (const boost of boosts) {
  assert(pathIds.has(boost.path_id)
    && boost.dice_path_value_ids.length === 12
    && boost.allowed_increment_values.length > 0
    && boost.source_value_conversion
      === "PercentInputDividedBy100ByStageAbility",
  `${boost.id} binding shape drift`);
  const expectedStat = expectedStats.get(boost.aeon_source_id);
  assert(expectedStat?.[0] === boost.boost_stat
    && expectedStat?.[1] === boost.target_property,
  `${boost.id} target property drift`);
}

const extrapolations = data.get("resonance-extrapolations.json");
assert(extrapolations.filter(({ enhanced }) => !enhanced).length === 9
  && extrapolations.filter(({ enhanced }) => enhanced).length === 27,
"Resonance Extrapolation base/enhanced distribution drift");
for (const row of extrapolations)
  assert(pathIds.has(row.path_id)
    && resonances.some(({ id }) => id === row.shared_resonance_id)
    && row.battle_scope === "ThirdPlaneBossBattle"
    && row.controller_policy.policy_id
      === "resonance-extrapolation-controller-v1"
    && row.controller_policy.action_and_polarity_lowering
      === "UnresolvedFailClosed"
    && row.quality_overrides.length === 1,
  `${row.id} controller/shared binding drift`);

const interplays = data.get("resonance-interplays.json");
for (const row of interplays)
  assert(pathIds.has(row.main_path_id) && pathIds.has(row.sub_path_id)
    && row.main_path_id !== row.sub_path_id
    && row.main_blessing_threshold === 3
    && row.sub_blessing_threshold === 3
    && row.source_parameters.every(({ value }) => value !== ""),
  `${row.id} interplay threshold drift`);
for (const pathId of pathIds)
  assert(interplays.filter(({ main_path_id: id }) => id === pathId).length === 2,
    `${pathId} interplay count drift`);

const bonuses = data.get("bonuses.json");
assert(JSON.stringify(bonuses.map(({ source_id: id }) => id))
  === JSON.stringify(["201", "202", "203", "204", "205"]),
"Trailblaze Bonus identity drift");
assert(bonuses.find(({ source_id: id }) => id === "201")
  .effect_contributions[0].value === "150",
"Fragmented Universe value drift");
assert(bonuses.find(({ source_id: id }) => id === "204")
  .effect_contributions[0].value === "1",
"Inorganic Universe cheat value drift");
assert(bonuses.find(({ source_id: id }) => id === "205")
  .effect_contributions[0].grants.length === 2,
"Equilibrium Universe grant drift");

const manifest = json(
  "content-manifests/gold-and-gears-v1/content-manifest.json",
);
for (const [file, [category]] of expected) {
  const identityField =
    category === "paths" || category === "resonances" ? "id" : "source_id";
  const actual = data.get(file).map((row) => row[identityField]).sort();
  const required = manifest.categories[category].records
    .map(({ id }) => id).sort();
  assert(JSON.stringify(actual) === JSON.stringify(required),
    `${file} manifest exact-once drift`);
}

console.log(
  "Gold and Gears Paths verified (9 Paths; 36 Resonances; 9 boosts; " +
  "36 Extrapolation bindings; 18 Interplays; 5 Trailblaze Bonuses).",
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
