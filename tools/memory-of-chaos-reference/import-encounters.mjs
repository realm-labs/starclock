#!/usr/bin/env node

import {
  assert,
  assertSource,
  digest,
  manifest,
  normalizedFile,
  record,
  source,
  sourceRecordId,
  structuredRef,
  textRef,
  writeCanonical,
  writeText,
} from "./lib.mjs";

const check = process.argv.includes("--check");
assertSource();
const [mazes, tierces, stageConfigs, textZh, textEn] = await Promise.all([
  source("ExcelOutput/ChallengeMazeConfig.json"),
  source("ExcelOutput/ChallengeMazeTierce.json"),
  source("ExcelOutput/StageConfig.json"),
  source("TextMap/TextMapCHS.json"),
  source("TextMap/TextMapEN.json"),
]);
const ordinary = mazes.filter(({ ID }) => ID >= 5201 && ID <= 5212)
  .sort((left, right) => left.ID - right.ID);
const tierce = tierces.find(({ PHFMCACHFIJ }) => PHFMCACHFIJ === 5213);
assert(ordinary.length === 12 && tierce, "active encounter selector drift");

const bindingByStage = new Map();
for (const maze of ordinary) {
  for (const stageId of maze.EventIDList1) {
    assert(!bindingByStage.has(stageId), `duplicate stage selector ${stageId}`);
    bindingByStage.set(stageId, { domain: "OrdinaryNode", mode_stage_id: maze.ID, node_index: 1, team_slot: 1 });
  }
  for (const stageId of maze.EventIDList2) {
    assert(!bindingByStage.has(stageId), `duplicate stage selector ${stageId}`);
    bindingByStage.set(stageId, { domain: "OrdinaryNode", mode_stage_id: maze.ID, node_index: 2, team_slot: 2 });
  }
}
for (const stageId of tierce.HFIAAGAKFMD) {
  assert(!bindingByStage.has(stageId), `duplicate Tierce selector ${stageId}`);
  bindingByStage.set(stageId, { domain: "TierceExtension", mode_stage_id: tierce.PHFMCACHFIJ, node_index: 1, team_slot: null });
}
assert(bindingByStage.size === 25, "selected encounter denominator drift");
const selectedStages = stageConfigs.filter(({ StageID }) => bindingByStage.has(StageID))
  .sort((left, right) => left.StageID - right.StageID);
assert(selectedStages.length === 25, "selected StageConfig closure drift");

const manifestSlots = manifest.categories.encounter_enemy_slots.records;
assert(manifestSlots.length === 99, "enemy-slot denominator drift");
const manifestSlotByKey = new Map(manifestSlots.map((slot) => [
  `${slot.stage_id}:${slot.wave}:${slot.slot}`,
  slot,
]));
assert(manifestSlotByKey.size === 99, "duplicate manifest enemy-slot key");

