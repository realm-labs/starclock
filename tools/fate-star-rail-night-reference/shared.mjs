#!/usr/bin/env node

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
const sourceCache = path.resolve(argument("--source-cache"));
const batch = argument("--batch");
const check = args.includes("--check");
const turnRoot = path.join(sourceCache, "turnbasedgamedata");
const manifest = json(path.join(root,
  "content-manifests/fate-star-rail-night-v1/content-manifest.json"));
const definitions = {
  "G19-P2-B1": {
    output: "pool-audits.json",
    purpose: "generated selector-closure audits for absent generic pools",
    families: ["ExactZeroPool"],
  },
  "G19-P2-B2": {
    output: "battle-bindings.json",
    purpose: "BattleArea, MazeBuff, BattleEvent and BattleTarget relationships",
    families: ["BattleArea", "BattleAreaConfig", "MazeBuff", "BattleEvent", "BattleTarget"],
  },
  "G19-P2-B3": {
    output: "encounters.json",
    purpose: "FateActivity stages, ordered waves and enemy slots",
    families: ["Stage"],
  },
  "G19-P2-B4": {
    output: "enemies.json",
    purpose: "enemy variants, templates, skills, statuses and configuration locators",
    families: ["EnemyVariant", "EnemyTemplate", "EnemySkill", "EnemyStatus"],
  },
};
const definition = definitions[batch];
assert(definition !== undefined, `unsupported shared batch ${batch}`);
const obligations = manifest.obligations.filter(({ family }) =>
  definition.families.includes(family));
let records = obligations.flatMap(normalizeObligation);
records.sort((left, right) => compareText(left.stable_id, right.stable_id));
const document = {
  schema_revision: "starclock.fate-star-rail-night-shared-normalized.v1",
  goal_id: "fate-star-rail-night-reference-v1",
  batch,
  purpose: definition.purpose,
  manifest_binding: manifest.canonical_obligations_sha256,
  counts: {
    manifest_obligations: obligations.length,
    records: records.length,
    families: countBy(records, "family"),
    disposition: countBy(records, "disposition"),
  },
  records,
};
document.canonical_records_sha256 = digest(canonical(records));
const output = path.join(root, "content-reference/fate-star-rail-night-v1",
  definition.output);
const serialized = `${JSON.stringify(document, null, 2)}\n`;

if (check) {
  assert(fs.existsSync(output), `missing ${definition.output}`);
  assert(fs.readFileSync(output, "utf8") === serialized,
    `shared normalized drift ${definition.output}`);
  assert(new Set(records.map(({ stable_id: id }) => id)).size === records.length,
    "duplicate shared normalized stable id");
  console.log(summary("verified"));
} else {
  fs.writeFileSync(output, serialized);
  console.log(summary("wrote"));
}

function normalizeObligation(obligation) {
  if (obligation.family === "ExactZeroPool") return [zeroRecord(obligation)];
  const index = Number(obligation.locator.replace(/^index:/u, ""));
  assert(Number.isSafeInteger(index), `invalid locator ${obligation.locator}`);
  const rows = losslessJson(fs.readFileSync(path.join(turnRoot,
    obligation.source_path)));
  const row = rows[index];
  assert(row !== undefined, `missing source row ${obligation.source_path}#${index}`);
  if (obligation.family === "Stage") return stageRecords(obligation, row);
  return [sourceRecord(obligation, row)];
}

function zeroRecord(obligation) {
  return {
    stable_id: `fate-star-rail-night.pool-audit.${slug(obligation.source_family)}`,
    family: "PoolAudit",
    name_zh: `${obligation.source_family} 池审计`,
    name_en: `${obligation.source_family} pool audit`,
    summary_zh: "完整 Fate/FateRin 选择器闭包中没有可达的通用池成员。",
    summary_en: "No generic pool member is reachable from the complete Fate/FateRin selector closure.",
    ownership: obligation.ownership,
    disposition: "DataReady",
    enabled: true,
    source_refs: [{ path: obligation.source_path, locator: obligation.locator,
      sha256: obligation.source_sha256 }],
    evidence_quality: "ExactStructured",
    mechanism_quality: "SelectorClosureExact",
    confidence: "High",
    mechanic_payload: { required: "0", accounted: "0", data_ready: "0" },
    notes: obligation.note,
  };
}

