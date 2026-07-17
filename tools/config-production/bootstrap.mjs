import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const args = process.argv.slice(2);
assert(args.length === 2 && args[0] === "--output", "usage: bootstrap.mjs --output <new-directory>");
const output = path.resolve(root, args[1]);
const relativeOutput = path.relative(root, output).replaceAll("\\", "/");
assert(relativeOutput && !relativeOutput.startsWith("../") && relativeOutput !== ".", "output must be a repository-relative child directory");
assert(!fs.existsSync(output), `refusing to overwrite existing output root ${relativeOutput}`);

run("node", ["tools/content-reference/verify.mjs", "content-reference/v4.4"]);
run("node", ["tools/goal-manifest/verify.mjs"]);
const toolPolicy = readJson("policy/sora-toolchain.json");
const sora = path.join(root, toolPolicy.install_root, "bin", process.platform === "win32" ? "sora.exe" : "sora");
assert(fs.existsSync(sora), `Sora ${toolPolicy.version} is not installed; run ${toolPolicy.install_command}`);

const work = path.join(root, ".cache/config-production-bootstrap");
assert(path.relative(root, work).replaceAll("\\", "/") === ".cache/config-production-bootstrap", "unexpected bootstrap work path");
fs.rmSync(work, { recursive: true, force: true });
const templates = path.join(work, "templates");
const rowsRoot = path.join(work, "rows");
fs.mkdirSync(rowsRoot, { recursive: true });
run(sora, ["--serial", "excel-template", "--project", "config/project.toml", "--out", path.relative(root, templates)]);
writeRows(rowsRoot);
run("cargo", ["run", "--manifest-path", "tools/workbook-bootstrap/Cargo.toml", "--locked", "--quiet", "--", templates, rowsRoot, output]);
console.log(`Bootstrapped ${identityRecords().length} disabled frozen identities into ${relativeOutput}.`);

function writeRows(directory) {
  const manifest = readJson("content-reference/v4.4/manifest.json");
  const repositoryRows = manifest.repositories.map((repository, index) => ({
    id: index + 1,
    stable_key: `source.${repository.id}.${repository.revision}`,
    publisher: repository.id,
    url: repository.remote.replace(/\.git$/, ""),
    accessed_on: "2026-07-17",
    applicable_game_version: "4.4",
    category: "CommunityMaintained",
    confidence: "PreparedExactStructured",
    usage_note: `${repository.usage} Revision ${repository.revision}.`,
    evidence_sha256: sha256Text(JSON.stringify(repository)),
  }));
  writeTsv(directory, "SourceRecord", repositoryRows);
  writeTsv(directory, "EvidenceRecord", [
    { id: 1, stable_key: "evidence.source.dimbreath-v4.4", kind: "SourcePayload", source_record_id: 1, sha256: repositoryRows[0].evidence_sha256, note: "Pinned released-data source identity used by the prepared pack." },
    { id: 2, stable_key: "evidence.source.mar-7th-v4.4", kind: "SourcePayload", source_record_id: 2, sha256: repositoryRows[1].evidence_sha256, note: "Pinned released-data/fallback source identity used by the prepared pack." },
    { id: 3, stable_key: "evidence.reference-pack.v4.4", kind: "SourcePayload", source_record_id: "", sha256: "0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a", note: "Deterministically normalized Version 4.4 reference pack; bootstrap evidence only." },
  ]);
  const identities = identityRecords();
  writeTsv(directory, "ContentIdentity", identities.map((record) => ({
    id: record.transport_id,
    stable_key: record.id,
    content_kind: record.kind,
    name_en: record.name_en,
    name_zh_cn: record.name_zh_cn,
    summary_en: `${record.summary_en} Catalog identity only; executable rows remain pending.`,
    summary_zh_cn: `${record.summary_zh_cn} 当前仅为目录身份；可执行数据尚待转录。`,
    game_version_introduced: "unresolved",
    game_version_snapshot: "4.4",
    release_state: "Released",
    enabled: "false",
    coverage_state: record.coverage,
    source_record_ids: record.source_id,
  })));
  writeTsv(directory, "ContentEvidenceBinding", identities.map((record) => ({
    content_id: record.transport_id,
    sequence: 1,
    fact_key: `bootstrap.identity:${record.id}`,
    source_record_id: record.source_id,
    evidence_record_id: 3,
    quality: record.quality,
    mechanism_quality: record.quality === "ExactPreviousRelease" ? "ExactPreviousReleaseText" : record.quality === "ProjectPolicy" ? "ProjectPolicy" : "ExactStructured",
    approximation_note: "",
  })));
  writeTsv(directory, "ConfigManifest", [{
    game_version: "4.4",
    snapshot_date: "2026-07-17",
    data_revision: "core-combat-v1-bootstrap-v1",
    required_rules_revision: "rules-unimplemented",
    sora_cli_version: "0.3.0",
    numeric_policy_revision: "fixed-i64-6dp-v1",
    rng_algorithm_revision: "chacha8-v1-pending",
    state_hash_revision: "sha256-v1-pending",
    replay_format_version: "replay-v1-pending",
    coverage_manifest_sha256: "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19",
  }]);
}