const encounters = [];
const waves = [];
const slots = [];
for (const stage of selectedStages) {
  const binding = bindingByStage.get(stage.StageID);
  const stageManifestId = `stage-config-${stage.StageID}`;
  const nameHash = String(stage.StageName.Hash);
  const nameZh = textZh[nameHash];
  const nameEn = textEn[nameHash];
  assert(typeof nameZh === "string" && typeof nameEn === "string", `missing StageName text ${nameHash}`);
  const data = Object.fromEntries(stage.StageConfigData.map((entry) => [entry.BFLIFKBEOPJ, entry.MNDFOPKBHKP]));
  assert(data._Wave === String(stage.MonsterList.length), `wave-count binding drift ${stage.StageID}`);
  assert(data._CreateBattleEvent === "30146", `battle-event binding drift ${stage.StageID}`);
  assert(stage.Release === true, `unreleased selected StageConfig ${stage.StageID}`);
  const stageRef = structuredRef("stage_configs", stageManifestId, "Exact selected StageConfig, difficulty, level, wave and monster binding.");
  encounters.push(record({
    id: `encounter.${stage.StageID}`,
    kind: "Encounter",
    nameEn: `${binding.domain === "TierceExtension" ? "Tierce" : `Stage ${binding.mode_stage_id} node ${binding.node_index}`} encounter`,
    nameZh: `${binding.domain === "TierceExtension" ? "Tierce扩展关" : `关卡${binding.mode_stage_id}节点${binding.node_index}`}遭遇`,
    summaryEn: `Released ${stage.StageType} encounter ${stage.StageID} at level ${stage.Level} with ${stage.MonsterList.length} ordered waves.`,
    summaryZh: `已发布${stage.StageType}遭遇${stage.StageID}，等级${stage.Level}，包含${stage.MonsterList.length}个有序波次。`,
    ownership: "MemoryOfChaos",
    sourceIds: [sourceRecordId("stage_configs", stageManifestId)],
    evidence: [stageRef, textRef("zh_cn", nameHash, nameZh), textRef("en", nameHash, nameEn)],
    tags: [binding.domain === "TierceExtension" ? "tierce" : "ordinary", "encounter", "released"],
    fields: {
      upstream_stage_config_id: stage.StageID,
      mode_stage_id: binding.mode_stage_id,
      domain: binding.domain,
      node_index: binding.node_index,
      team_slot: binding.team_slot,
      stage_type: stage.StageType,
      level: stage.Level,
      hard_level_group: stage.HardLevelGroup,
      elite_group: stage.EliteGroup,
      level_graph_path: stage.LevelGraphPath,
      stage_ability_config: stage.StageAbilityConfig,
      sub_level_graphs: stage.SubLevelGraphs,
      battle_event_id: Number(data._CreateBattleEvent),
      wave_count: stage.MonsterList.length,
      wave_ids: stage.MonsterList.map((_, index) => `wave.${stage.StageID}.${index + 1}`),
      release: stage.Release,
      forbid_exit_battle: stage.ForbidExitBattle,
      monster_warning_ratio: String(stage.MonsterWarningRatio),
      source_display_text: { en: nameEn, zh_cn: nameZh },
      evidence_quality: "ExactStructured",
      mechanism_quality: "ExactRelationship",
      approximations: [],
    },
  }));

  stage.MonsterList.forEach((wave, waveIndex) => {
    const waveNumber = waveIndex + 1;
    const orderedEntries = Object.entries(wave).sort((left, right) =>
      Number(left[0].replace("Monster", "")) - Number(right[0].replace("Monster", "")));
    const waveSlotIds = [];
    for (const [slotName, monsterId] of orderedEntries) {
      const manifestSlot = manifestSlotByKey.get(`${stage.StageID}:${waveNumber}:${slotName}`);
      assert(manifestSlot, `missing manifest slot ${stage.StageID}:${waveNumber}:${slotName}`);
      assert(manifestSlot.monster_id === monsterId, `manifest monster drift ${manifestSlot.id}`);
      const slotId = `enemy-slot.${manifestSlot.id.replace("slot-", "")}`;
      waveSlotIds.push(slotId);
      slots.push(record({
        id: slotId,
        kind: "EnemySlot",
        nameEn: `Enemy slot ${manifestSlot.id.replace("slot-", "")}`,
        nameZh: `敌方槽位${manifestSlot.id.replace("slot-", "")}`,
        summaryEn: `Places enemy variant ${monsterId} in ${slotName} of wave ${waveNumber} for encounter ${stage.StageID}.`,
        summaryZh: `在遭遇${stage.StageID}第${waveNumber}波的${slotName}放置敌人变体${monsterId}。`,
        ownership: "MemoryOfChaos",
        sourceIds: [sourceRecordId("encounter_enemy_slots", manifestSlot.id)],
        evidence: [structuredRef("encounter_enemy_slots", manifestSlot.id, "Exact StageConfig wave, slot and enemy variant binding.")],
        tags: [binding.domain === "TierceExtension" ? "tierce" : "ordinary", "enemy-slot", `wave-${waveNumber}`],
        fields: {
          encounter_id: `encounter.${stage.StageID}`,
          wave_id: `wave.${stage.StageID}.${waveNumber}`,
          upstream_stage_config_id: stage.StageID,
          wave_index: waveNumber,
          slot_name: slotName,
          slot_index: Number(slotName.replace("Monster", "")),
          upstream_enemy_variant_id: monsterId,
          enemy_variant_id: `enemy-variant.${monsterId}`,
          order_key: `${String(stage.StageID).padStart(8, "0")}:${String(waveNumber).padStart(2, "0")}:${String(Number(slotName.replace("Monster", ""))).padStart(2, "0")}`,
          evidence_quality: "ExactStructured",
          mechanism_quality: "ExactRelationship",
          approximations: [],
        },
      }));
    }
    waves.push(record({
      id: `wave.${stage.StageID}.${waveNumber}`,
      kind: "EncounterWave",
      nameEn: `Encounter ${stage.StageID} wave ${waveNumber}`,
      nameZh: `遭遇${stage.StageID}第${waveNumber}波`,
      summaryEn: `Ordered wave ${waveNumber} contains ${orderedEntries.length} exact enemy slots.`,
      summaryZh: `有序波次${waveNumber}包含${orderedEntries.length}个精确敌方槽位。`,
      ownership: "MemoryOfChaos",
      sourceIds: [],
      evidence: [stageRef],
      tags: [binding.domain === "TierceExtension" ? "tierce" : "ordinary", "wave"],
      fields: {
        encounter_id: `encounter.${stage.StageID}`,
        upstream_stage_config_id: stage.StageID,
        wave_index: waveNumber,
        enemy_slot_ids: waveSlotIds,
        enemy_slot_count: waveSlotIds.length,
        next_wave_id: waveNumber < stage.MonsterList.length ? `wave.${stage.StageID}.${waveNumber + 1}` : null,
        transition_clock_rule_id: waveNumber < stage.MonsterList.length ? "clock.wave-carry" : null,
        evidence_quality: "ExactStructured",
        mechanism_quality: "ExactRelationship",
        approximations: [],
      },
    }));
  });
}

