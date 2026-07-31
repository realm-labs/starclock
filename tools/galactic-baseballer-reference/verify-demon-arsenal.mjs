#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const sourceCache = option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/galactic-baseballer-source");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);

function option(name) {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  if (value === undefined || value.startsWith("--"))
    throw new Error(`${name} requires a path`);
  return value;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-demon-arsenal.mjs",
  ),
  "--check",
  "--source-cache",
  sourceCache,
], { cwd: root, stdio: "inherit" });
execFileSync(process.execPath, [
  path.join(
    "tools",
    "galactic-baseballer-reference",
    "normalize-demon-arsenal-fixtures.mjs",
  ),
  "--check",
], { cwd: root, stdio: "inherit" });

const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));
const weapons = await read("demon-weapons.json");
const weaponLevels = await read("demon-weapon-levels.json");
const weaponTriggers = await read("demon-weapon-triggers.json");
const accessories = await read("demon-accessories.json");
const accessoryLevels = await read("demon-accessory-levels.json");
const accessoryBindings = await read("demon-accessory-bindings.json");
const recipes = await read("demon-synthesis-recipes.json");
const inputs = await read("demon-synthesis-inputs.json");
const rules = await read("demon-arsenal-mechanic-rules.json");
const fixtures = await read("demon-arsenal-review-fixtures.json");

assert(weapons.length === 29, "Demon King weapon count drift");
const tierCounts = Object.fromEntries(
  ["Standard", "Legendary", "Twin", "Supreme"].map((tier) => [
    tier,
    weapons.filter(({ tier: rowTier }) => rowTier === tier).length,
  ]),
);
assert(
  JSON.stringify(tierCounts) === JSON.stringify({
    Standard: 15,
    Legendary: 12,
    Twin: 1,
    Supreme: 1,
  }),
  "Demon King weapon tier counts drift",
);
assert(weaponLevels.length === 134, "Demon King weapon-level count drift");
assert(weaponTriggers.length === 29, "Demon King trigger count drift");
assert(accessories.length === 16, "Demon King accessory count drift");
assert(accessoryLevels.length === 64, "Demon King accessory-level count drift");
assert(accessoryBindings.length === 16, "accessory binding count drift");

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
  assert(
    levels.length === accessory.maximum_level
      && accessory.maximum_level === 4,
    `accessory level closure drift: ${accessory.id}`,
  );
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
const rangerBinding = weaponTriggers.find(({ parent_id: id }) =>
  id === "galactic-baseballer.demon-king.weapon.3113003");
assert(
  rangerBinding.actor_program_bindings.length === 3
    && rangerBinding.actor_program_bindings.every(({ ability_names: names }) =>
      names.length >= 1),
  "Ranger's Badge actor-program closure drift",
);
assert(
  weaponTriggers.filter(({ actor_program_bindings: bindings }) =>
    bindings.length > 0).length === 1,
  "unexpected actor-program binding drift",
);

const collectionManifestIds = new Set(
  [...weapons, ...accessories].flatMap(({ manifest_record_ids: ids }) =>
    ids.filter((id) => id.includes("EvoBdSCGearCollection"))),
);
const typeManifestIds = new Set(
  [...weapons, ...accessories].flatMap(({ manifest_record_ids: ids }) =>
    ids.filter((id) => id.includes("EvoBdSCGearTypeConfig"))),
);
assert(collectionManifestIds.size === 45, "collection exact-once closure drift");
assert(typeManifestIds.size === 5, "gear-type closure drift");

const gearManifestIds = new Set(
  [...weaponLevels, ...accessoryLevels].flatMap(
    ({ manifest_record_ids: ids }) => ids.filter((id) =>
      id.includes("EvoBdSCGearConfig")),
  ),
);
const mazeBuffManifestIds = new Set(
  [...weaponLevels, ...accessoryLevels].flatMap(
    ({ manifest_record_ids: ids }) => ids.filter((id) =>
      id.includes("EvoBdSCMazeBuff")),
  ),
);
assert(
  gearManifestIds.size === 198 && mazeBuffManifestIds.size === 198,
  "GearConfig-to-MazeBuff exact closure drift",
);

