import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const sourceRoot = resolve(process.env.STARCLOCK_PF_SOURCE ?? ".cache/pure-fiction/turnbasedgamedata");
const output = resolve("content-manifests/pure-fiction-v1/content-manifest.json");
const check = process.argv.includes("--check");
const inventory = JSON.parse(readFileSync("content-manifests/pure-fiction-v1/source-inventory.json", "utf8"));
const inventoryByPath = new Map(inventory.records.map((row) => [row.path, row]));

function parse(path) {
  const text = readFileSync(`${sourceRoot}/${path}`, "utf8")
    .replace(/("Hash"\s*:\s*)(\d+)/g, '$1"$2"');
  return JSON.parse(text);
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") return `{${Object.keys(value)
    .sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(",")}}`;
  return JSON.stringify(value);
}

function locatorValue(value) {
  if (/^-?\d+$/.test(value)) return Number(value);
  if (value === "true" || value === "false") return value === "true";
  return value;
}

function sourceDigest(path, locator) {
  const record = inventoryByPath.get(path);
  if (!record) throw new Error(`Source absent from inventory: ${path}`);
  if (path.startsWith("ExcelOutput/") && path.endsWith(".json")) {
    const rows = parse(path);
    if (Array.isArray(rows)) {
      const selectors = locator.split(":").map((part) => part.split("=", 2))
        .filter(([key, value]) => key && value !== undefined
          && rows.some((row) => Object.prototype.hasOwnProperty.call(row, key)));
      const matches = rows.filter((row) => selectors.every(([key, value]) =>
        row[key] === locatorValue(value)));
      if (selectors.length && matches.length === 1)
        return createHash("sha256").update(canonicalJson(matches[0])).digest("hex");
    }
  }
  return record.sha256;
}

const tables = {
  groups: parse("ExcelOutput/ChallengeStoryGroupConfig.json"),
  extras: parse("ExcelOutput/ChallengeStoryGroupExtra.json"),
  mazes: parse("ExcelOutput/ChallengeStoryMazeConfig.json"),
  mazeExtras: parse("ExcelOutput/ChallengeStoryMazeExtra.json"),
  tierces: parse("ExcelOutput/ChallengeStoryMazeTierce.json"),
  targets: parse("ExcelOutput/ChallengeStoryTargetConfig.json"),
  themes: parse("ExcelOutput/ChallengeStoryTheme.json"),
  schedules: parse("ExcelOutput/ScheduleDataChallengeStory.json"),
  buffs: parse("ExcelOutput/MazeBuff.json"),
  battleEvents: parse("ExcelOutput/BattleEventConfig.json"),
  stages: parse("ExcelOutput/StageConfig.json"),
  monsters: parse("ExcelOutput/MonsterConfig.json"),
  templates: parse("ExcelOutput/MonsterTemplateConfig.json"),
  skills: parse("ExcelOutput/MonsterSkillConfig.json"),
  statuses: parse("ExcelOutput/MonsterStatusConfig.json")
};

const activeGroup = tables.groups.find((row) => row.GroupID === 2024);
const activeSchedule = tables.schedules.find((row) => row.ID === 202024);
const scheduledGroup = tables.groups.find((row) => row.GroupID === 2025);
if (!activeGroup || activeGroup.ScheduleDataID !== 202024 || !activeSchedule) throw new Error("Active selector mismatch");
if (activeSchedule.BeginTime !== "2026-06-22 04:00:00" || activeSchedule.EndTime !== "2026-08-03 04:00:00") throw new Error("Active period drift");
if (!scheduledGroup || scheduledGroup.ScheduleDataID !== 202025) throw new Error("Scheduled exclusion mismatch");
const groupExtra = tables.extras.find((row) => row.GroupID === activeGroup.GroupID);
const mazes = tables.mazes.filter((row) => row.GroupID === activeGroup.GroupID).sort((a, b) => a.Floor - b.Floor);
if (mazes.length !== 4 || mazes.some((row, index) => row.ID !== 20241 + index || row.StageNum !== 2)) throw new Error("Active stage closure drift");
const mazeExtras = mazes.map((maze) => tables.mazeExtras.find((row) => row.ID === maze.ID));
if (mazeExtras.some((row) => !row)) throw new Error("Missing active maze extra");
const tierce = tables.tierces.find((row) => row.PHFMCACHFIJ === activeGroup.TierceID);
if (!tierce || tierce.DLCKKJFMJOB !== 20244) throw new Error("Tierce selector drift");

const stageIds = [
  ...mazes.flatMap((maze) => [...maze.EventIDList1, ...maze.EventIDList2]),
  ...tierce.HFIAAGAKFMD
].sort((a, b) => a - b);
const stageRows = stageIds.map((id) => tables.stages.find((row) => row.StageID === id));
if (stageRows.some((row) => !row || !row.Release)) throw new Error("Missing released StageConfig row");

const initialEnemyIds = [...new Set(stageRows.flatMap((stage) => stage.MonsterList.flatMap((wave) => Object.values(wave))))].sort((a, b) => a - b);
const monsterById = new Map(tables.monsters.map((row) => [row.MonsterID, row]));
const enemyIds = new Set(initialEnemyIds);
const pending = [...initialEnemyIds];
while (pending.length > 0) {
  const id = pending.shift();
  const row = monsterById.get(id);
  if (!row) throw new Error(`Missing MonsterConfig row ${id}`);
  for (const summon of row.SummonIDList ?? []) {
    if (!enemyIds.has(summon)) {
      enemyIds.add(summon);
      pending.push(summon);
    }
  }
}
const enemyRows = [...enemyIds].sort((a, b) => a - b).map((id) => monsterById.get(id));
const templateIds = [...new Set(enemyRows.map((row) => row.MonsterTemplateID))].sort((a, b) => a - b);
const templateById = new Map(tables.templates.map((row) => [row.MonsterTemplateID, row]));
const templateRows = templateIds.map((id) => templateById.get(id));
if (templateRows.some((row) => !row)) throw new Error("Missing MonsterTemplateConfig row");
const skillIds = [...new Set(enemyRows.flatMap((row) => row.SkillList ?? []))].sort((a, b) => a - b);
const skillById = new Map(tables.skills.map((row) => [row.SkillID, row]));
if (skillIds.some((id) => !skillById.has(id))) throw new Error("Missing MonsterSkillConfig row");

const obligations = [];
function add(id, category, sourcePath, locator, owner = "PureFiction", note = "") {
  obligations.push({
    id,
    category,
    owner,
    source_path: sourcePath,
    source_locator: locator,
    evidence_digest: sourceDigest(sourcePath, locator),
    state: "Required",
    note
  });
}
add("pf.profile.v1", "profile", "ExcelOutput/ChallengeStoryGroupConfig.json", "GroupID=2024");
add("pf.season.2024", "season", "ExcelOutput/ScheduleDataChallengeStory.json", "ID=202024");
add("pf.entry.3000301", "entry", "ExcelOutput/ChallengeStoryMazeConfig.json", "GroupID=2024:MapEntranceID=3000301");
for (const outcome of ["node_score_finalized", "season_score_aggregated"]) add(`pf.terminal.${outcome}`, "terminal_outcome", "ExcelOutput/ChallengeStoryMazeExtra.json", "GroupID=2024", "PureFiction", "Reference lifecycle contract; no runtime claim");
for (const maze of mazes) {
  add(`pf.stage.${maze.ID}`, "stage", "ExcelOutput/ChallengeStoryMazeConfig.json", `ID=${maze.ID}`);
  for (const side of [1, 2]) add(`pf.node.${maze.ID}.${side}`, "ordinary_node", "ExcelOutput/ChallengeStoryMazeConfig.json", `ID=${maze.ID}:side=${side}`);
  add(`pf.clock.${maze.ID}`, "turn_clock", "ExcelOutput/ChallengeStoryMazeExtra.json", `ID=${maze.ID}`);
}
add(`pf.tierce.${tierce.PHFMCACHFIJ}`, "tierce_starward", "ExcelOutput/ChallengeStoryMazeTierce.json", `PHFMCACHFIJ=${tierce.PHFMCACHFIJ}`);
add(`pf.node.${tierce.PHFMCACHFIJ}.1`, "tierce_node", "ExcelOutput/ChallengeStoryMazeTierce.json", `PHFMCACHFIJ=${tierce.PHFMCACHFIJ}:node=1`);
for (const kind of ["participant_policy", "loadout_lock", "attempt_policy", "retry_policy", "spawn_policy", "score_aggregation", "initial_resources"]) add(`pf.contract.${kind}`, kind, "ExcelOutput/ChallengeStoryMazeConfig.json", "GroupID=2024", "PureFiction", "Stable challenge-family reference obligation");
for (const targetId of [...new Set([...mazes.flatMap((row) => row.ChallengeTargetID), ...tierce.OGEOMCGNNMP])].sort((a, b) => a - b)) {
  const path = targetId >= 4000 ? "ExcelOutput/ChallengeStoryMazeTierce.json" : "ExcelOutput/ChallengeStoryTargetConfig.json";
  add(`pf.objective.${targetId}`, "objective_star", path, targetId >= 4000 ? `PHFMCACHFIJ=${tierce.PHFMCACHFIJ}:target=${targetId}` : `ID=${targetId}`);
}
const buffIds = [...new Set([activeGroup.MazeBuffID, ...mazes.map((row) => row.MazeBuffID), ...groupExtra.SubMazeBuffList, ...groupExtra.BuffList])].sort((a, b) => a - b);
for (const id of buffIds) {
  const row = tables.buffs.find((candidate) => candidate.ID === id);
  if (!row) throw new Error(`Missing MazeBuff ${id}`);
  const category = groupExtra.BuffList.includes(id) ? "cacophony" : groupExtra.SubMazeBuffList.includes(id) ? "grit_fever" : "maze_buff";
  add(`pf.buff.${id}`, category, "ExcelOutput/MazeBuff.json", `ID=${id}`);
}
add(`pf.theme.${groupExtra.ThemeID}`, "theme", "ExcelOutput/ChallengeStoryTheme.json", `ThemeID=${groupExtra.ThemeID}`);
const battleEventIds = [...new Set(stageRows.flatMap((stage) => stage.StageConfigData)
  .filter((row) => row.BFLIFKBEOPJ === "_CreateBattleEvent")
  .map((row) => Number(row.MNDFOPKBHKP)))].sort((a, b) => a - b);
for (const id of battleEventIds) {
  if (!tables.battleEvents.some((row) => row.BattleEventID === id)) throw new Error(`Missing BattleEvent ${id}`);
  add(`pf.battle-event.${id}`, "battle_event", "ExcelOutput/BattleEventConfig.json", `BattleEventID=${id}`, "Shared");
}
const activeBindingKeys = buffIds.map((id) => tables.buffs.find((row) => row.ID === id).InBattleBindingKey);
const activeProgramPaths = inventory.records
  .filter((row) => row.classification === "pure_fiction_program_candidate")
  .filter((row) => activeBindingKeys.some((key) => readFileSync(`${sourceRoot}/${row.path}`, "utf8").includes(key)))
  .map((row) => row.path)
  .sort();
for (const path of activeProgramPaths) add(`pf.program.${path.replace(/[^A-Za-z0-9]+/g, ".").replace(/^\.|\.$/g, "")}`, "ability_program", path, "active MazeBuff binding-key match", "PureFiction");
for (const stage of stageRows) {
  add(`pf.stage-config.${stage.StageID}`, "stage_config", "ExcelOutput/StageConfig.json", `StageID=${stage.StageID}`, "Shared");
  stage.MonsterList.forEach((wave, waveIndex) => {
    add(`pf.wave.${stage.StageID}.${waveIndex + 1}`, "encounter_wave", "ExcelOutput/StageConfig.json", `StageID=${stage.StageID}:wave=${waveIndex + 1}`, "PureFiction");
    Object.entries(wave).sort(([left], [right]) => left.localeCompare(right)).forEach(([slot, monsterId]) => add(`pf.enemy-slot.${stage.StageID}.${waveIndex + 1}.${slot}`, "enemy_slot", "ExcelOutput/StageConfig.json", `StageID=${stage.StageID}:wave=${waveIndex + 1}:${slot}=${monsterId}`, "PureFiction"));
  });
}
for (const row of enemyRows) add(`pf.enemy.${row.MonsterID}`, "enemy_variant", "ExcelOutput/MonsterConfig.json", `MonsterID=${row.MonsterID}`, "Shared");
for (const row of templateRows) add(`pf.enemy-template.${row.MonsterTemplateID}`, "enemy_template", "ExcelOutput/MonsterTemplateConfig.json", `MonsterTemplateID=${row.MonsterTemplateID}`, "Shared");
for (const id of skillIds) add(`pf.enemy-skill.${id}`, "enemy_skill", "ExcelOutput/MonsterSkillConfig.json", `SkillID=${id}`, "Shared");
const characterConfigPaths = [...new Set(templateRows.map((row) => row.JsonConfig).filter(Boolean))].sort();
for (const path of characterConfigPaths) {
  if (!inventoryByPath.has(path)) throw new Error(`Enemy character config absent from inventory: ${path}`);
  add(`pf.enemy-character-config.${path.replace(/[^A-Za-z0-9]+/g, ".").replace(/^\.|\.$/g, "")}`, "enemy_character_config", path, "selected MonsterTemplateConfig.JsonConfig", "Shared");
}
const aiPaths = [...new Set([
  ...templateRows.map((row) => row.AIPath),
  ...enemyRows.map((row) => row.OverrideAIPath)
].filter(Boolean))].sort();
for (const path of aiPaths) {
  if (!inventoryByPath.has(path)) throw new Error(`Enemy AI config absent from inventory: ${path}`);
  add(`pf.enemy-ai.${path.replace(/[^A-Za-z0-9]+/g, ".").replace(/^\.|\.$/g, "")}`, "enemy_ai", path, "selected template/variant AI path", "Shared");
}
const abilityKeys = new Set();
for (const path of characterConfigPaths) {
  const config = parse(path);
  for (const skill of config.SkillList ?? []) if (skill.EntryAbility) abilityKeys.add(skill.EntryAbility);
  for (const key of config.AbilityList ?? []) abilityKeys.add(key);
  for (const binding of config.SkillAbilityList ?? []) for (const key of binding.AbilityList ?? []) abilityKeys.add(key);
}
const selectedEnemyAbilityPaths = inventory.records
  .filter((row) => row.classification === "enemy_ability_closure_candidate")
  .filter((row) => {
    const text = readFileSync(`${sourceRoot}/${row.path}`, "utf8");
    return [...abilityKeys].some((key) => text.includes(`\"${key}\"`));
  })
  .map((row) => row.path)
  .sort();