assert(encounters.length === 25 && waves.length === 50 && slots.length === 99,
  "encounter/wave/slot output denominator drift");
const claims = [...encounters, ...slots].flatMap(({ source_record_ids: sourceIds }) => sourceIds);
const expectedClaims = [
  ...manifest.categories.stage_configs.records.map(({ id }) => sourceRecordId("stage_configs", id)),
  ...manifestSlots.map(({ id }) => sourceRecordId("encounter_enemy_slots", id)),
].sort();
assert(claims.length === new Set(claims).size, "encounter obligations must be claimed exactly once");
assert(JSON.stringify([...claims].sort()) === JSON.stringify(expectedClaims), "encounter obligation coverage drift");

const encounterOutput = normalizedFile("encounters.json", "Encounter", encounters);
const waveOutput = normalizedFile("waves.json", "EncounterWave", waves);
const slotOutput = normalizedFile("enemy-slots.json", "EnemySlot", slots);
await writeCanonical("encounters.json", encounterOutput, check);
await writeCanonical("waves.json", waveOutput, check);
await writeCanonical("enemy-slots.json", slotOutput, check);
const levels = encounters.map(({ level }) => level);
await writeText(
  "evidence/memory-of-chaos-reference-v1/encounter-audit.md",
  `# Goal 17 encounter audit

- StageConfig obligations: 25/25, each claimed exactly once
- Encounter rows: 25 (24 ordinary node encounters, one Tierce encounter)
- Ordered waves: 50/50
- Enemy-slot obligations: 99/99, each claimed exactly once
- Stage levels: ${Math.min(...levels)}–${Math.max(...levels)}
- Hard-level groups: ${[...new Set(encounters.map(({ hard_level_group: value }) => value))].sort((a, b) => a - b).join(", ")}
- BattleEvent binding: 30146 on every selected StageConfig
- Encounter digest: \`${digest(encounterOutput)}\`
- Wave digest: \`${digest(waveOutput)}\`
- Enemy-slot digest: \`${digest(slotOutput)}\`
- Runtime executable rows: 0

Stage selection, difficulty, level, two-wave order and every enemy-slot binding
are ExactStructured. No slot is inferred from an ID range or unordered map.
`,
  check,
);
console.log(`Goal 17 encounters ${check ? "verified" : "generated"}: 25 encounters, 50 waves, 99 slots.`);
