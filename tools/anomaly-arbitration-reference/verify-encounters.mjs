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
const fallback =
  "/Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

execFileSync(process.execPath, [
  path.join(root,
    "tools/anomaly-arbitration-reference/import-encounters.mjs"),
  "--check",
  "--source-cache",
  path.join(root, ".cache/content-reference"),
  "--fallback-source-cache",
  fallback,
], { stdio: "inherit" });

const expected = {
  "encounters.json": ["Encounter", 5],
  "encounter-waves.json": ["EncounterWave", 10],
  "enemy-slots.json": ["EnemySlot", 16],
  "enemies.json": ["EnemyVariant", 27],
  "enemy-skills.json": ["EnemySkill", 115],
  "enemy-statuses.json": ["EnemyStatus", 52],
  "ability-bindings.json": ["AbilityBinding", 73],
};
const files = {};
for (const [name, [kind, count]] of Object.entries(expected)) {
  const encoded = await readFile(path.join(outputRoot, name));
  const document = JSON.parse(encoded);
  files[name] = { encoded, document };
  assert(document.schema_revision
    === "starclock.anomaly-arbitration-normalized-file.v1"
    && document.goal_id === "anomaly-arbitration-reference-v1"
    && document.profile === "anomaly-arbitration-v1"
    && document.file === name
    && document.record_kind === kind
    && document.records.length === count,
  `${name} envelope/count drift`);
  for (const record of document.records) {
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined, `${record.id} lacks ${field}`);
    assert(record.kind === kind
      && record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn
      && record.coverage_state === "DataReady"
      && record.runtime_executable === false,
    `${record.id} normalized boundary drift`);
    for (const source of record.source_refs) {
      for (const field of schema.types.source_ref.required_fields)
        assert(source[field] !== undefined && source[field] !== "",
          `${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256),
        `${record.id} source digest drift`);
    }
  }
}

const encounters = files["encounters.json"].document.records;
assert(JSON.stringify(encounters.map((row) => [
  row.source_stage_id,
  row.wave_count,
  row.level,
  row.battle_event_id,
])) === JSON.stringify([
  [30508011, 2, 95, "battle-event.30502"],
  [30508012, 2, 95, "battle-event.30502"],
  [30508013, 2, 95, "battle-event.30502"],
  [30508021, 2, 100, "battle-event.30503"],
  [30508022, 2, 120, "battle-event.30504"],
]), "encounter StageConfig projection drift");
assert(JSON.stringify(encounters[3].boss_enemy_ids)
  === JSON.stringify(encounters[4].boss_enemy_ids),
"normal/Plight boss identity drift");

const waves = files["encounter-waves.json"].document.records;
const slots = files["enemy-slots.json"].document.records;
for (const encounter of encounters) {
  const selectedWaves = waves.filter(
    ({ encounter_id: id }) => id === encounter.id,
  );
  assert(selectedWaves.length === 2
    && selectedWaves[0].carries_stage_clock === false
    && selectedWaves[1].carries_stage_clock === true,
  `${encounter.id} wave/clock projection drift`);
  for (const wave of selectedWaves) {
    const selectedSlots = slots.filter(({ encounter_id, wave_order }) =>
      encounter_id === encounter.id && wave_order === wave.wave_order);
    assert(JSON.stringify(selectedSlots.map(({ enemy_id }) => enemy_id))
      === JSON.stringify(wave.enemy_ids),
    `${wave.id} slot projection drift`);
  }
}
assert(new Set(slots.map(({ source_numeric_id: id }) => id)).size === 12,
  "direct StageConfig enemy denominator drift");

function projectedManifestIds(name) {
  return files[name].document.records.flatMap(
    ({ manifest_record_ids: ids }) => ids,
  ).sort(compareText);
}
function expectedManifestIds(categories) {
  return categories.flatMap((category) =>
    manifest.categories[category].records.map(
      ({ id }) => `${category}:${id}`,
    )).sort(compareText);
}
for (const [name, categories] of [
  ["encounters.json", ["stage_configs"]],
  ["enemies.json", ["enemy_variants", "enemy_templates"]],
  ["enemy-skills.json", ["enemy_skills"]],
  ["enemy-statuses.json", ["enemy_statuses"]],
  ["ability-bindings.json", ["config_programs"]],
]) {
  assert(JSON.stringify(projectedManifestIds(name))
    === JSON.stringify(expectedManifestIds(categories)),
  `${name} exact-once manifest drift`);
}

const enemies = files["enemies.json"].document.records;
const enemyIds = new Set(enemies.map(({ id }) => id));
assert(enemies.filter(({ direct_stage_member: direct }) => direct).length
  === 12, "direct enemy count drift");
for (const enemy of enemies) {
  assert(enemy.skill_ids.every((id) =>
    files["enemy-skills.json"].document.records.some(
      ({ id: skillId }) => skillId === id,
    )), `${enemy.id} skill closure drift`);
  assert(enemy.summon_enemy_ids.every((id) => enemyIds.has(id)),
    `${enemy.id} summon closure drift`);
  for (const value of [
    enemy.attack_ratio,
    enemy.defence_ratio,
    enemy.hp_ratio,
    enemy.speed_ratio,
    enemy.stance_ratio,
    ...Object.values(enemy.base_stats),
  ])
    assert(value === null || typeof value === "string",
      `${enemy.id} decimal boundary drift`);
}
const bossRows = enemies.filter(({ rank }) =>
  rank === "Boss" || rank === "LittleBoss");
assert(bossRows.length > 0
  && bossRows.some(({ phase_markers }) => phase_markers.length > 1),
"boss phase-marker coverage drift");

const statuses = files["enemy-statuses.json"].document.records;
const programPaths = new Set(manifest.categories.config_programs.records.map(
  ({ source_path: sourcePath }) => sourcePath,
));
assert(statuses.every((row) =>
  row.owner_resolution === "TransitiveProgramClosure"
  && row.referencing_program_paths.length > 0
  && row.referencing_program_paths.every((sourcePath) =>
    programPaths.has(sourcePath))),
"status-to-program closure drift");
const abilities = files["ability-bindings.json"].document.records;
assert(abilities.every((row) =>
  row.program_body_imported === false
  && /^[0-9a-f]{64}$/u.test(row.program_sha256)),
"ability program-body boundary drift");

console.log(
  "Anomaly Arbitration encounters verified: "
    + Object.entries(files).map(([name, { encoded }]) =>
      `${name}=${createHash("sha256").update(encoded).digest("hex")}`)
      .join(", "),
);
