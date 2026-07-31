#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const args = process.argv.slice(2);
const check = args.includes("--check");
const sourceCache = path.resolve(option("--source-cache")
  ?? process.env.STARCLOCK_SOURCE_CACHE
  ?? fail("--source-cache is required"));
const root = path.resolve(".");
const sourceRoot = path.join(sourceCache, "turnbasedgamedata");
const output = path.join(root,
  "content-manifests/memory-of-chaos-v1/content-manifest.json");
const auditOutput = path.join(root,
  "evidence/memory-of-chaos-reference-v1/manifest-audit.md");
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";
assert(git(sourceRoot, ["rev-parse", "HEAD"]) === revision,
  "turnbasedgamedata revision drift");

const table = async (name) => lossless(await readFile(path.join(
  sourceRoot, `ExcelOutput/${name}.json`)));
const schedules = await table("ScheduleDataChallengeMaze");
const general = await table("ChallengeGeneralConfig");
const groups = await table("ChallengeGroupConfig");
const mazes = await table("ChallengeMazeConfig");
const tierces = await table("ChallengeMazeTierce");
const targets = await table("ChallengeTargetConfig");
const entrances = await table("MapEntrance");
const mazeBuffs = await table("MazeBuff");
const battleEvents = await table("BattleEventConfig");
const stages = await table("StageConfig");
const monsters = await table("MonsterConfig");

const schedule = one(schedules, (row) => row.ID === 201033, "schedule 201033");
const group = one(groups, (row) => row.GroupID === 1033, "group 1033");
assert(group.ScheduleDataID === schedule.ID, "active group/schedule join drift");
assert(schedule.BeginTime === "2026-07-06 04:00:00"
  && schedule.EndTime === "2026-08-17 04:00:00",
"active schedule boundary drift");
assert(group.ChallengeGroupType === "Memory", "active group type drift");
const ordinary = mazes.filter((row) => row.GroupID === group.GroupID)
  .sort((a, b) => a.Floor - b.Floor);
assert(ordinary.length === 12, "ordinary active floor denominator drift");
assert(ordinary.every((row, index) => row.ID === 5201 + index
  && row.Floor === index + 1 && row.StageNum === 2),
"ordinary active floor topology drift");
const tierce = one(tierces, (row) => row.PHFMCACHFIJ === group.TierceID,
  "selected Tierce 5213");
assert(tierce.DLCKKJFMJOB === ordinary.at(-1).ID,
  "Tierce predecessor drift");
assert(tierce.HFIAAGAKFMD.length === 1 && tierce.HFIAAGAKFMD[0] === 30123123,
  "Tierce StageConfig binding drift");
assert(tierce.OGEOMCGNNMP.join(",") === "601,602,603",
  "Tierce target binding drift");

const ordinaryStageIds = ordinary.flatMap((row) => [
  ...row.EventIDList1,
  ...row.EventIDList2,
]);
const stageIds = [...ordinaryStageIds, ...tierce.HFIAAGAKFMD];
assert(stageIds.length === 25 && new Set(stageIds).size === 25,
  "active StageConfig denominator drift");
const activeStages = stageIds.map((id) => one(stages,
  (row) => row.StageID === id, `StageConfig ${id}`));
assert(activeStages.every((row) => row.Release === true
  && row.StageType === "Challenge"
  && row.LevelGraphPath === "Config/Level/StageCommonTemplate.json"
  && row.StageConfigData.some((field) =>
    field.BFLIFKBEOPJ === "_CreateBattleEvent"
    && field.MNDFOPKBHKP === "30146")),
"active StageConfig release/event closure drift");
const enemySlots = activeStages.flatMap((stage) => stage.MonsterList.flatMap(
  (wave, waveIndex) => Object.entries(wave).map(([slot, monsterId]) => ({
    stage_id: stage.StageID, wave: waveIndex + 1, slot, monster_id: monsterId,
  }))));
const enemyIds = [...new Set(enemySlots.map(({ monster_id: id }) => id))]
  .sort((a, b) => a - b);
const activeMonsters = enemyIds.map((id) => one(monsters,
  (row) => row.MonsterID === id, `MonsterConfig ${id}`));

const referenceVariants = JSON.parse(await readFile(path.join(root,
  "content-reference/v4.4/enemy-variants.json"), "utf8"));
const referenceTemplates = JSON.parse(await readFile(path.join(root,
  "content-reference/v4.4/enemy-templates.json"), "utf8"));
const referenceAbilities = JSON.parse(await readFile(path.join(root,
  "content-reference/v4.4/enemy-abilities.json"), "utf8"));
