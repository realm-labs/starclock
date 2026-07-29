#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/generate-sora-schema.mjs",
  root,
], { cwd: root, stdio: "inherit" });

const project =
  path.join(root, "config/divergent-universe/project.toml");
const core = path.join(root, "config/divergent-universe/schema/core.toml");
const projectText = fs.readFileSync(project, "utf8");
const coreText = fs.readFileSync(core, "utf8");
assert(projectText.includes(
  'package = "starclock_divergent_universe_reference_config"',
), "isolated project package drift");
assert(projectText.includes(
  'schema_lock = "../divergent-universe-generated/schema.lock"',
), "isolated schema-lock path drift");
assert(!projectText.includes("config/generated")
  && !projectText.includes("unknowable-domain"),
"shared/other-mode generated path leak");
assert((coreText.match(/\[\[tables\]\]/gu) ?? []).length === 18,
  "P3-B1 core table denominator drift");
for (const table of [
  "DivergentUniverseProfiles",
  "DivergentUniverseModules",
  "DivergentUniverseEntries",
  "DivergentUniverseAreas",
  "DivergentUniverseDifficulties",
  "DivergentUniverseLayers",
  "DivergentUniverseStageFlow",
  "DivergentUniverseProtocols",
  "DivergentUniverseArithmeticMappingEligibility",
  "DivergentUniverseArithmeticMappingBuilds",
  "DivergentUniverseArithmeticMappingRules",
])
  assert(coreText.includes(`name = "${table}"`), `missing table ${table}`);
for (const typedReference of [
  "optional<ref<DivergentUniverseModules.id>>",
  "optional<list<ref<DivergentUniverseEntries.id>>>",
  "optional<list<ref<DivergentUniverseDifficulties.id>>>",
  "optional<list<ref<DivergentUniverseLayers.id>>>",
])
  assert(coreText.includes(typedReference),
    `missing typed reference ${typedReference}`);

const sora = locateSora();
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === "sora 0.3.0", "wrong Sora CLI version");
execFileSync(sora, [
  "--serial",
  "check",
  "--project",
  project,
], { cwd: root, stdio: "inherit" });
console.log(
  "Divergent Universe P3-B1 Sora schema verified (18 isolated core, flow, " +
  "protocol and Arithmetic Mapping tables; typed references; Sora 0.3.0).",
);

function locateSora() {
  const policy = JSON.parse(fs.readFileSync(
    path.join(root, "policy/sora-toolchain.json"),
    "utf8",
  ));
  const candidates = [
    path.join(root, policy.install_root, "bin/sora"),
    path.join(
      "/Users/mikai/CLionProjects/starclock",
      policy.install_root,
      "bin/sora",
    ),
  ];
  const result = candidates.find((candidate) => fs.existsSync(candidate));
  if (!result) throw new Error("Sora 0.3.0 executable is unavailable");
  return result;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