for (const path of selectedEnemyAbilityPaths) add(`pf.enemy-ability.${path.replace(/[^A-Za-z0-9]+/g, ".").replace(/^\.|\.$/g, "")}`, "enemy_ability", path, "selected character EntryAbility/AbilityList match", "Shared");
const selectedAbilityText = selectedEnemyAbilityPaths.map((path) => readFileSync(`${sourceRoot}/${path}`, "utf8")).join("\n");
const statusRows = tables.statuses.filter((row) => row.ModifierName && selectedAbilityText.includes(row.ModifierName));
for (const row of statusRows) add(`pf.enemy-status.${row.StatusID}`, "enemy_status", "ExcelOutput/MonsterStatusConfig.json", `StatusID=${row.StatusID}`, "Shared");
for (const family of ["blessing", "curio", "occurrence", "event_choice", "service", "currency", "shop"]) add(`pf.zero-proof.${family}`, "exact_zero_proof", "ExcelOutput/ChallengeStoryGroupConfig.json", `GroupID=2024:family=${family}`, "EvidenceOnly", "Selector closure must prove zero reachable rows");
const mechanicFamilies = [
  "profile_entry", "stage_unlock", "participant_uniqueness", "loadout_lock", "attempt_begin", "attempt_retry", "attempt_abandon",
  "node_transition", "clock_tick", "clock_expiry", "continuous_spawn", "simultaneous_defeat", "defeat_score", "damage_score",
  "score_cap", "objective_aggregation", "grit_gain", "fever_transition", "fever_teardown", "cacophony_2261", "cacophony_2264",
  "cacophony_2263", "initial_resources", "tierce_entry", "tierce_settlement"
];
for (const family of mechanicFamilies) add(`pf.rule.${family}`, "mechanic_rule", "ExcelOutput/ChallengeStoryGroupConfig.json", `GroupID=2024:mechanic=${family}`, "PureFiction", "Reference contribution only; runtime remains unreleased");
const fixtureFamilies = [
  "profile_and_stage_flow", "participant_and_loadout_rejection", "attempt_retry_and_reset", "clock_tick_and_expiry",
  "spawn_refill_order", "simultaneous_defeats", "defeat_scoring", "damage_partial_scoring", "score_cap_and_aggregation",
  "objective_stars", "grit_gain", "fever_enter_and_teardown", "cacophony_2261", "cacophony_2264", "cacophony_2263",
  "initial_resources", "tierce_entry_and_settlement", "encounter_wave_closure"
];
for (const family of fixtureFamilies) add(`pf.fixture.${family}`, "semantic_fixture", "ExcelOutput/ChallengeStoryGroupConfig.json", `GroupID=2024:fixture=${family}`, "EvidenceOnly", "Reference semantic review fixture; not a runtime golden");
add("pf.exclusion.group.2025", "scheduled_unreleased_exclusion", "ExcelOutput/ScheduleDataChallengeStory.json", "ID=202025", "EvidenceOnly", "Begins after the fixed 2026-07-30 access boundary");

