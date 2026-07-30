#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-progression.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const data = new Map([
  ["mode_constants", json(
    "content-reference/unknowable-domain-v1/mode-constants.json",
  )],
  ["talents", json("content-reference/unknowable-domain-v1/talents.json")],
  ["unlocks", json("content-reference/unknowable-domain-v1/unlocks.json")],
  ["layer_effects", json(
    "content-reference/unknowable-domain-v1/layer-effects.json",
  )],
  ["maze_buffs", json(
    "content-reference/unknowable-domain-v1/maze-buffs.json",
  )],
  ["score_inputs", json(
    "content-reference/unknowable-domain-v1/score-inputs.json",
  )],
  ["progression_effects", json(
    "content-reference/unknowable-domain-v1/progression-effects.json",
  )],
]);
const expected = {
  mode_constants: 14,
  talents: 25,
  unlocks: 30,
  layer_effects: 1,
  maze_buffs: 387,
  score_inputs: 133,
  progression_effects: 576,
};
for (const [name, count] of Object.entries(expected))
  assert(data.get(name).length === count, `${name} denominator drift`);
const allRows = [...data.values()].flat();
assert(unique(allRows.map(({ id }) => id)), "duplicate progression stable ID");
for (const row of allRows) {
  assert(row.schema_revision === "starclock.unknowable-domain-row.v1"
    && row.ownership === "UnknowableDomain"
    && row.coverage_state === "DataReady"
    && row.evidence_quality === "ExactStructured",
  `${row.id} envelope drift`);
  for (const field of ["name_en", "name_zh_cn", "summary_en", "summary_zh_cn"])
    assert(typeof row[field] === "string" && row[field] !== "",
      `${row.id} lacks ${field}`);
  assert(row.source_refs.length >= 1
    && row.source_refs.every((source) =>
      source.game_version === "4.4"
        && source.mechanism_quality === "DirectStructured"
        && /^[0-9a-f]{64}$/u.test(source.sha256)),
  `${row.id} provenance drift`);
}

const constants = data.get("mode_constants");
assert(constants.filter(({ value_type: type }) => type === "Integer").length === 8
  && constants.filter(({ value_type: type }) =>
    type === "IntegerArray").length === 6,
"mode constant type split drift");
assert(constants.every(({ consumer_ids: ids, consumer_resolution: resolution }) =>
  ids.length === 0 && resolution === "Unspecified"),
"mode constant inferred unavailable consumers");

const componentLevelIds = new Set(
  json("content-reference/unknowable-domain-v1/component-levels.json")
    .map(({ id }) => id),
);
const talents = data.get("talents");
assert(talents.map(({ level }) => Number(level)).sort((a, b) => a - b)
  .every((level, index) => level === index + 1),
"Talent level progression drift");
assert(talents.every(({ cost, prerequisite_ids: prerequisites,
  prerequisite_resolution: resolution, effect_ids: effects }) =>
  cost.length === 1
    && prerequisites.length === 0
    && resolution === "Unspecified"
    && effects.every((id) => componentLevelIds.has(id))),
"Talent cost/prerequisite/effect binding drift");
assert(talents.flatMap(({ effect_ids: ids }) => ids).length === 20,
  "Talent Component-reference denominator drift");

const finishIds = new Set(
  json("content-reference/unknowable-domain-v1/finish-conditions.json")
    .map(({ id }) => id),
);
const unlocks = data.get("unlocks");
assert(unlocks.every(({ finish_condition_id: id,
  consequence, evaluation_boundary: boundary }) =>
  finishIds.has(id)
    && consequence === "EnableSourceRowsReferencingUnlockID"
    && ["AfterFinishConditionSatisfied", "Unspecified"].includes(boundary)),
"Unlock finish/consequence boundary drift");
assert(unlocks.some(({ consumer_source_locators: consumers }) =>
  consumers.length > 0),
"Unlock consumer closure is unexpectedly empty");

const layerEffect = data.get("layer_effects")[0];
assert(layerEffect.parameters.join(",") === "500,4"
  && layerEffect.trigger === "Unspecified"
  && layerEffect.ordered_operations.join(",") ===
    "GrantCosmicFragments:500,GrantRandomComponents:4"
  && layerEffect.component_pool_resolution === "Unspecified",
"layer-effect boundary drift");

const mazeBuffs = data.get("maze_buffs");
assert(new Set(mazeBuffs.map(({ binding }) => binding.key)).size === 171,
  "maze-buff ability binding denominator drift");
assert(mazeBuffs.every(({ series, rarity, maze_buff_type: type, binding,
  battle_projection: projection }) =>
  series === "1"
    && rarity === "1"
    && type === "Level"
    && binding.type === "StageAbilityBeforeCharacterBorn"
    && binding.ability_path.startsWith(
      "Config/ConfigAbility/Level/Level_RogueMagic_Ability_")
    && projection === "SourceProgramPreservedNotLowered"),
"maze-buff binding drift");

const scores = data.get("score_inputs");
const worldCounts = Object.fromEntries(
  [...Map.groupBy(scores, ({ world_level: world }) => world).entries()]
    .map(([world, rows]) => [world, rows.length]),
);
assert(["default", "1", "2", "3", "4", "5", "6"]
  .every((world) => worldCounts[world] === 19)
  && Object.keys(worldCounts).length === 7
  && scores.every(({ account_reward_ids: rewards }) => rewards.length === 0),
"score-input/default-world boundary drift");

const effects = data.get("progression_effects");
const expectedEffectCounts = {
  Talent: 25,
  Unlock: 30,
  LayerEffect: 1,
  MazeBuff: 387,
  ScoreInput: 133,
};
const actualEffectCounts = Object.fromEntries(
  [...Map.groupBy(effects, ({ source_kind: kind }) => kind).entries()]
    .map(([kind, rows]) => [kind, rows.length]),
);
assert(Object.entries(expectedEffectCounts).every(([kind, count]) =>
  actualEffectCounts[kind] === count)
  && Object.keys(actualEffectCounts).length ===
    Object.keys(expectedEffectCounts).length,
"progression contribution split drift");
assert(effects.every(({ runtime_lowered: lowered }) => lowered === false),
  "progression reference unexpectedly claims runtime lowering");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
for (const category of [
  "mode_constants",
  "talents",
  "unlocks",
  "layer_effects",
  "maze_buffs",
  "score_inputs",
]) assert(exactOnce(
  data.get(category).map(({ source_id: id }) => id),
  manifest.categories[category].records.map(({ id }) => id),
), `${category} manifest closure drift`);

console.log(
  "Unknowable Domain progression verified (14 constants; 25 Talents; 30 " +
  "unlocks; 1 layer effect; 387 maze buffs/171 ability keys; 133 score " +
  "inputs; 576 non-runtime contribution rows).",
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
