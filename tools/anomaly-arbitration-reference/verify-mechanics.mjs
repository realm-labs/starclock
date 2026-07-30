#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const outputRoot = path.join(
  root,
  "content-reference/anomaly-arbitration-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function compare(left, right) {
  return left.localeCompare(right);
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/import-mechanics.mjs"),
  "--check",
], { stdio: "inherit" });

const files = {};
for (const name of [
  "traits.json",
  "maze-buff-bindings.json",
  "mechanic-contributions.json",
]) {
  const encoded = await readFile(path.join(outputRoot, name));
  files[name] = { encoded, document: JSON.parse(encoded) };
}

const expectedKinds = {
  "traits.json": ["EnemyTrait", 8],
  "maze-buff-bindings.json": ["MazeBuffBinding", 14],
  "mechanic-contributions.json": ["MechanicContribution", 87],
};
const mechanismQualities = new Set(schema.common_envelope
  .mechanism_quality.enum);
for (const [name, { document }] of Object.entries(files)) {
  const [kind, count] = expectedKinds[name];
  assert(
    document.schema_revision
      === "starclock.anomaly-arbitration-normalized-file.v1"
      && document.goal_id === "anomaly-arbitration-reference-v1"
      && document.profile === "anomaly-arbitration-v1"
      && document.file === name
      && document.record_kind === kind
      && document.records.length === count,
    `${name} envelope/count drift`,
  );
  for (const record of document.records) {
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined, `${record.id} lacks ${field}`);
    assert(record.kind === kind
      && record.coverage_state === "DataReady"
      && record.runtime_executable === false
      && record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn
      && mechanismQualities.has(record.mechanism_quality),
    `${record.id} normalized boundary drift`);
    assert(JSON.stringify(record.manifest_record_ids)
      === JSON.stringify([...record.manifest_record_ids].sort(compare)),
    `${record.id} manifest order drift`);
    for (const source of record.source_refs) {
      for (const field of schema.types.source_ref.required_fields)
        assert(source[field] !== undefined && source[field] !== "",
          `${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256),
        `${record.id} source digest drift`);
    }
  }
}

const traits = files["traits.json"].document.records;
const expectedTraits = [
  [3033023, ["6"], "ChallengePeakBattle_BaseAbility_0008",
    ["stage.knight-1"]],
  [3033038, ["0.5", "0.5", "2"],
    "ChallengePeakBattle_BaseAbility_0013", ["stage.knight-2"]],
  [3033051, ["1", "0.2", "1", "0.15"],
    "ChallengePeakBattle_BaseAbility_0016", ["stage.king-normal"]],
  [3033052, ["2", "0.2", "1", "0.15"],
    "ChallengePeakBattle_EnhancedAbility_0016", ["stage.king-plight"]],
  [3033058, ["500"], "ChallengePeakBattle_BaseAbility_0018",
    ["stage.knight-3"]],
  [3033063, ["0.02", "0.04", "10", "5"],
    "ChallengePeakBattle_BaseAbility_0019", ["stage.knight-2"]],
  [3033069, ["0.3", "4"], "ChallengePeakBattle_BaseAbility_0020",
    ["stage.king-normal"]],
  [3033070, ["0.5", "4"], "ChallengePeakBattle_EnhancedAbility_0020",
    ["stage.king-plight"]],
];
for (const [index, record] of traits.entries()) {
  const [id, parameters, binding, stages] = expectedTraits[index];
  assert(record.source_numeric_id === id
    && JSON.stringify(record.source_parameters) === JSON.stringify(parameters)
    && record.in_battle_binding_key === binding
    && JSON.stringify(record.stage_ids) === JSON.stringify(stages)
    && record.binding_program_state === "ResolvedInExtractedAbilityList",
  `${record.id} trait relationship drift`);
}

const bindings = files["maze-buff-bindings.json"].document.records;
const sortedBindings = [...bindings].sort((left, right) =>
  left.stage_id.localeCompare(right.stage_id)
    || left.binding_order - right.binding_order
    || left.id.localeCompare(right.id));
assert(JSON.stringify(bindings) === JSON.stringify(sortedBindings),
  "MazeBuff binding ordering drift");
assert(bindings.filter(({ source_role }) => source_role === "EnemyTrait")
  .length === 8, "trait binding count drift");
assert(bindings.filter(({ source_role }) => source_role === "QuadrantOption")
  .length === 6, "Quadrant binding count drift");
assert(bindings.filter(({ binding_program_state }) =>
  binding_program_state
    === "NamedInLayoutButMissingFromExtractedAbilityList").length === 4,
"unavailable plugin binding count drift");

const contributions =
  files["mechanic-contributions.json"].document.records;
const sortedContributions = [...contributions].sort((left, right) =>
  left.scope.localeCompare(right.scope)
    || left.install_order - right.install_order
    || left.id.localeCompare(right.id));
assert(JSON.stringify(contributions) === JSON.stringify(sortedContributions),
  "mechanic contribution ordering drift");
const categoryNames = [
  "stage_traits",
  "quadrant_options",
  "battle_events",
  "config_programs",
];
const expectedManifestIds = categoryNames.flatMap((category) =>
  manifest.categories[category].records.map(({ id }) => `${category}:${id}`))
  .sort(compare);
const actualManifestIds = contributions.flatMap(
  ({ manifest_record_ids: ids }) => ids,
).sort(compare);
assert(JSON.stringify(actualManifestIds)
  === JSON.stringify(expectedManifestIds),
"mechanic contribution exact-once manifest drift");
const programs = contributions.filter(({ source_path }) => source_path);
assert(programs.length === 73
  && new Set(programs.map(({ source_path }) => source_path)).size === 73
  && programs.every(({ program_body_imported, runtime_executable }) =>
    program_body_imported === false && runtime_executable === false),
"configuration-program boundary drift");

console.log(
  "Anomaly Arbitration mechanics verified: "
    + Object.entries(files).map(([name, { encoded }]) =>
      `${name}=${createHash("sha256").update(encoded).digest("hex")}`)
      .join(", "),
);
