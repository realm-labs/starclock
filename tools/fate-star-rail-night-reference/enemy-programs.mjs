#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = path.resolve(argument("--source-cache"));
const check = args.includes("--check");
const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const manifest = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json"));
const templateObligations = manifest.obligations.filter(({ family }) =>
  family === "EnemyTemplate");
const records = [];
const presentationExclusions = new Set();

for (const obligation of templateObligations) {
  const index = Number(obligation.locator.replace("index:", ""));
  const template = losslessJson(fs.readFileSync(path.join(turnRoot,
    obligation.source_path)))[index];
  for (const [programKind, relative] of [
    ["CharacterConfig", template.JsonConfig], ["AIConfig", template.AIPath],
  ]) {
    assert(typeof relative === "string" && relative.endsWith(".json"),
      `missing ${programKind} path for template ${template.MonsterTemplateID}`);
    const bytes = fs.readFileSync(path.join(turnRoot, relative));
    const value = losslessJson(bytes);
    collectPresentationPaths(value, presentationExclusions);
    records.push({
      stable_id: `fate-star-rail-night.enemy-program.${slug(programKind)}.${slug(String(template.MonsterTemplateID))}`,
      family: "EnemyProgram",
      name_zh: `${template.MonsterTemplateID} ${programKind} 配置`,
      name_en: `${template.MonsterTemplateID} ${programKind} configuration`,
      summary_zh: "保留敌人配置身份、大小与摘要，不复制或执行上游程序。",
      summary_en: "Retains enemy configuration identity, size and digest without copying or executing the upstream program.",
      ownership: "Shared",
      disposition: "DataReady",
      enabled: true,
      source_refs: [{ path: relative, locator: "file", sha256: digest(bytes) }],
      evidence_quality: "ExactStructured",
      mechanism_quality: "ProgramDigestExact",
      confidence: "High",
      mechanic_payload: {
        enemy_template_source_id: String(template.MonsterTemplateID),
        program_kind: programKind,
        bytes: String(bytes.length),
        program_sha256: digest(bytes),
      },
      notes: "Runtime lowering is excluded; presentation AnimEvent references are named exclusions.",
    });
  }
}
records.sort((left, right) => left.stable_id < right.stable_id ? -1 : 1);
const document = {
  schema_revision: "starclock.fate-star-rail-night-enemy-programs.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch: "G19-P2-B4",
  manifest_binding: manifest.canonical_obligations_sha256,
  parent_template_obligations: templateObligations.length,
  counts: {
    records: records.length,
    character_configs: records.filter(({ mechanic_payload: payload }) =>
      payload.program_kind === "CharacterConfig").length,
    ai_configs: records.filter(({ mechanic_payload: payload }) =>
      payload.program_kind === "AIConfig").length,
    presentation_exclusions: presentationExclusions.size,
  },
  exact_ability_program_boundary: "No released typed path from these five templates selects a separate ability-program file; eight MonsterSkillConfig rows are the exact skill closure.",
  presentation_exclusions: [...presentationExclusions].sort(),
  records,
};
document.canonical_records_sha256 = digest(canonical(records));
const output = path.join(root,
  "content-reference/fate-star-rail-night-v1/enemy-programs.json");
const serialized = `${JSON.stringify(document, null, 2)}\n`;

if (check) {
  assert(fs.existsSync(output), "missing enemy-programs.json");
  assert(fs.readFileSync(output, "utf8") === serialized,
    "enemy program closure drift");
  assert(records.length === templateObligations.length * 2,
    "enemy program exact closure drift");
  console.log(summary("verified"));
} else {
  fs.writeFileSync(output, serialized);
  console.log(summary("wrote"));
}

function collectPresentationPaths(value, target) {
  if (Array.isArray(value)) for (const entry of value) collectPresentationPaths(entry, target);
  else if (value && typeof value === "object")
    for (const entry of Object.values(value)) collectPresentationPaths(entry, target);
  else if (typeof value === "string" && value.startsWith("Config/ConfigAnimEvents/"))
    target.add(value);
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu, '$1"$2"'));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value).sort()
    .map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function json(absolute) {
  return JSON.parse(fs.readFileSync(absolute, "utf8"));
}

function argument(name) {
  const index = args.indexOf(name);
  if (index === -1 || !args[index + 1]) throw new Error(`${name} requires a value`);
  return args[index + 1];
}

function slug(value) {
  return value.replace(/([a-z0-9])([A-Z])/gu, "$1-$2")
    .replace(/[^A-Za-z0-9]+/gu, "-").replace(/^-|-$/gu, "").toLowerCase();
}

function digest(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function summary(verb) {
  return `Goal 19 enemy programs ${verb} (${records.length} records, ${document.canonical_records_sha256}).`;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
