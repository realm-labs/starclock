#!/usr/bin/env node

import fs from "node:fs";
import os from "node:os";
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
const systems =
  path.join(root, "config/divergent-universe/schema/systems.toml");
const content =
  path.join(root, "config/divergent-universe/schema/content.toml");
const evidence =
  path.join(root, "config/divergent-universe/schema/evidence.toml");
const projectText = fs.readFileSync(project, "utf8");
const coreText = fs.readFileSync(core, "utf8");
const systemsText = fs.readFileSync(systems, "utf8");
const contentText = fs.readFileSync(content, "utf8");
const evidenceText = fs.readFileSync(evidence, "utf8");
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
assert((systemsText.match(/\[\[tables\]\]/gu) ?? []).length === 26,
  "P3-B2 system table denominator drift");
assert((contentText.match(/\[\[tables\]\]/gu) ?? []).length === 28,
  "P3-B3 content table denominator drift");
assert((evidenceText.match(/\[\[tables\]\]/gu) ?? []).length === 8,
  "P3-B4 evidence table denominator drift");
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
for (const table of [
  "DivergentUniverseEquations",
  "DivergentUniverseEquationRecipes",
  "DivergentUniverseBlessings",
  "DivergentUniverseBlessingLevels",
  "DivergentUniverseCurios",
  "DivergentUniverseCurioStates",
  "DivergentUniverseGrandMiracles",
  "DivergentUniverseTitanTypes",
  "DivergentUniverseTitanBoons",
  "DivergentUniverseTitanContributions",
])
  assert(systemsText.includes(`name = "${table}"`), `missing table ${table}`);
for (const typedReference of [
  "optional<ref<DivergentUniverseEquationRecipes.id>>",
  "optional<ref<DivergentUniverseEquations.id>>",
  "optional<ref<DivergentUniverseBlessings.id>>",
  "optional<list<ref<DivergentUniverseEquations.id>>>",
  "optional<ref<DivergentUniverseCurios.id>>",
  "optional<ref<DivergentUniverseGrandMiracles.id>>",
  "optional<ref<DivergentUniverseTitanTypes.id>>",
  "optional<list<ref<DivergentUniverseTitanBoons.id>>>",
])
  assert(systemsText.includes(typedReference),
    `missing typed system reference ${typedReference}`);
for (const table of [
  "DivergentUniverseWorkbenches",
  "DivergentUniverseServiceRules",
  "DivergentUniversePermanentTalents",
  "DivergentUniverseWeeklyModifiers",
  "DivergentUniverseOccurrences",
  "DivergentUniverseOccurrenceVariants",
  "DivergentUniverseAdventureOutcomes",
  "DivergentUniverseEncounterGroups",
  "DivergentUniverseEncounterWaves",
  "DivergentUniverseEnemySlots",
  "DivergentUniverseBossPools",
  "DivergentUniverseMechanicRules",
])
  assert(contentText.includes(`name = "${table}"`), `missing table ${table}`);
for (const typedReference of [
  "optional<list<ref<DivergentUniverseWorkbenchFunctions.id>>>",
  "optional<ref<DivergentUniverseCurrencies.id>>",
  "optional<list<ref<DivergentUniversePermanentTalents.id>>>",
  "optional<ref<DivergentUniverseFinishConditions.id>>",
  "optional<ref<DivergentUniverseCurioStates.id>>",
  "optional<list<ref<DivergentUniverseOccurrenceVariants.id>>>",
  "optional<list<ref<DivergentUniverseEnemySlots.id>>>",
  "optional<ref<DivergentUniverseEncounterWaves.id>>",
  "optional<ref<DivergentUniverseEncounterGroups.id>>",
  "optional<ref<DivergentUniverseMechanicSourceFiles.id>>",
])
  assert(contentText.includes(typedReference),
    `missing typed content reference ${typedReference}`);
for (const table of [
  "DivergentUniverseSources",
  "DivergentUniverseCoverage",
  "DivergentUniverseResearchGaps",
  "DivergentUniverseSemanticFixtureFamilies",
  "DivergentUniverseReviewFixtures",
  "DivergentUniverseReconciliationReceipts",
  "DivergentUniverseManifest",
  "DivergentUniversePackIndex",
])
  assert(evidenceText.includes(`name = "${table}"`), `missing table ${table}`);
const allSchemaText = [coreText, systemsText, contentText, evidenceText]
  .join("\n");
assert(allSchemaText.includes(
  "optional<list<ref<DivergentUniverseSources.id>>>",
), "common source refs are not typed");
for (const typedReference of [
  "optional<list<ref<DivergentUniverseReviewFixtures.id>>>",
  "optional<list<ref<DivergentUniverseResearchGaps.id>>>",
  "optional<ref<DivergentUniverseSemanticFixtureFamilies.id>>",
])
  assert(allSchemaText.includes(typedReference),
    `missing typed evidence reference ${typedReference}`);

const sora = locateSora();
assert(execFileSync(sora, ["--version"], { encoding: "utf8" }).trim()
  === "sora 0.3.0", "wrong Sora CLI version");
execFileSync(sora, [
  "--serial",
  "check",
  "--project",
  project,
], { cwd: root, stdio: "inherit" });
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/generate-sora-artifacts.mjs",
  root,
], { cwd: root, stdio: "inherit" });
const generated = path.join(root, "config/divergent-universe-generated");
const lock = path.join(generated, "schema.lock");
const parsedLock = JSON.parse(fs.readFileSync(lock, "utf8")).schema;
assert(parsedLock.package
  === "starclock_divergent_universe_reference_config",
"generated schema-lock package drift");
assert(parsedLock.tables.length === 80, "generated schema-lock table drift");
const templates = fs.readdirSync(path.join(generated, "templates")).sort();
assert(JSON.stringify(templates) === JSON.stringify([
  "DivergentUniverse.xlsx",
  "DivergentUniverseBindings.xlsx",
  "DivergentUniverseReview.xlsx",
]), "isolated Excel template set drift");
const readerFiles = fs.readdirSync(path.join(generated, "reader"))
  .filter((file) => file.endsWith(".rs")).sort();
assert(readerFiles.length === 85, "generated Rust reader file count drift");

const temporary = fs.mkdtempSync(
  path.join(os.tmpdir(), "starclock-divergent-universe-sora-"),
);
try {
  execFileSync(sora, [
    "--serial", "schema-lock", "--project", projectText
      ? path.join(root, "config/divergent-universe/project.toml")
      : "", "--out", path.join(temporary, "schema.lock"),
  ], { cwd: root, stdio: "inherit" });
  assert(fs.readFileSync(lock).equals(
    fs.readFileSync(path.join(temporary, "schema.lock")),
  ), "committed schema lock is not deterministic");
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}
console.log(
  "Divergent Universe P3-B4 Sora schema verified (80 isolated tables; typed " +
  "source/evidence references; deterministic lock, three templates and 85 " +
  "Rust reader files; Sora 0.3.0).",
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
