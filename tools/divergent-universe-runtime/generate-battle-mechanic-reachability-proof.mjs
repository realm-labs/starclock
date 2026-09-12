#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
const output = "content-manifests/divergent-universe-runtime-v1/battle-mechanic-reachability-proof.json";
const sourceCache = sourceCacheArgument();
const sources = [
  "Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S1.json",
  "Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.json",
  "Config/ConfigAbility/Level/Level_RogueBuff_Ability_Tourn1.json",
];

export function buildBattleMechanicReachabilityProof() {
  assert(execFileSync("git", ["-C", sourceCache, "rev-parse", "HEAD"], {
    encoding: "utf8",
  }).trim() === revision, `source cache must be detached at ${revision}`);
  const programs = sources.map((source) => {
    const layout = source.replace(/\.json$/u, ".layout.json");
    const parsed = JSON.parse(fs.readFileSync(path.join(sourceCache, source), "utf8"));
    const abilityNames = parsed.AbilityList.map(({ Name: name }) => name);
    assert(abilityNames.length > 0
      && abilityNames.every((name) => /^StageAbility_[0-9]+$/u.test(name)),
    `invalid ability identity set in ${source}`);
    return {
      source,
      source_sha256: sha256(path.join(sourceCache, source)),
      layout,
      layout_sha256: sha256(path.join(sourceCache, layout)),
      ability_names: abilityNames,
      ability_ids: abilityNames.map((name) => name.slice("StageAbility_".length)),
    };
  });
  const names = programs.flatMap(({ ability_names: values }) => values);
  assert(new Set(names).size === names.length && names.length === 76,
    "battle mechanic ability identity denominator drift");
  const allowed = new Set(programs.flatMap(({ source, layout }) => [source, layout]));
  const nameMatches = ripgrep(names, true);
  const outsideNameMatches = nameMatches.filter(({ file }) => !allowed.has(file));
  assert(outsideNameMatches.length === 0,
    "a frozen battle mechanic ability has a released reference outside its source/layout pair");
  for (const program of programs)
    for (const ability of program.ability_names)
      assert(nameMatches.some(({ file, match }) =>
        (file === program.source || file === program.layout) && match === ability),
      `${ability} is absent from its source/layout pair`);
  const numericMatches = ripgrep(programs.flatMap(({ ability_ids: values }) =>
    values.map((value) => `\\b${value}\\b`)), false);
  assert(numericMatches.length === 0,
    "a frozen battle mechanic numeric ability ID has a released catalog/config reference");
  return {
    schema_revision: "starclock.divergent-universe-battle-mechanic-reachability-proof.v1",
    goal_id: "divergent-universe-runtime-v1",
    batch: "G22-P6-M01",
    status: "CompleteProvenNonRuntimeSourceLibraries",
    source_revision: revision,
    searched_roots: ["Config", "ExcelOutput"],
    identity_forms: ["StageAbility_<id>", "standalone numeric ability ID"],
    programs: programs.map(({ ability_ids: _, ...program }) => ({
      ...program,
      ability_count: program.ability_names.length,
      in_pair_reference_lines: nameMatches.filter(({ file }) =>
        file === program.source || file === program.layout).length,
      outside_pair_reference_lines: 0,
      disposition: "ExcludedWithProof",
      replacement_condition: "Reopen when a released profile, catalog or config outside the source/layout pair references a frozen ability identity.",
    })),
    layouts: programs.map(({ layout, layout_sha256 }) => ({
      source: layout,
      source_sha256: layout_sha256,
      disposition: "MetadataOnlyAudited",
      runtime_trigger: null,
    })),
    summary: {
      source_programs: programs.length,
      layout_companions: programs.length,
      ability_identities: names.length,
      in_pair_reference_lines: nameMatches.length,
      outside_pair_reference_lines: outsideNameMatches.length,
      standalone_numeric_reference_lines: numericMatches.length,
      runtime_programs_admitted: 0,
    },
  };
}

function ripgrep(patterns, fixed) {
  const args = ["--json"];
  if (fixed) args.push("-F");
  args.push("-f", "-", "Config", "ExcelOutput");
  const result = spawnSync("rg", args, {
    cwd: sourceCache,
    input: `${patterns.join("\n")}\n`,
    encoding: "utf8",
    maxBuffer: 100 * 1024 * 1024,
  });
  assert(result.status === 0 || result.status === 1,
    `rg reachability scan failed: ${result.stderr}`);
  return result.stdout.split(/\r?\n/u).filter(Boolean).flatMap((line) => {
    const value = JSON.parse(line);
    if (value.type !== "match") return [];
    const file = value.data.path.text.replaceAll("\\", "/");
    return value.data.submatches.map(({ match }) => ({ file, match: match.text }));
  });
}

function sourceCacheArgument() {
  const index = process.argv.indexOf("--source-cache");
  const value = index >= 0 ? process.argv[index + 1]
    : ".cache/content-reference/turnbasedgamedata";
  assert(typeof value === "string" && value.length > 0, "missing --source-cache value");
  return path.resolve(root, value);
}

function sha256(file) {
  return crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}
function pretty(value) { return `${JSON.stringify(value, null, 2)}\n`; }
function assert(condition, message) { if (!condition) throw new Error(message); }

if (fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const serialized = pretty(buildBattleMechanicReachabilityProof());
  if (process.argv.includes("--check")) {
    assert(fs.readFileSync(path.join(root, output), "utf8") === serialized,
      `${output} is stale`);
    console.log("Divergent Universe battle mechanic reachability proof is current.");
  } else {
    fs.writeFileSync(path.join(root, output), serialized);
    console.log("Generated Divergent Universe battle mechanic reachability proof.");
  }
}