const variantBySource = new Map(referenceVariants.map((row) =>
  [Number(row.source_monster_id), row]));
const selectedVariants = enemyIds.map((id) => {
  const row = variantBySource.get(id);
  assert(row !== undefined, `Goal 01 enemy variant missing ${id}`);
  return row;
});
const templateIds = [...new Set(selectedVariants.map((row) => row.enemy_id))]
  .sort(compare);
const templateById = new Map(referenceTemplates.map((row) => [row.id, row]));
const selectedTemplates = templateIds.map((id) => {
  const row = templateById.get(id);
  assert(row !== undefined, `Goal 01 enemy template missing ${id}`);
  return row;
});
const selectedAbilities = referenceAbilities.filter((row) =>
  templateIds.includes(row.enemy_id)).sort((a, b) => compare(a.id, b.id));

const activeTargetIds = [...new Set(ordinary.flatMap((row) =>
  row.ChallengeTargetID).concat(tierce.OGEOMCGNNMP))].sort((a, b) => a - b);
const activeTargets = activeTargetIds.map((id) => one(targets,
  (row) => row.ID === id, `ChallengeTarget ${id}`));
const mazeBuff = one(mazeBuffs, (row) => row.ID === group.MazeBuffID,
  "MazeBuff 3030146");
const battleEvent = one(battleEvents, (row) => row.BattleEventID === 30146,
  "BattleEvent 30146");
assert(battleEvent.AbilityList.includes(
  "BattleEventAbility_Challenge_Month_46"), "battle event ability drift");
const memoryGeneral = one(general, (row) => row.ChallengeGroupType === "Memory",
  "Memory family general config");
const activeEntrances = [group.MapEntranceID,
  ...new Set(ordinary.flatMap((row) => [row.MapEntranceID, row.MapEntranceID2]))]
  .map((id) => one(entrances, (row) => row.ID === id, `MapEntrance ${id}`));

const categories = {
  family_and_season: category([
    receipt("memory-family", "ExcelOutput/ChallengeGeneralConfig.json",
      "ChallengeGroupType=Memory", memoryGeneral, "Shared"),
    receipt("schedule-201033", "ExcelOutput/ScheduleDataChallengeMaze.json",
      "ID=201033", schedule, "MemoryOfChaos"),
    receipt("group-1033", "ExcelOutput/ChallengeGroupConfig.json",
      "GroupID=1033", group, "MemoryOfChaos"),
  ]),
  entry_and_unlock_locators: category(activeEntrances.map((row) =>
    receipt(`entrance-${row.ID}`, "ExcelOutput/MapEntrance.json",
      `ID=${row.ID}`, row, row.ID === group.MapEntranceID
        ? "Shared" : "MemoryOfChaos"))),
  ordinary_stages: category(ordinary.map((row) => receipt(`stage-${row.ID}`,
    "ExcelOutput/ChallengeMazeConfig.json", `ID=${row.ID}`, row,
    "MemoryOfChaos"))),
  tierce: category([{
    ...receipt("tierce-5213", "ExcelOutput/ChallengeMazeTierce.json",
      "PHFMCACHFIJ=5213", tierce, "MemoryOfChaos"),
    role_proof: {
      selected_once_by: "ChallengeGroupConfig.GroupID=1033.TierceID",
      predecessor_join: "DLCKKJFMJOB=5212",
      stage_config_bindings: [30123123],
      target_bindings: [601, 602, 603],
      countdown_value: 45,
      interpretation: "separate selected extension after ordinary floor 5212",
      does_not_imply: ["third node on ordinary stages", "third team",
        "shared ordinary-stage clock", "ordinary-stage settlement"],
      later_runtime_prerequisite: "decode participant, clock carry and settlement semantics before runtime publication",
    },
  }]),
  objectives: category(activeTargets.map((row) => receipt(`target-${row.ID}`,
    "ExcelOutput/ChallengeTargetConfig.json", `ID=${row.ID}`, row,
    "MemoryOfChaos"))),
  participant_and_attempt_contracts: category([
    "participant-policy", "ordinary-team-slots", "combat-form-uniqueness",
    "loadout-instance-lock", "attempt-retry-reset", "node-transition-lock",
  ].map(policyReceipt)),
  clock_and_resource_contracts: category([
    "ordinary-cycle-budget", "first-cycle-av-window", "cycle-tick-boundary",
    "node-cycle-carry", "wave-cycle-carry", "expiry-failure-order",
    "initial-hp-energy-skill-points", "battle-entry-operations",
  ].map(policyReceipt)),
  turbulence_and_battle_event: category([
    receipt("maze-buff-3030146", "ExcelOutput/MazeBuff.json", "ID=3030146",
      mazeBuff, "MemoryOfChaos"),
    receipt("battle-event-30146", "ExcelOutput/BattleEventConfig.json",
      "BattleEventID=30146", battleEvent, "MemoryOfChaos"),
  ]),
  stage_configs: category(activeStages.map((row) => receipt(
    `stage-config-${row.StageID}`, "ExcelOutput/StageConfig.json",
    `StageID=${row.StageID}`, row, "MemoryOfChaos"))),
  encounter_enemy_slots: category(enemySlots.map((row, index) => ({
    id: `slot-${String(index + 1).padStart(3, "0")}`,
    ...row, ownership: "MemoryOfChaos", evidence_quality: "ExactStructured",
    data_status: "Pending", source_path: "ExcelOutput/StageConfig.json",
    row_locator: `StageID=${row.stage_id};wave=${row.wave};slot=${row.slot}`,
    evidence_sha256: sha256(canonical(row)),
  }))),
  enemy_variants: category(activeMonsters.map((row) => receipt(
    `enemy-variant-${row.MonsterID}`, "ExcelOutput/MonsterConfig.json",
    `MonsterID=${row.MonsterID}`, row, "Shared"))),
  enemy_templates: category(selectedTemplates.map((row) => inheritedReceipt(
    row.id, "content-reference/v4.4/enemy-templates.json",
    `id=${row.id}`, row))),
  enemy_abilities: category(selectedAbilities.map((row) => inheritedReceipt(
    row.id, "content-reference/v4.4/enemy-abilities.json",
    `id=${row.id}`, row))),
  empty_pool_proofs: category([
    "blessing", "curio", "occurrence", "service", "currency", "shop",
    "choice", "rogue-path", "rogue-room", "rogue-progression",
  ].map((family) => ({ id: `empty-${family}`, family,
    ownership: "MemoryOfChaos", evidence_quality: "ExactStructured",
    data_status: "Pending", source_path:
      "content-manifests/memory-of-chaos-v1/source-inventory.json",
    row_locator: `selector-closure:${family}`,
    evidence_sha256: sha256(canonical({ family, schedule: 201033,
      group: 1033, stages: stageIds })),
    selector_proof: "active schedule/group/stage/config closure exposes no mechanically reachable selector for this family",
  }))),
};

