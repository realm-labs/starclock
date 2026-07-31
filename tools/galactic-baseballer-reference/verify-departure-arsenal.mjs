#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const packRoot = path.join(root, "content-reference", "galactic-baseballer-v1");

function option(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}
function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-departure-arsenal.mjs",
  ),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(packRoot, file), "utf8"));
const weapons = await read("weapons.json");
const weaponLevels = await read("weapon-levels.json");
const weaponTriggers = await read("weapon-triggers.json");
const accessories = await read("accessories.json");
const accessoryLevels = await read("accessory-levels.json");
const accessoryBindings = await read("accessory-bindings.json");
const recipes = await read("synthesis-recipes.json");
const inputs = await read("synthesis-inputs.json");

assert(weapons.length === 26, "weapon count drift");
assert(
  weapons.filter(({ tier }) => tier === "Standard").length === 13
    && weapons.filter(({ tier }) => tier === "Legendary").length === 13,
  "weapon tier count drift",
);
assert(weaponLevels.length === 117, "weapon level count drift");
assert(weaponTriggers.length === 26, "weapon trigger count drift");
assert(accessories.length === 16, "accessory count drift");
assert(accessoryLevels.length === 64, "accessory level count drift");
assert(accessoryBindings.length === 16, "accessory binding count drift");
assert(recipes.length === 13, "legendary recipe count drift");
assert(inputs.length === 26, "recipe input count drift");

for (const weapon of weapons) {
  const levels = weaponLevels.filter(({ parent_id: id }) => id === weapon.id);
  assert(
    levels.length === weapon.maximum_level,
    `weapon level closure drift: ${weapon.id}`,
  );
  assert(
    levels.map(({ level }) => level).join(",")
      === Array.from(
        { length: weapon.maximum_level },
        (_, index) => index + 1,
      ).join(","),
    `weapon level sequence drift: ${weapon.id}`,
  );
}
for (const accessory of accessories) {
  const levels = accessoryLevels.filter(({ parent_id: id }) =>
    id === accessory.id);
  assert(levels.length === 4, `accessory level closure drift: ${accessory.id}`);
}
for (const binding of [...weaponTriggers, ...accessoryBindings]) {
  assert(binding.ability_names.length >= 1, `missing abilities: ${binding.id}`);
  assert(
    binding.operation_types.length >= 1
      && /^[0-9a-f]{64}$/u.test(binding.program_fragment_sha256),
    `program summary drift: ${binding.id}`,
  );
  assert(binding.runtime_executable === false, `runtime leak: ${binding.id}`);
}

const graph = new Map(recipes.map(({ output_weapon_id: output, id }) => [
  id,
  {
    output,
    inputs: inputs.filter(({ recipe_id: recipeId }) => recipeId === id),
  },
]));
for (const [recipeId, node] of graph) {
  assert(node.inputs.length === 2, `recipe arity drift: ${recipeId}`);
  const weaponInput = node.inputs.find(({ input_kind: kind }) =>
    kind === "Weapon");
  const accessoryInput = node.inputs.find(({ input_kind: kind }) =>
    kind === "Accessory");
  assert(
    weaponInput?.required_level === 8 && weaponInput.consumed === true,
    `weapon prerequisite/consumption drift: ${recipeId}`,
  );
  assert(
    accessoryInput?.required_level === 1 && accessoryInput.consumed === false,
    `accessory prerequisite/consumption drift: ${recipeId}`,
  );
  assert(
    !node.inputs.some(({ input_id: input }) => input === node.output),
    `self-cycle in recipe: ${recipeId}`,
  );
}
const outputs = new Set([...graph.values()].map(({ output }) => output));
assert(
  inputs.every(({ input_id: input }) => !outputs.has(input)),
  "legendary recipe graph contains a cycle",
);
const linkedMazeBuffManifestIds = new Set(
  [...weaponLevels, ...accessoryLevels].flatMap(
    ({ manifest_record_ids: ids }) => ids.filter((id) =>
      id.includes("EvolveBuildMazeBuff")),
  ),
);
assert(
  linkedMazeBuffManifestIds.size === 181,
  "GearConfig-to-MazeBuff exact-once closure drift",
);

console.log(
  "Departure arsenal verified: 26 weapons, 117 weapon levels, "
  + "16 accessories, 64 accessory levels, 13 acyclic Legendary recipes",
);
