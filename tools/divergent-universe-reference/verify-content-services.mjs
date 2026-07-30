#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sourceRoot = path.resolve(
  valueAfter("--source-cache")
    ?? path.join(root, ".cache/content-reference/turnbasedgamedata"),
);
execFileSync(process.execPath, [
  "tools/divergent-universe-reference/import-content-services.mjs",
  "--check",
  "--root",
  root,
  "--source-cache",
  sourceRoot,
], { cwd: root, stdio: "inherit" });

const outputRoot = path.join(root, "content-reference/divergent-universe-v1");
const services = json(path.join(outputRoot, "mode-service-npcs.json"));
const adventures = json(path.join(outputRoot, "adventure-outcomes.json"));
const manifest = json(path.join(
  root,
  "content-manifests/divergent-universe-v1/content-manifest.json",
));
assert(services.length === 23, "mode service NPC count drift");
assert(adventures.length === 32, "Adventure outcome count drift");
assert(exactOnce(
  services.map((row) => row.source_id),
  manifest.categories.mode_service_npcs.records.map((row) => row.id),
), "mode service NPC exact-once drift");
assert(exactOnce(
  adventures.map((row) => row.source_id),
  manifest.categories.adventure_outcomes.records.map((row) => row.id),
), "Adventure outcome exact-once drift");
assert(services.every((row) =>
  row.coverage_state === "Researched"
    && row.evidence_quality === "ProjectPolicy"
    && row.graph_resolution === "MissingAtPinnedRevision"
    && row.service_kind === "UnclassifiedMissingGraph"
    && row.choice_ids.length === 0
    && row.fallback === "RejectWithoutMutation"
    && row.runtime_lowered === false),
"mode service NPC missing-graph boundary drift");
for (const service of services) {
  const result = spawnSync("git", [
    "cat-file",
    "-e",
    `fd978d6ef09f941fba644c731ab54abd6f7c3568:${service.graph_path}`,
  ], { cwd: sourceRoot, stdio: "ignore" });
  assert(result.status !== 0,
    `${service.id} graph unexpectedly exists at the fixed revision`);
}

assert(adventures.filter((row) =>
  row.coverage_state === "DataReady").length === 26,
"resolved Adventure outcome count drift");
assert(adventures.filter((row) =>
  row.coverage_state === "Researched").length === 6,
"unresolved Adventure outcome count drift");
assert(adventures.filter((row) =>
  row.adventure_type === "RogueWolfGun").every((row) =>
  row.parameter_program.length === 0
    && row.abstract_outcome.kind === "ExternalAdventureResult"),
"Wolf Gun missing parameter boundary drift");
assert(adventures.filter((row) =>
  row.adventure_type !== "RogueWolfGun").every((row) =>
  row.parameter_program.length > 0
    && row.abstract_outcome.input === "AcceptedExternalAdventureResult"),
"Adventure parameter/abstract-result closure drift");
assert(adventures.every((row) =>
  row.action_gameplay === "Excluded"
    && row.fallback === "RejectWithoutMutation"
    && row.runtime_lowered === false),
"Adventure exclusion/runtime boundary drift");
assert(JSON.stringify(Object.fromEntries(
  [...Map.groupBy(adventures, (row) => row.adventure_type)]
    .map(([type, values]) => [type, values.length]).sort(),
)) === JSON.stringify({
  RogueCandyCrash: 4,
  RogueCaptureMonster: 9,
  RogueDestroyProp: 8,
  RogueEscapeLaser: 3,
  RogueTurntable: 2,
  RogueWolfGun: 6,
}), "Adventure type distribution drift");

const digest = crypto.createHash("sha256");
for (const file of ["mode-service-npcs.json", "adventure-outcomes.json"])
  digest.update(fs.readFileSync(path.join(outputRoot, file)));
console.log(
  `Divergent Universe content services verified (23 missing-graph NPCs; ` +
  `26 resolved and 6 fail-closed Adventure outcomes; digest ` +
  `${digest.digest("hex")}).`,
);

function valueAfter(flag) {
  const index = args.indexOf(flag);
  if (index < 0) return undefined;
  if (!args[index + 1] || args[index + 1].startsWith("--"))
    throw new Error(`${flag} requires a value`);
  return args[index + 1];
}

function json(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function exactOnce(left, right) {
  return JSON.stringify([...left].sort())
    === JSON.stringify([...right].sort());
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