function sourceRecord(obligation, row) {
  const id = firstId(row) ?? obligation.locator.replace("index:", "row-");
  return {
    stable_id: `fate-star-rail-night.shared.${slug(obligation.family)}.${slug(String(id))}`,
    family: obligation.family,
    name_zh: `${obligation.family} ${id}`,
    name_en: `${obligation.family} ${id}`,
    summary_zh: `由 Fate 直接引用闭包选中的 ${obligation.family} 事实。`,
    summary_en: `${obligation.family} fact selected by the direct Fate reference closure.`,
    ownership: obligation.ownership,
    disposition: obligation.disposition,
    enabled: obligation.disposition === "DataReady",
    source_refs: [{ path: obligation.source_path, locator: obligation.locator,
      sha256: obligation.source_sha256 }],
    evidence_quality: "ExactStructured",
    mechanism_quality: obligation.relation ?? "TransportedExact",
    confidence: obligation.disposition === "DataReady" ? "High" : "ResearchRequired",
    mechanic_payload: canonicalValue(stripPresentation(row)),
    notes: obligation.disposition === "ResearchRequired"
      ? "Typed reference meaning must close before release; scalar equality alone is not promoted."
      : "Canonical source-shaped payload; runtime lowering is excluded.",
  };
}

function stageRecords(obligation, row) {
  const stageId = String(row.StageID);
  const base = sourceRecord(obligation, row);
  base.stable_id = `fate-star-rail-night.stage.${slug(stageId)}`;
  base.family = "Stage";
  const derived = [base];
  for (const [waveIndex, wave] of (row.MonsterList ?? []).entries()) {
    const waveId = `${stageId}.${waveIndex + 1}`;
    derived.push(derivedRecord(obligation, "Wave", waveId, {
      stage_id: stageId, order: String(waveIndex + 1),
    }));
    const slots = Object.entries(wave).filter(([key]) => /^Monster\d+$/u.test(key))
      .sort(([left], [right]) => Number(left.replace("Monster", ""))
        - Number(right.replace("Monster", "")));
    for (const [slotIndex, [, monsterId]] of slots.entries())
      derived.push(derivedRecord(obligation, "EnemySlot",
        `${waveId}.${slotIndex + 1}`, {
          stage_id: stageId, wave_order: String(waveIndex + 1),
          slot_order: String(slotIndex + 1), enemy_variant_source_id: String(monsterId),
        }));
  }
  return derived;
}

function derivedRecord(obligation, family, id, payload) {
  return {
    stable_id: `fate-star-rail-night.${slug(family)}.${slug(id)}`,
    family,
    name_zh: `${family} ${id}`,
    name_en: `${family} ${id}`,
    summary_zh: `从 FateActivity StageConfig 顺序派生的 ${family}。`,
    summary_en: `${family} derived in source order from FateActivity StageConfig.`,
    ownership: "FateStarRailNight",
    disposition: "DataReady",
    enabled: true,
    source_refs: [{ path: obligation.source_path, locator: obligation.locator,
      sha256: obligation.source_sha256 }],
    evidence_quality: "ExactStructured",
    mechanism_quality: "DerivedExact",
    confidence: "High",
    mechanic_payload: payload,
    notes: "Derived child of one manifest Stage obligation; does not enlarge the exact-once source denominator.",
  };
}

function firstId(row) {
  for (const key of ["StageID", "BattleAreaID", "ID", "BattleEventID",
    "MonsterID", "MonsterTemplateID", "SkillID", "StatusID"])
    if (row[key] !== undefined) return String(row[key]);
  return undefined;
}

function stripPresentation(value, key = "") {
  if (Array.isArray(value)) return value.map((entry) => stripPresentation(entry, key));
  if (value && typeof value === "object") {
    if (Object.keys(value).length === 1 && Object.hasOwn(value, "Hash")) return undefined;
    return Object.fromEntries(Object.entries(value)
      .map(([childKey, child]) => [childKey, stripPresentation(child, childKey)])
      .filter(([, child]) => child !== undefined));
  }
  if (typeof value === "string" && (/path|icon|sprite|prefab|image|audio/iu.test(key)
    || /^(?:SpriteOutput|UI|Characters|Audio|Texture|Effects)\//u.test(value)))
    return undefined;
  return value;
}

function canonicalValue(value) {
  if (Array.isArray(value)) return value.map(canonicalValue);
  if (value && typeof value === "object") return Object.fromEntries(
    Object.entries(value).sort(([left], [right]) => compareText(left, right))
      .map(([key, child]) => [key, canonicalValue(child)]));
  if (typeof value === "number") return String(value);
  return value;
}

function losslessJson(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu, '$1"$2"'));
}

function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") return `{${Object.keys(value)
    .sort(compareText).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function countBy(rows, field) {
  return Object.fromEntries([...new Set(rows.map((row) => row[field]))]
    .sort(compareText).map((value) => [value,
      rows.filter((row) => row[field] === value).length]));
}

function json(absolute) {
  return JSON.parse(fs.readFileSync(absolute, "utf8"));
}

function argument(name) {
  const index = args.indexOf(name);
  assert(index !== -1 && args[index + 1], `${name} requires a value`);
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
  return `${batch} shared pack ${verb} (${obligations.length} obligations, ${records.length} records, ${document.canonical_records_sha256}).`;
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