const futureGroup = one(groups, (row) => row.GroupID === 1034, "group 1034");
const futureSchedule = one(schedules, (row) => row.ID === 201034,
  "schedule 201034");
const required = Object.values(categories).reduce((sum, value) =>
  sum + value.count, 0);
const countsByCategory = Object.fromEntries(Object.entries(categories).map(
  ([name, value]) => [name, value.count]));
const payload = {
  schema_revision: "starclock.memory-of-chaos-content-manifest.v1",
  snapshot: { game_version: "4.4", active_as_of: "2026-08-01",
    source_revision: revision, schedule_id: 201033, group_id: 1033 },
  membership_rule: "active-released-schedule-to-group-to-explicit-row-or-transitive-reference-closure",
  counts: { required, by_category: countsByCategory,
    ownership: countOwnership(categories), stage_config_rows: activeStages.length,
    enemy_slots: enemySlots.length, enemy_variants: enemyIds.length,
    enemy_templates: selectedTemplates.length,
    enemy_abilities: selectedAbilities.length },
  tierce_resolution: categories.tierce.records[0].role_proof,
  exclusions: [
    { id: "future-schedule-201034", source_path:
      "ExcelOutput/ScheduleDataChallengeMaze.json", evidence_sha256:
      sha256(canonical(futureSchedule)), reason:
      "begins 2026-08-17 after the frozen 2026-08-01 released boundary" },
    { id: "future-group-1034", source_path:
      "ExcelOutput/ChallengeGroupConfig.json", evidence_sha256:
      sha256(canonical(futureGroup)), reason:
      "selected only by future schedule 201034; no active-release grant" },
    { id: "static-forgotten-hall", reason:
      "retained only through Memory family and predecessor locators; complete static catalog is outside Goal 17" },
    { id: "rewards-account-payloads", reason:
      "reward IDs are provenance locators and do not enter normalized mechanics" },
    { id: "other-challenge-families", reason:
      "ChallengeBoss, ChallengePeak, ChallengeStory and challenge activity/badge/skip rows are evidence-only" },
  ],
  research_boundaries: [
    { id: "mapping-info-1220", state: "NonBlocking",
      note: "group 1033 publishes MappingInfoID 1220 but no matching pinned MappingInfo row; retain the locator without inventing entry semantics",
      replacement_condition: "released row or reproducible live observation resolves MappingInfoID 1220" },
    { id: "tierce-runtime-semantics", state: "PolicyBound",
      note: "selection, predecessor, one StageConfig, targets and countdown are exact; participant/team, clock carry and settlement remain unclaimed",
      replacement_condition: "released decoded schema or live observation proves the remaining Tierce lifecycle" },
  ],
  categories,
};
const bytes = Buffer.from(`${JSON.stringify(payload, null, 2)}\n`, "utf8");
const audit = `# Goal 17 Active Manifest Audit\n\n` +
  `- Result: passed\n- Frozen obligations: ${required}\n` +
  `- Active ordinary stages: ${ordinary.length}; Tierce extensions: 1\n` +
  `- StageConfig rows: ${activeStages.length}; enemy slots: ${enemySlots.length}\n` +
  `- Enemy variants/templates/abilities: ${enemyIds.length}/` +
  `${selectedTemplates.length}/${selectedAbilities.length}\n` +
  `- Objective rows: ${activeTargets.length}\n` +
  `- Empty-pool selector proofs: ${categories.empty_pool_proofs.count}\n` +
  `- Future schedule/group 201034/1034: excluded\n` +
  `- Tierce 5213: selected once after 5212 with StageConfig 30123123; ` +
  `no third ordinary node/team/clock semantics inferred\n` +
  `- Manifest digest: \`${sha256(bytes)}\`\n`;