obligations.sort((a, b) => a.id.localeCompare(b.id));
if (new Set(obligations.map((row) => row.id)).size !== obligations.length) throw new Error("Duplicate manifest obligation ID");
const counts = Object.fromEntries([...new Set(obligations.map((row) => row.category))].sort().map((category) => [category, obligations.filter((row) => row.category === category).length]));
const manifest = {
  schema_revision: "pure-fiction-content-manifest-v1",
  snapshot: "Version 4.4",
  access_boundary: "2026-07-30",
  selectors: {
    schedule_id: activeSchedule.ID,
    group_id: activeGroup.GroupID,
    stage_ids: mazes.map((row) => row.ID),
    tierce_id: tierce.PHFMCACHFIJ,
    stage_config_ids: stageIds,
    scheduled_unreleased_group_id: scheduledGroup.GroupID
  },
  closure: {
    enemy_variant_ids: [...enemyIds].sort((a, b) => a - b),
    enemy_template_ids: templateIds,
    enemy_skill_ids: skillIds,
    enemy_character_config_paths: characterConfigPaths,
    enemy_ai_paths: aiPaths,
    enemy_ability_paths: selectedEnemyAbilityPaths,
    enemy_status_ids: statusRows.map((row) => row.StatusID).sort((a, b) => a - b),
    maze_buff_ids: buffIds,
    exact_zero_families: ["blessing", "curio", "occurrence", "event_choice", "service", "currency", "shop"]
  },
  counts,
  obligation_count: obligations.length,
  obligations
};
manifest.manifest_digest = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
const canonical = `${JSON.stringify(manifest, null, 2)}\n`;
if (check) {
  if (readFileSync(output, "utf8") !== canonical) throw new Error("Pure Fiction manifest drift");
} else writeFileSync(output, canonical);
console.log(`Pure Fiction manifest verified: ${obligations.length} obligations, ${stageRows.length} stages, ${enemyRows.length} enemy variants`);
