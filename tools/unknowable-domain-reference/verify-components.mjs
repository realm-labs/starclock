#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(
  process.execPath,
  ["tools/unknowable-domain-reference/import-components.mjs", "--check"],
  { cwd: root, stdio: "inherit" },
);
const rows = json("content-reference/unknowable-domain-v1/components.json");
const levels = json(
  "content-reference/unknowable-domain-v1/component-levels.json",
);
const compatibility = json(
  "content-reference/unknowable-domain-v1/component-slot-compatibility.json",
);
const definitions = rows.filter(({ kind }) => kind === "Component");
const categories = rows.filter(({ kind }) => kind === "ComponentCategory");
const types = rows.filter(({ kind }) => kind === "ComponentType");
assert(definitions.length === 109, "Component denominator drift");
assert(levels.length === 277, "Component-level denominator drift");
assert(categories.length === 2, "Component-category denominator drift");
assert(types.length === 3, "Component-type denominator drift");
assert(compatibility.length === 346, "Component compatibility denominator drift");
const allRows = [...rows, ...levels, ...compatibility];
assert(unique(allRows.map(({ id }) => id)), "duplicate Component stable ID");

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
  assert(JSON.stringify(row.tags) === JSON.stringify([...row.tags].sort()),
    `${row.id} tags are not canonical`);
}

assert(exactOnce(categories.map(({ source_id: id }) => id), ["Common", "Ultra"]),
  "Component categories drift");
assert(exactOnce(types.map(({ source_id: id }) => id),
  ["Active", "Attach", "Passive"]),
"Component types drift");
assert(definitions.filter(({ category }) => category === "Common").length === 84
  && definitions.filter(({ category }) => category === "Ultra").length === 25,
"Component category split drift");
assert(definitions.filter(({ component_type: type }) => type === "Active")
  .length === 24
  && definitions.filter(({ component_type: type }) => type === "Attach")
    .length === 35
  && definitions.filter(({ component_type: type }) => type === "Passive")
    .length === 50,
"Component type split drift");
assert(definitions.every(({ style_ids: ids, style_resolution: resolution }) =>
  ids.length === 0 && resolution === "Unspecified"),
"Component definition inferred an unavailable style binding");

const levelIds = new Set(levels.map(({ id }) => id));
for (const definition of definitions) {
  assert(definition.level_ids.length ===
    (definition.category === "Ultra" ? 1 : 3),
  `${definition.id} level cardinality drift`);
  assert(definition.level_ids.every((id) => levelIds.has(id)),
    `${definition.id} level reference does not resolve`);
}
const abilityBindings = new Set();
for (const row of levels) {
  assert(["1", "2", "3"].includes(row.level), `${row.id} invalid level`);
  assert(row.shape === row.component_type
    && row.shape_basis === "MagicUnitType",
  `${row.id} shape basis drift`);
  assert(row.range_ids.length > 0 && row.effect_types.length > 0,
    `${row.id} lacks range/effect selectors`);
  assert(row.style_ids.length === 0 && row.style_resolution === "Unspecified",
    `${row.id} inferred an unavailable style binding`);
  const program = row.effect_program;
  assert(program.binding_type === "StageAbilityBeforeCharacterBorn"
    && program.binding_key !== ""
    && program.ability_path.startsWith(
      "Config/ConfigAbility/Level/Level_RogueMagic_Ability_")
    && program.operation_resolution === "SourceProgramPreservedNotLowered"
    && program.parameter_values.every((value) =>
      /^(0|-?(?:[1-9][0-9]*(?:\.[0-9]*[1-9])?|0\.[0-9]*[1-9]))$/u
        .test(value)),
  `${row.id} effect program drift`);
  abilityBindings.add(program.binding_key);
}
assert(abilityBindings.size === 109,
  "Component ability binding exactness drift");

const compatByLevel = Map.groupBy(
  compatibility,
  ({ component_level_id: id }) => id,
);
assert(compatByLevel.size === 277
  && [...levelIds].every((id) => compatByLevel.has(id)),
"Component compatibility does not cover every level");
assert(compatibility.every(({ ordinal, eligibility,
  slot_layout_resolution: resolution }) =>
  Number.isSafeInteger(ordinal) && ordinal >= 0
    && eligibility === "SourceCompatible"
    && resolution === "DeferredToLoadoutValidation"),
"Component compatibility overclaims loadout validity");
const rangeCounts = Object.fromEntries(
  [...Map.groupBy(compatibility, ({ range }) => range).entries()]
    .map(([range, values]) => [range, values.length]),
);
assert(rangeCounts.None === 235
  && rangeCounts.Eject === 33
  && rangeCounts.AOE === 33
  && rangeCounts.Spread === 36
  && rangeCounts.Concentrate === 9
  && Object.keys(rangeCounts).length === 5,
"Component range denominator drift");

const manifest = json(
  "content-manifests/unknowable-domain-v1/content-manifest.json",
);
assert(exactOnce(
  definitions.map(({ source_id: id }) => id),
  manifest.categories.components.records.map(({ id }) => id),
), "Component manifest closure drift");
assert(exactOnce(
  levels.map(({ source_id: id }) => id),
  manifest.categories.component_levels.records.map(({ id }) => id),
), "Component-level manifest closure drift");
assert(exactOnce(
  levels.map(({ effect_source_id: id }) => id),
  manifest.categories.component_effects.records.map(({ id }) => id),
), "Component-effect manifest closure drift");
assert(exactOnce(
  categories.map(({ source_id: id }) => id),
  manifest.categories.component_categories.records.map(({ id }) => id),
), "Component-category manifest closure drift");
assert(exactOnce(
  types.map(({ source_id: id }) => id),
  manifest.categories.component_types.records.map(({ id }) => id),
), "Component-type manifest closure drift");

console.log(
  "Unknowable Domain Components verified (109 definitions; 277 levels and " +
  "effect programs; 2 categories; 3 types; 346 exact range/slot " +
  "compatibilities; style binding remains fail-closed).",
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
