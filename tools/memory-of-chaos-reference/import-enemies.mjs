#!/usr/bin/env node

import { readFile } from "node:fs/promises";
import path from "node:path";
import {
  assert,
  digest,
  manifest,
  normalizedFile,
  record,
  root,
  sourceRecordId,
  structuredRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
const sharedRevision = "60ca52ed98c5c83d867d33bff7f88c69e0b389de";
const sharedFiles = {
  enemy_variants: "content-reference/v4.4/enemy-variants.json",
  enemy_templates: "content-reference/v4.4/enemy-templates.json",
  enemy_abilities: "content-reference/v4.4/enemy-abilities.json",
};
const [sharedVariants, sharedTemplates, sharedAbilities] = await Promise.all([
  readShared(sharedFiles.enemy_variants),
  readShared(sharedFiles.enemy_templates),
  readShared(sharedFiles.enemy_abilities),
]);

async function readShared(relativePath) {
  return JSON.parse(await readFile(path.join(root, relativePath), "utf8"));
}

function inheritedRef(category, manifestRow, selectedRow, note) {
  return {
    id: `starclock:${manifestRow.source_path}:${manifestRow.row_locator}`,
    repository_or_url: "https://github.com/realm-labs/starclock.git",
    revision_or_access_date: sharedRevision,
    game_version: "4.4",
    path_or_page: manifestRow.source_path,
    row_locator: manifestRow.row_locator,
    evidence_sha256: manifestRow.evidence_sha256,
    selected_row_sha256: digest(selectedRow),
    quality: manifestRow.evidence_quality,
    mechanism_quality: "ExactInheritedDefinition",
    note,
  };
}

const templateManifest = manifest.categories.enemy_templates.records;
const abilityManifest = manifest.categories.enemy_abilities.records;
const variantManifest = manifest.categories.enemy_variants.records;
assert(variantManifest.length === 41 && templateManifest.length === 41 && abilityManifest.length === 221,
  "enemy manifest denominator drift");
const templateSourceById = new Map(sharedTemplates.map((row) => [row.id, row]));
const abilitySourceById = new Map(sharedAbilities.map((row) => [row.id, row]));
const variantSourceByMonsterId = new Map(sharedVariants.map((row) => [row.source_monster_id, row]));
assert(templateSourceById.size === sharedTemplates.length, "duplicate shared template id");
assert(abilitySourceById.size === sharedAbilities.length, "duplicate shared ability id");
assert(variantSourceByMonsterId.size === sharedVariants.length, "duplicate shared variant monster id");

const selectedTemplateIds = new Set(templateManifest.map(({ id }) => id));
const variants = variantManifest.map((manifestRow) => {
  const monsterId = manifestRow.id.replace("enemy-variant-", "");
  const definition = variantSourceByMonsterId.get(monsterId);
  assert(definition, `missing inherited enemy variant ${monsterId}`);
  assert(selectedTemplateIds.has(definition.enemy_id), `variant template outside closure ${definition.enemy_id}`);
  const template = templateSourceById.get(definition.enemy_id);
  assert(template, `missing variant template ${definition.enemy_id}`);
  return record({
    id: `enemy-variant.${monsterId}`,
    kind: "EnemyVariant",
    nameEn: `${template.name_en} variant ${monsterId}`,
    nameZh: `${template.name_zh_cn}变体${monsterId}`,
    summaryEn: `Exact encounter variant of ${template.name_en}, including multipliers, weaknesses, resistances, skills, summons and AI overrides.`,
    summaryZh: `${template.name_zh_cn}的精确遭遇变体，包含倍率、弱点、抗性、技能、召唤与AI覆盖。`,
    ownership: "Shared",
    sourceIds: [sourceRecordId("enemy_variants", manifestRow.id)],
    evidence: [
      structuredRef("enemy_variants", manifestRow.id, "Exact reachable MonsterConfig variant identity."),
      inheritedRef("enemy_variants", manifestRow, definition, "Exact Goal 03 normalized variant definition reused without mutation."),
    ],
    tags: ["enemy", "shared", "variant"],
    fields: {
      upstream_monster_id: monsterId,
      enemy_template_id: definition.enemy_id,
      shared_variant_id: definition.id,
      definition,
      source_definition_sha256: digest(definition),
      evidence_quality: definition.quality,
      mechanism_quality: "ExactInheritedDefinition",
      approximations: [],
    },
  });
});

const templates = templateManifest.map((manifestRow) => {
  const definition = templateSourceById.get(manifestRow.id);
  assert(definition, `missing inherited enemy template ${manifestRow.id}`);
  return record({
    id: definition.id,
    kind: "EnemyTemplate",
    nameEn: definition.name_en,
    nameZh: definition.name_zh_cn,
    summaryEn: `Shared ${definition.rank} enemy template with exact base stats, AI program, ordered skills and ability closure.`,
    summaryZh: `共享${definition.rank}敌人模板，包含精确基础属性、AI程序、有序技能与能力闭包。`,
    ownership: "Shared",
    sourceIds: [sourceRecordId("enemy_templates", manifestRow.id)],
    evidence: [inheritedRef("enemy_templates", manifestRow, definition, "Exact Goal 03 normalized template definition reused without mutation.")],
    tags: ["enemy", "shared", "template"],
    fields: {
      definition,
      source_definition_sha256: digest(definition),
      source_template_id: definition.source_template_id,
      rank: definition.rank,
      ai_program_path: definition.source_ai?.path ?? null,
      ai_sequence_source_skill_ids: definition.ai_sequence_source_skill_ids,
      ability_ids: definition.ability_ids,
      evidence_quality: definition.quality,
      mechanism_quality: "ExactInheritedDefinition",
      approximations: [],
    },
  });
});

const abilities = abilityManifest.map((manifestRow) => {
  const definition = abilitySourceById.get(manifestRow.id);
  assert(definition, `missing inherited enemy ability ${manifestRow.id}`);
  assert(selectedTemplateIds.has(definition.enemy_id), `ability owner outside enemy closure ${definition.id}`);
  const displayEn = definition.name_en || `Ability ${definition.source_skill_id}`;
  const displayZh = definition.name_zh_cn || `能力${definition.source_skill_id}`;
  return record({
    id: definition.id,
    kind: "EnemyAbility",
    nameEn: displayEn,
    nameZh: displayZh,
    summaryEn: `Exact shared enemy ability binding for skill ${definition.source_skill_id}, trigger ${definition.trigger_key}.`,
    summaryZh: `共享敌人技能${definition.source_skill_id}、触发键${definition.trigger_key}的精确能力绑定。`,
    ownership: "Shared",
    sourceIds: [sourceRecordId("enemy_abilities", manifestRow.id)],
    evidence: [inheritedRef("enemy_abilities", manifestRow, definition, "Exact Goal 03 normalized ability, status and operation definition reused without mutation.")],
    tags: ["ability", "enemy", "shared"],
    fields: {
      enemy_template_id: definition.enemy_id,
      definition,
      source_definition_sha256: digest(definition),
      source_skill_id: definition.source_skill_id,
      trigger_key: definition.trigger_key,
      attack_type: definition.attack_type,
      damage_type: definition.damage_type,
      use_type: definition.use_type,
      phases: definition.phases,
      operation_types: definition.operation_types,
      modifier_count: definition.modifiers.length,
      status_ref_count: definition.status_refs.length,
      evidence_quality: definition.quality,
      mechanism_quality: definition.mechanism_quality,
      approximations: [],
    },
  });
});

assert(new Set(variants.map(({ enemy_template_id: id }) => id)).size === 41,
  "expected one selected template per selected variant");
for (const template of templates) {
  for (const abilityId of template.ability_ids) {
    assert(abilitySourceById.has(abilityId), `template references missing shared ability ${abilityId}`);
  }
}
const allRecords = [...variants, ...templates, ...abilities];
const claims = allRecords.flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expectedClaims = [
  ...variantManifest.map(({ id }) => sourceRecordId("enemy_variants", id)),
  ...templateManifest.map(({ id }) => sourceRecordId("enemy_templates", id)),
  ...abilityManifest.map(({ id }) => sourceRecordId("enemy_abilities", id)),
].sort();
assert(claims.length === new Set(claims).size, "enemy obligations must be claimed exactly once");
assert(JSON.stringify([...claims].sort()) === JSON.stringify(expectedClaims), "enemy obligation coverage drift");

const variantOutput = normalizedFile("enemy-variants.json", "EnemyVariant", variants);
const templateOutput = normalizedFile("enemy-templates.json", "EnemyTemplate", templates);
const abilityOutput = normalizedFile("enemy-abilities.json", "EnemyAbility", abilities);
await writeCanonical("enemy-variants.json", variantOutput, check);
await writeCanonical("enemy-templates.json", templateOutput, check);
await writeCanonical("enemy-abilities.json", abilityOutput, check);
const summonVariants = variants.filter(({ definition }) => definition.summon_source_ids.length > 0).length;
const aiOverrideVariants = variants.filter(({ definition }) => definition.ai_override !== null).length;
const phasedAbilities = abilities.filter(({ phases }) => phases.length > 0).length;
const statusAbilities = abilities.filter(({ status_ref_count: count }) => count > 0).length;
await writeText(
  "evidence/memory-of-chaos-reference-v1/enemy-closure-audit.md",
  `# Goal 17 enemy closure audit

- Enemy variant obligations: 41/41
- Enemy template obligations: 41/41
- Enemy ability obligations: 221/221
- Total exact-once enemy claims: 303/303
- Variant-to-template cardinality: 41/41
- Variants with explicit summon locators: ${summonVariants}
- Variants with explicit AI overrides: ${aiOverrideVariants}
- Abilities with phase bindings: ${phasedAbilities}
- Abilities with status references: ${statusAbilities}
- Variant digest: \`${digest(variantOutput)}\`
- Template digest: \`${digest(templateOutput)}\`
- Ability digest: \`${digest(abilityOutput)}\`
- Runtime executable rows: 0

Definitions are filtered from the immutable Goal 03 Version 4.4 shared enemy
pack at commit \`${sharedRevision}\`. This Goal adds reachability and provenance
receipts only; it does not edit or reinterpret shared definitions.
`,
  check,
);
console.log(`Goal 17 enemies ${check ? "verified" : "generated"}: 41 variants, 41 templates, 221 abilities.`);
