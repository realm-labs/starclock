#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const check = process.argv.includes("--check");
const phaseArg = process.argv.find((argument) => argument.startsWith("--phase="));
const phase = Number(phaseArg?.slice("--phase=".length) ?? 4);
if (!Number.isInteger(phase) || phase < 1 || phase > 4) throw new Error("--phase must be 1..4");
const root = path.resolve(".");
const configRoot = path.join(root, "config/memory-of-chaos");

const partitions = [
  {
    phase: 1,
    schema: "core.toml",
    files: ["profile.json", "seasons.json", "entries.json", "stages.json", "nodes.json", "tierce.json", "participant-policies.json", "attempt-rules.json"],
  },
  {
    phase: 2,
    schema: "systems.toml",
    files: ["clock-rules.json", "resource-rules.json", "objectives.json", "turbulence.json", "battle-events.json", "rule-contributions.json"],
  },
  {
    phase: 3,
    schema: "content.toml",
    files: ["pool-audits.json", "encounters.json", "waves.json", "enemy-slots.json", "enemy-variants.json", "enemy-templates.json", "enemy-abilities.json"],
  },
  {
    phase: 4,
    schema: "review.toml",
    files: ["sources.json", "reconciliation-receipts.json", "research-gaps.json", "coverage.json", "semantic-fixtures.json", "pack-index.json"],
  },
];
const fileBindings = JSON.parse(await readFile(
  path.join(root, "content-manifests/memory-of-chaos-v1/authoring-contract.json"),
  "utf8",
)).normalized_family_bindings;

function pascal(file) {
  return file.replace(/\.json$/u, "").split(/[^a-zA-Z0-9]+/u)
    .filter(Boolean)
    .map((part) => `${part[0].toUpperCase()}${part.slice(1)}`)
    .join("");
}
function tableName(file) { return `Moc${pascal(file)}`; }
function sheetName(file) { return pascal(file).slice(0, 31); }
function field(name, type, constraints = []) {
  return ["[[tables.fields]]", `name = ${JSON.stringify(name)}`, `type = ${JSON.stringify(type)}`, ...constraints].join("\n");
}
const commonFields = [
  field("id", "i32", ["range = [1, 2147483647]"]),
  field("stable_key", "string", ["length = [1, 1200]"]),
  field("row_order", "i32", ["range = [1, 1000000]"]),
  field("name_en", "string", ["length = [1, 12000]"]),
  field("name_zh_cn", "string", ["length = [1, 12000]"]),
  field("summary_en", "string", ["length = [1, 24000]"]),
  field("summary_zh_cn", "string", ["length = [1, 24000]"]),
  field("ownership", "enum<MocOwnership>"),
  field("coverage_state", "enum<MocCoverageState>"),
  field("evidence_quality", "string", ["length = [1, 400]"]),
  field("mechanism_quality", "string", ["length = [1, 400]"]),
  field("manifest_record_ids", "optional<list<string>>", ['parser = { kind = "split", separator = "|" }', "length = [0, 2048]"]),
  field("source_ref_ids", "optional<list<string>>", ['parser = { kind = "split", separator = "|" }', "length = [0, 4096]"]),
  field("tags", "optional<list<string>>", ['parser = { kind = "split", separator = "|" }', "length = [0, 512]"]),
  field("payload_json", "string", ["length = [2, 32767]"]),
  field("runtime_executable", "bool"),
];
const relationFields = {
  "seasons.json": [field("profile_id", "ref<MocProfile.id>")],
  "entries.json": [field("profile_id", "ref<MocProfile.id>")],
  "stages.json": [field("profile_id", "ref<MocProfile.id>")],
  "nodes.json": [field("profile_id", "ref<MocProfile.id>"), field("stage_id", "ref<MocStages.id>")],
  "tierce.json": [field("profile_id", "ref<MocProfile.id>")],
  "participant-policies.json": [field("profile_id", "ref<MocProfile.id>")],
  "attempt-rules.json": [field("profile_id", "ref<MocProfile.id>")],
  "waves.json": [field("profile_id", "ref<MocProfile.id>"), field("encounter_id", "ref<MocEncounters.id>")],
  "enemy-slots.json": [field("profile_id", "ref<MocProfile.id>"), field("wave_id", "ref<MocWaves.id>"), field("enemy_variant_id", "ref<MocEnemyVariants.id>")],
  "enemy-variants.json": [field("profile_id", "ref<MocProfile.id>"), field("enemy_template_id", "ref<MocEnemyTemplates.id>")],
  "enemy-abilities.json": [field("profile_id", "ref<MocProfile.id>"), field("enemy_template_id", "ref<MocEnemyTemplates.id>")],
};

function renderTable(file) {
  const workbook = fileBindings[file]?.workbook;
  if (!workbook) throw new Error(`missing workbook binding ${file}`);
  const extras = relationFields[file] ?? (file === "profile.json" ? [] : [field("profile_id", "ref<MocProfile.id>")]);
  return [
    "[[tables]]",
    `name = ${JSON.stringify(tableName(file))}`,
    'mode = "map"',
    'key = "id"',
    "[tables.source]",
    'format = "xlsx"',
    `file = ${JSON.stringify(workbook)}`,
    `sheet = ${JSON.stringify(sheetName(file))}`,
    ...commonFields,
    ...extras,
    "[[tables.indexes]]",
    'name = "by_stable_key"',
    'fields = ["stable_key"]',
    "unique = true",
  ].join("\n");
}
for (const partition of partitions.filter((entry) => entry.phase <= phase)) {
  const header = partition.phase === 1
    ? `[[enums]]
name = "MocOwnership"
values = ["MemoryOfChaos", "Shared"]

[[enums]]
name = "MocCoverageState"
values = ["DataReady"]

`
    : "";
  const bytes = `# @generated by tools/memory-of-chaos-reference/generate-sora-schema.mjs; do not edit.\n\n${header}${partition.files.map(renderTable).join("\n\n")}\n`;
  await output(`schema/${partition.schema}`, bytes);
}
const included = partitions.filter((entry) => entry.phase <= phase).map(({ schema }) => schema);
const project = `# @generated by tools/memory-of-chaos-reference/generate-sora-schema.mjs; do not edit.
package = "starclock_memory_of_chaos_reference_config"
includes = [
${included.map((schema) => `  "schema/${schema}",`).join("\n")}
]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "../memory-of-chaos-generated/schema.lock"
excel_templates = "../memory-of-chaos-generated/templates"

[[build.codegen]]
target = "rust"
out = "../memory-of-chaos-generated/readers/rust"
format = "never"

[[build.exports]]
format = "binary"
out = "../memory-of-chaos-generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "../memory-of-chaos-generated/debug-json"

[codegen.rust]
runtime_format = "sora"
`;
await output("project.toml", project);
console.log(`Goal 17 Sora schema ${check ? "verified" : "generated"}: phase ${phase}, ${partitions.filter((entry) => entry.phase <= phase).flatMap(({ files }) => files).length} tables.`);

async function output(relative, bytes) {
  const destination = path.join(configRoot, relative);
  if (check) {
    const existing = await readFile(destination, "utf8").catch(() => "");
    if (existing !== bytes) throw new Error(`${relative} generation drift`);
  } else {
    await mkdir(path.dirname(destination), { recursive: true });
    await writeFile(destination, bytes);
  }
}