assert(recipes.length === 14, "Demon King recipe count drift");
assert(inputs.length === 28, "Demon King recipe input count drift");
assert(
  recipes.filter(({ tier }) => tier === "Legendary").length === 12
    && recipes.filter(({ tier }) => tier === "Twin").length === 1
    && recipes.filter(({ tier }) => tier === "Supreme").length === 1,
  "advanced recipe tier count drift",
);
for (const recipe of recipes) {
  const recipeInputs = inputs.filter(({ recipe_id: id }) => id === recipe.id);
  assert(recipeInputs.length === 2, `recipe arity drift: ${recipe.id}`);
  assert(
    recipe.candidate_precedence.includes(
      "Supreme, Twin, Legendary",
    ) && recipe.failure_behavior.includes("without inventory mutation"),
    `recipe policy boundary drift: ${recipe.id}`,
  );
  if (recipe.tier === "Legendary") {
    const weaponInput = recipeInputs.find(({ input_kind: kind }) =>
      kind === "Weapon");
    const accessoryInput = recipeInputs.find(({ input_kind: kind }) =>
      kind === "Accessory");
    assert(
      weaponInput?.required_level === 8 && weaponInput.consumed === true,
      `Legendary weapon prerequisite drift: ${recipe.id}`,
    );
    assert(
      accessoryInput?.required_level === 1
        && accessoryInput.consumed === false,
      `Legendary accessory retention drift: ${recipe.id}`,
    );
  }
}
const twin = recipes.find(({ tier }) => tier === "Twin");
const twinInputs = inputs.filter(({ recipe_id: id }) => id === twin.id);
assert(
  twin.output_weapon_id.endsWith(".3113201")
    && twinInputs.every(({ input_kind: kind, required_level: level, consumed }) =>
      kind === "Weapon" && level === 8 && consumed)
    && twin.consumption_order.map(
      ({ input_source_numeric_id: id }) => id,
    ).join(",") === "3113005,3113006",
  "Twin recipe edge/order drift",
);
const supreme = recipes.find(({ tier }) => tier === "Supreme");
const supremeInputs = inputs.filter(({ recipe_id: id }) => id === supreme.id);
assert(
  supreme.output_weapon_id.endsWith(".3113301")
    && supremeInputs.map(({ input_id: id, required_level: level }) =>
      `${id.slice(id.lastIndexOf(".") + 1)}:${level}`).sort().join(",")
      === "3113014:8,3113901:1"
    && supremeInputs.every(({ consumed }) => consumed)
    && supreme.consumption_order.map(
      ({ input_source_numeric_id: id }) => id,
    ).join(",") === "3113901,3113014",
  "Supreme recipe edge/order drift",
);

const graph = new Map(recipes.map((recipe) => [
  recipe.output_weapon_id,
  inputs.filter(({ recipe_id: id, input_kind: kind }) =>
    id === recipe.id && kind === "Weapon").map(({ input_id: id }) => id),
]));
const visiting = new Set();
const visited = new Set();
function visit(node) {
  if (visiting.has(node)) throw new Error(`synthesis cycle: ${node}`);
  if (visited.has(node)) return;
  visiting.add(node);
  for (const input of graph.get(node) ?? []) visit(input);
  visiting.delete(node);
  visited.add(node);
}
for (const output of graph.keys()) visit(output);

const ruinBotLevels = weaponLevels.filter(({ parent_id: id, level }) =>
  id.endsWith(".3113002") && (level === 7 || level === 8));
assert(
  ruinBotLevels.length === 2
    && ruinBotLevels.every(({ released_correction_ids: ids }) =>
      ids.join(",")
        === "galactic-baseballer.correction.v3_4.ruinbot-level-7-8"),
  "RuinBot correction binding drift",
);
assert(
  ruinBotLevels.find(({ level }) => level === 7).parameter_values.join(",")
    === "0,9,70,14,0.75,30,0.3,0,0,0,0,0,0,0,0,0,0,0,0,0"
    && ruinBotLevels.find(({ level }) => level === 8)
      .parameter_values.join(",")
      === "0,12,70,14,0.75,30,0.3,0,0,0,0,0,0,0,0,0,0,0,0,0",
  "RuinBot retained Version 4.4 vectors drift",
);

assert(
  rules.length === 2
    && rules.map(({ family_id: id }) => id).sort().join(",")
      === "supreme-weapon-synthesis,twin-weapon-synthesis",
  "advanced semantic rule family drift",
);
assert(fixtures.length === 6, "Demon King arsenal fixture count drift");
for (const rule of rules) {
  assert(
    rule.fixture_ids.length === 2
      && rule.fixture_ids.every((fixtureId) =>
        fixtures.some(({ id }) => id === fixtureId)),
    `advanced fixture link drift: ${rule.id}`,
  );
}
for (const familyId of [
  "twin-weapon-synthesis",
  "supreme-weapon-synthesis",
]) {
  const familyFixtures = fixtures.filter(({ family_id: id }) => id === familyId);
  assert(
    familyFixtures.length === 2
      && familyFixtures.some(({ expected_facts: facts }) =>
        facts.inventory_byte_identical === true)
      && familyFixtures.some(({ expected_facts: facts }) =>
        facts.output_level === 1),
    `advanced success/rejection fixture drift: ${familyId}`,
  );
}
assert(
  fixtures.filter(({ id }) =>
    id.startsWith("fixture.galactic-baseballer.demon-king.weapon-ruinbot"))
    .length === 2,
  "RuinBot correction fixture count drift",
);

console.log(
  "Demon King arsenal verified: 29 weapons/134 levels, 16 accessories/"
  + "64 levels, 12 Legendary + 1 Twin + 1 Supreme acyclic recipes, "
  + "2 advanced rules and 6 fixtures",
);