function identityRecords() {
  const characters = readJson("content-reference/v4.4/characters.json");
  const lightCones = readJson("content-reference/v4.4/light-cones.json");
  const enemyVariants = readJson("content-reference/v4.4/enemy-variants.json");
  const enemyTemplates = readJson("content-reference/v4.4/enemy-templates.json");
  const characterById = new Map(characters.map((record) => [record.id, record]));
  const coneById = new Map(lightCones.map((record) => [record.id, record]));
  const variantById = new Map(enemyVariants.map((record) => [record.id, record]));
  const templateById = new Map(enemyTemplates.map((record) => [record.id, record]));
  const coverage = readJson("evidence/core-combat-v1/coverage/goal-coverage.json");
  const coverageById = new Map(coverage.entries.map((entry) => [entry.id, entry.terminal_state]));
  const characterManifest = readJson("content-manifests/core-combat-v1/released-character-forms.json");
  const coneManifest = readJson("content-manifests/core-combat-v1/released-light-cones.json");
  const standard = readJson("content-manifests/core-combat-v1/standard-v1.json");
  const records = [];
  for (const entry of characterManifest.entries) {
    const reference = required(characterById, entry.reference_id);
    records.push(base(entry.id, "CharacterForm", reference.name_en, reference.name_zh_cn, "Released character combat-form identity.", "已发布角色战斗形态身份。", entry.reference_quality));
  }
  for (const entry of coneManifest.entries) {
    const reference = required(coneById, entry.reference_id);
    records.push(base(entry.id, "LightCone", reference.name_en, reference.name_zh_cn, "Released Light Cone identity.", "已发布光锥身份。", entry.reference_quality));
  }
  for (const entry of standard.enemies) {
    const variant = required(variantById, entry.variant_reference_id);
    const template = required(templateById, variant.enemy_id);
    records.push(base(entry.id, "EnemyVariant", `${template.name_en} Variant`, `${template.name_zh_cn}变体`, "Frozen Standard enemy-variant identity.", "已冻结的标准模式敌人变体身份。", entry.reference_quality));
  }
  for (const entry of standard.encounters) records.push(base(entry.id, "Encounter", title(entry.id), `标准遭遇：${entry.id}`, entry.note, "已冻结的标准模式遭遇身份。", "ExactStructured"));
  for (const entry of standard.scenarios) records.push(base(entry.id, "Scenario", title(entry.id), `标准场景：${entry.id}`, "Frozen seeded Standard scenario identity.", "已冻结的标准模式种子场景身份。", "ExactStructured"));
  records.push(base(standard.profile.id, "StandardProfile", "Standard Version 1 Profile", "标准模式第一版配置", "Ordinary battle profile without challenge semantics.", "不含挑战模式语义的普通战斗配置。", "ProjectPolicy"));
  records.sort((left, right) => left.id.localeCompare(right.id));
  assert(records.length === 283 && new Set(records.map((record) => record.id)).size === 283, "frozen identity bootstrap does not contain exactly 283 unique entries");
  return records.map((record, index) => ({
    ...record,
    transport_id: index + 1,
    coverage: required(coverageById, record.id),
    source_id: record.quality === "ExactPreviousRelease" ? 2 : 1,
  }));
}

function base(id, kind, name_en, name_zh_cn, summary_en, summary_zh_cn, quality) { return { id, kind, name_en, name_zh_cn, summary_en, summary_zh_cn, quality }; }
function title(id) { return id.split(".").slice(1).join(" ").split("-").map((part) => part ? `${part[0].toUpperCase()}${part.slice(1)}` : part).join(" "); }
function required(map, key) { const value = map.get(key); assert(value !== undefined, `missing prepared record ${key}`); return value; }
function writeTsv(directory, name, records) {
  assert(records.length > 0, `${name} requires bootstrap rows`);
  const fields = Object.keys(records[0]);
  const lines = [fields.join("\t"), ...records.map((record) => fields.map((field) => tsvCell(record[field])).join("\t"))];
  fs.writeFileSync(path.join(directory, `${name}.tsv`), `${lines.join("\n")}\n`);
}
function tsvCell(value) { const text = String(value); assert(!/[\t\r\n]/.test(text), "bootstrap TSV cell contains a control separator"); return text; }
function sha256Text(value) { return crypto.createHash("sha256").update(value, "utf8").digest("hex"); }
function readJson(relative) { return JSON.parse(fs.readFileSync(path.join(root, relative), "utf8")); }
function run(command, arguments_) { const environment = command === "cargo" ? { ...process.env, CARGO_TARGET_DIR: path.join(root, ".cache/workbook-bootstrap-target") } : process.env; const result = spawnSync(command, arguments_, { cwd: root, stdio: "inherit", env: environment }); if (result.error) throw result.error; assert(result.status === 0, `${command} ${arguments_.join(" ")} exited with ${result.status}`); }
function assert(condition, message) { if (!condition) throw new Error(message); }