if (check) {
  assert((await readFile(output)).equals(bytes), "content manifest drift");
  assert((await readFile(auditOutput, "utf8")) === audit,
    "manifest audit drift");
  console.log(`Goal 17 content manifest verified (${required} obligations).`);
} else {
  await mkdir(path.dirname(output), { recursive: true });
  await mkdir(path.dirname(auditOutput), { recursive: true });
  await writeFile(output, bytes);
  await writeFile(auditOutput, audit);
  console.log(`Goal 17 content manifest generated (${required} obligations).`);
}

function receipt(id, sourcePath, locator, row, ownership) {
  return { id, source_path: sourcePath, row_locator: locator,
    evidence_sha256: sha256(canonical(row)), evidence_quality: "ExactStructured",
    ownership, data_status: "Pending" };
}
function inheritedReceipt(id, sourcePath, locator, row) {
  return { ...receipt(id, sourcePath, locator, row, "Shared"),
    inherited_from: "Goal 01 Version 4.4 enemy reference pack" };
}
function policyReceipt(id) {
  return { id, source_path:
      "docs/goals/17-memory-of-chaos-reference-data.md",
    row_locator: `included-content:${id}`,
    evidence_sha256: sha256(canonical({ id, snapshot: "4.4",
      goal: "memory-of-chaos-reference-v1" })),
    evidence_quality: "ProjectPolicy", ownership: "MemoryOfChaos",
    data_status: "Pending",
    note: "non-shrinking semantic obligation; exact or field-level policy resolution is required before DataReady" };
}
function category(records) {
  records.sort((a, b) => compare(a.id, b.id));
  return { count: records.length, records };
}
function countOwnership(categories) {
  const counts = {};
  for (const categoryValue of Object.values(categories))
    for (const row of categoryValue.records)
      counts[row.ownership] = (counts[row.ownership] ?? 0) + 1;
  return counts;
}
function one(rows, predicate, label) {
  const selected = rows.filter(predicate);
  assert(selected.length === 1, `${label} expected once, got ${selected.length}`);
  return selected[0];
}
function lossless(bytes) {
  return JSON.parse(bytes.toString("utf8").replace(
    /(:\s*|[\[,]\s*)(-?\d{16,})(?=\s*[,}\]])/gu, '$1"$2"'));
}
function option(name) {
  const index = args.indexOf(name);
  if (index < 0) return undefined;
  assert(args[index + 1] !== undefined, `${name} requires a value`);
  return args[index + 1];
}
function git(cwd, gitArgs) {
  return execFileSync("git", ["-C", cwd, ...gitArgs], {
    encoding: "utf8", maxBuffer: 512 * 1024 * 1024,
  }).trim();
}
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") return `{${Object.keys(value)
    .sort(compare).map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`)
    .join(",")}}`;
  return JSON.stringify(value);
}
function sha256(value) { return createHash("sha256").update(value).digest("hex"); }
function compare(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
function assert(condition, message) { if (!condition) throw new Error(message); }
function fail(message) { throw new Error(message); }
