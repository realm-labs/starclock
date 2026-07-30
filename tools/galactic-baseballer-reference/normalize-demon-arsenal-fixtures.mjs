#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const outputRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
const profileId = "galactic-baseballer.demon-king.v3_3";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const approximationRegister = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "galactic-baseballer-v1",
  "approximation-register.json",
), "utf8"));
const read = async (file) =>
  JSON.parse(await readFile(path.join(outputRoot, file), "utf8"));
const recipes = await read("demon-synthesis-recipes.json");
const inputs = await read("demon-synthesis-inputs.json");
const weaponLevels = await read("demon-weapon-levels.json");
const releasedCorrections = await read("demon-released-corrections.json");

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) =>
      `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value) {
  return createHash("sha256").update(canonical(value)).digest("hex");
}

function policySource(id) {
  const record = approximationRegister.records.find(({ id: recordId }) =>
    recordId === id);
  if (record === undefined) throw new Error(`missing policy record: ${id}`);
  return {
    source_id: `source.goal16.policy.${digest(record).slice(0, 16)}`,
    repository_or_url: "starclock",
    revision_or_access_date: approximationRegister.schema_revision,
    game_version: "4.4",
    path_or_page:
      "content-manifests/galactic-baseballer-v1/approximation-register.json",
    locator: id,
    sha256: digest(record),
    evidence_quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
    note: "explicit deterministic synthesis policy; not an observed parity claim",
    replacement_condition: record.replacement_condition,
  };
}

function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  manifestIds,
  sourceRefs,
  tags,
  mechanismQuality = "ExactRelationship",
}) {
  return {
    id,
    schema_revision: rowRevision,
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    profile_ids: [profileId],
    ownership: "DemonKing",
    coverage_state: "Researched",
    evidence_quality: sourceRefs.some(({ evidence_quality: quality }) =>
      quality === "ProjectPolicy") ? "ProjectPolicy" : "ExactStructured",
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...new Set(manifestIds)].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
  };
}

function synthesisRule(familyId, tier, order) {
  const recipe = recipes.find(({ tier: recipeTier }) => recipeTier === tier);
  if (recipe === undefined) throw new Error(`${tier} recipe missing`);
  const recipeInputs = inputs.filter(({ recipe_id: id }) => id === recipe.id);
  const policy = policySource("gb.policy.simultaneous-synthesis-order");
  return {
    ...envelope({
      id: `galactic-baseballer.demon-king.rule.${familyId}`,
      kind: "MechanicRule",
      nameEn: `Demon King rule: ${familyId}`,
      nameZh: `魔王篇机制规则：${familyId}`,
      summaryEn:
        "ReferenceOnly synthesis rule binding exact recipe edges and an explicit deterministic ordering boundary.",
      summaryZh: "仅供资料使用的合成规则，绑定精确配方边与显式确定性顺序边界。",
      manifestIds: [
        familyId,
        ...recipe.manifest_record_ids,
        ...recipeInputs.flatMap(({ manifest_record_ids: ids }) => ids),
      ],
      sourceRefs: [...recipe.source_refs, policy],
      tags: ["demon-king", familyId, "mechanic-rule"],
      mechanismQuality: "PolicyBoundary",
    }),
    family_id: familyId,
    rule_order: order,
    trigger_point: "accepted level-up offer resolution",
    state_owner: "battle-local weapon and accessory inventory",
    preconditions: {
      recipe_id: recipe.id,
      input_requirements: recipeInputs.map((input) => ({
        input_id: input.input_id,
        required_level: input.required_level,
      })),
    },
    ordered_operations: [
      "validate every prerequisite in input stable-ID order",
      "emit synthesis candidate before ordinary duplicate upgrade",
      "select candidates by Supreme, Twin, Legendary, then stable recipe ID",
      "consume only CostGearList inputs in source order",
      "install the level-1 output weapon",
    ].map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    fixture_ids: [
      `galactic-baseballer.demon-king.fixture.${familyId}.success`,
      `galactic-baseballer.demon-king.fixture.${familyId}.rejected`,
    ],
    runtime_executable: false,
  };
}

const mechanicRules = [
  synthesisRule("twin-weapon-synthesis", "Twin", 14),
  synthesisRule("supreme-weapon-synthesis", "Supreme", 15),
];

function synthesisFixtures(rule, tier) {
  const recipe = recipes.find(({ tier: recipeTier }) => recipeTier === tier);
  const recipeInputs = inputs.filter(({ recipe_id: id }) => id === recipe.id);
  const policy = policySource("gb.policy.simultaneous-synthesis-order");
  const common = {
    family_id: rule.family_id,
    source_record_ids: [recipe.id, ...recipeInputs.map(({ id }) => id)],
    trigger_point: rule.trigger_point,
    state_owner: rule.state_owner,
    evidence_refs: [
      ...recipe.source_refs.map(({ source_id: id }) => id),
      policy.source_id,
    ],
    runtime_executable: false,
  };
  return [
    {
      ...envelope({
        id: `galactic-baseballer.demon-king.fixture.${rule.family_id}.success`,
        kind: "SemanticReviewFixture",
        nameEn: `${tier} synthesis success`,
        nameZh: `${tier === "Twin" ? "双重" : "至尊"}武器合成成功`,
        summaryEn: "Concrete ReferenceOnly advanced-synthesis success trace.",
        summaryZh: "具体的仅供资料审查高级合成成功轨迹。",
        manifestIds: rule.manifest_record_ids,
        sourceRefs: rule.source_refs,
        tags: ["demon-king", rule.family_id, "semantic-review"],
        mechanismQuality: "PolicyBoundary",
      }),
      ...common,
      preconditions: {
        inventory: recipeInputs.map((input) => ({
          input_id: input.input_id,
          level: input.required_level,
        })),
      },
      input: { accept_recipe_id: recipe.id },
      ordered_operations: rule.ordered_operations,
      expected_facts: {
        output_weapon_id: recipe.output_weapon_id,
        output_level: 1,
        consumed_input_ids: [...recipeInputs]
          .filter(({ consumed }) => consumed)
          .sort((left, right) =>
            left.consumption_order - right.consumption_order)
          .map(({ input_id: id }) => id),
        retained_input_ids: recipeInputs.filter(({ consumed }) => !consumed)
          .map(({ input_id: id }) => id),
      },
    },
    {
      ...envelope({
        id: `galactic-baseballer.demon-king.fixture.${rule.family_id}.rejected`,
        kind: "SemanticReviewFixture",
        nameEn: `${tier} synthesis rejected`,
        nameZh: `${tier === "Twin" ? "双重" : "至尊"}武器合成拒绝`,
        summaryEn:
          "Concrete ReferenceOnly missing-prerequisite trace proving failure invariance.",
        summaryZh: "具体的仅供资料审查缺少前置轨迹，证明失败不变性。",
        manifestIds: rule.manifest_record_ids,
        sourceRefs: rule.source_refs,
        tags: ["demon-king", rule.family_id, "semantic-review"],
        mechanismQuality: "PolicyBoundary",
      }),
      ...common,
      preconditions: {
        inventory: recipeInputs.slice(0, -1).map((input) => ({
          input_id: input.input_id,
          level: input.required_level,
        })),
      },
      input: { request_recipe_id: recipe.id },
      ordered_operations: [
        { operation_order: 0, operation: "validate prerequisites" },
        { operation_order: 1, operation: "detect missing exact input" },
        { operation_order: 2, operation: "reject before consumption" },
      ],
      expected_facts: {
        accepted: false,
        inventory_byte_identical: true,
        output_added: false,
        consumed_input_ids: [],
      },
    },
  ];
}

const reviewFixtures = mechanicRules.flatMap((rule) =>
  synthesisFixtures(
    rule,
    rule.family_id === "twin-weapon-synthesis" ? "Twin" : "Supreme",
  ));
const ruinBotCorrection = releasedCorrections.find(({ id }) =>
  id === "galactic-baseballer.correction.v3_4.ruinbot-level-7-8");
if (ruinBotCorrection === undefined)
  throw new Error("RuinBot released correction boundary missing");
for (const levelNumber of [7, 8]) {
  const level = weaponLevels.find(({ id }) =>
    id === `galactic-baseballer.demon-king.weapon.3113002.level.${levelNumber}`);
  if (level === undefined) throw new Error(`RuinBot level ${levelNumber} missing`);
  reviewFixtures.push({
    ...envelope({
      id:
        `fixture.galactic-baseballer.demon-king.weapon-ruinbot-level-${levelNumber}`,
      kind: "SemanticReviewFixture",
      nameEn: `RuinBot corrected level ${levelNumber}`,
      nameZh: `歼灭机器人修正后等级 ${levelNumber}`,
      summaryEn:
        "Concrete ReferenceOnly review of the retained Version 4.4 post-correction parameter vector.",
      summaryZh: "具体审查 Version 4.4 保留的修正后参数向量，仅供资料使用。",
      manifestIds: level.manifest_record_ids,
      sourceRefs: [...level.source_refs, ...ruinBotCorrection.source_refs],
      tags: ["correction", "demon-king", "ruinbot", "semantic-review"],
    }),
    family_id: "weapon-automatic-action",
    source_record_ids: [level.id, ruinBotCorrection.id],
    trigger_point: "load retained Version 4.4 weapon level",
    state_owner: "immutable weapon-level definition",
    preconditions: {
      weapon_id: "galactic-baseballer.demon-king.weapon.3113002",
      level: levelNumber,
    },
    input: { resolve_binding_key: level.binding_key },
    ordered_operations: [
      { operation_order: 0, operation: "resolve exact GearConfig row" },
      { operation_order: 1, operation: "resolve exact MazeBuff level row" },
      {
        operation_order: 2,
        operation: "retain post-correction parameter vector without reconstructing pre-fix values",
      },
    ],
    expected_facts: {
      maze_buff_id: level.maze_buff_id,
      parameter_values: level.parameter_values,
      correction_id: ruinBotCorrection.id,
      pre_fix_values_modeled: false,
    },
    evidence_refs: [
      ...level.source_refs.map(({ source_id: id }) => id),
      ...ruinBotCorrection.source_refs.map(({ source_id: id }) => id),
    ],
    runtime_executable: false,
  });
}

for (const rows of [mechanicRules, reviewFixtures])
  rows.sort((left, right) => left.id.localeCompare(right.id, "en"));
const outputs = new Map([
  ["demon-arsenal-mechanic-rules.json", mechanicRules],
  ["demon-arsenal-review-fixtures.json", reviewFixtures],
]);
await mkdir(outputRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(outputRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King arsenal fixture drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}

console.log(
  `Demon King arsenal fixtures ${check ? "verified" : "wrote"}: `
  + `${mechanicRules.length} advanced rules/${reviewFixtures.length} fixtures`,
);
