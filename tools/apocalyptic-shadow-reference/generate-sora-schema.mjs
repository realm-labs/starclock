#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { root } from "./source.mjs";

const check = process.argv.includes("--check");
const groups = {
  system: [
    "profiles", "periods", "stages", "nodes", "participant-policies",
    "team-slots", "loadout-records", "attempts", "transitions", "clocks",
    "boss-progress", "scores", "objectives", "stars", "safeguards",
    "axioms", "embers", "buffs", "mechanic-contributions",
  ],
  bindings: [
    "pool-audits", "encounters", "encounter-waves", "enemy-slots", "enemies",
    "enemy-skills", "enemy-statuses", "ability-bindings",
  ],
  review: [
    "mechanic-rules", "sources", "reconciliation", "research-gaps", "coverage",
    "review-fixtures", "manifest", "pack-index",
  ],
};
const workbook = {
  system: "ApocalypticShadow.xlsx",
  bindings: "ApocalypticShadowBindings.xlsx",
  review: "ApocalypticShadowReview.xlsx",
};
function pascal(value) {
  return value.split("-").map((part) => part[0].toUpperCase() + part.slice(1)).join("");
}
function field(name, type, constraint = "") {
  return ["[[tables.fields]]", `name = ${JSON.stringify(name)}`,
    `type = ${JSON.stringify(type)}`, constraint].filter(Boolean).join("\n");
}
const common = [
  field("id", "i32", "range = [1, 2147483647]"),
  field("stable_key", "string", "length = [1, 1200]"),
  field("row_order", "i32", "range = [1, 1000000]"),
  field("name_en", "string", "length = [1, 2400]"),
  field("name_zh_cn", "string", "length = [1, 2400]"),
  field("summary_en", "string", "length = [1, 6000]"),
  field("summary_zh_cn", "string", "length = [1, 6000]"),
  field("ownership", "enum<ApsOwnership>"),
  field("coverage_state", "enum<ApsCoverageState>"),
  field("evidence_quality", "enum<ApsEvidenceQuality>"),
  field("mechanism_quality", "enum<ApsMechanismQuality>"),
  field("manifest_record_ids", "list<string>",
    'parser = { kind = "split", separator = "|" }\nlength = [1, 4096]'),
  field("source_ref_ids", "list<string>",
    'parser = { kind = "split", separator = "|" }\nlength = [1, 4096]'),
  field("payload_json", "string", "length = [2, 1000000]"),
  field("runtime_executable", "bool"),
];
const enums = `[[enums]]
name = "ApsOwnership"
values = ["ApocalypticShadow", "Shared"]

[[enums]]
name = "ApsCoverageState"
values = ["DataReady", "ResearchGap", "Excluded"]

[[enums]]
name = "ApsEvidenceQuality"
values = ["ExactStructured", "ExactPublicText", "Observed", "ApproximateFromReleasedText", "ProjectPolicy"]

[[enums]]
name = "ApsMechanismQuality"
values = ["ExactRelationship", "ExactIdentity", "ReleasedTextBoundary", "ObservationBoundary", "PolicyBoundary"]
`;
for (const [group, files] of Object.entries(groups)) {
  const tables = files.map((file) => [
    "[[tables]]", `name = "Aps${pascal(file)}"`, 'mode = "map"', 'key = "id"',
    "[tables.source]", 'format = "xlsx"', `file = "${workbook[group]}"`,
    `sheet = "${pascal(file)}"`, ...common,
    "[[tables.indexes]]", 'name = "by_stable_key"',
    'fields = ["stable_key"]', "unique = true",
  ].join("\n")).join("\n\n");
  const bytes = `${group === "system" ? enums : ""}\n${tables}\n`;
  const output = path.join(root, `config/apocalyptic-shadow/schema/${group}.toml`);
  if (check) {
    if (await readFile(output, "utf8").catch(() => "") !== bytes)
      throw new Error(`${group}.toml generation drift`);
  } else {
    await mkdir(path.dirname(output), { recursive: true });
    await writeFile(output, bytes);
  }
}
const project = `package = "starclock_apocalyptic_shadow_reference_config"
includes = [
  "schema/system.toml",
  "schema/bindings.toml",
  "schema/review.toml",
]

[build]
default_source_format = "xlsx"
data_root = "data"
schema_lock = "../apocalyptic-shadow-generated/schema.lock"
excel_templates = "../apocalyptic-shadow-generated/templates"

[[build.codegen]]
target = "rust"
out = "../apocalyptic-shadow-generated/readers/rust"
format = "never"

[[build.exports]]
format = "binary"
out = "../apocalyptic-shadow-generated/config.sora"

[[build.exports]]
format = "json-debug"
out = "../apocalyptic-shadow-generated/debug-json"

[codegen.rust]
runtime_format = "sora"
`;
const projectPath = path.join(root, "config/apocalyptic-shadow/project.toml");
if (check) {
  if (await readFile(projectPath, "utf8").catch(() => "") !== project)
    throw new Error("project.toml generation drift");
} else {
  await mkdir(path.dirname(projectPath), { recursive: true });
  await writeFile(projectPath, project);
}
console.log("Apocalyptic Shadow Sora schema: 35 isolated tables.");
