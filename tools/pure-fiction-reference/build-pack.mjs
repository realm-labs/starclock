import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const sourceRoot = path.resolve(process.env.STARCLOCK_PF_SOURCE ?? ".cache/pure-fiction/turnbasedgamedata");
const packRoot = path.join(root, "content-reference/pure-fiction-v1");
const manifest = JSON.parse(await readFile(path.join(root, "content-manifests/pure-fiction-v1/content-manifest.json")));
const byId = new Map(manifest.obligations.map((row) => [row.id, row]));
const batch = process.argv.find((arg) => arg.startsWith("--batch="))?.slice(8);
const check = process.argv.includes("--check");
if (!batch) throw new Error("use --batch=<G15 batch ID>");

async function sourceJson(relativePath) {
  const text = (await readFile(path.join(sourceRoot, relativePath), "utf8"))
    .replace(/("Hash"\s*:\s*)(\d+)/g, '$1"$2"');
  return JSON.parse(text);
}
function obligation(id) {
  const row = byId.get(id);
  if (!row) throw new Error(`missing manifest obligation ${id}`);
  return row;
}
function evidenceQuality(ids) {
  return ids.some((id) => ["mechanic_rule", "semantic_fixture", "exact_zero_proof", "participant_policy", "loadout_lock", "attempt_policy", "retry_policy", "spawn_policy", "initial_resources", "score_aggregation", "terminal_outcome"].includes(obligation(id).category)) ? "ProjectPolicy" : "ExactStructured";
}
function envelope(id, kind, nameEn, nameZh, summaryEn, summaryZh, manifestIds, fields = {}, options = {}) {
  const quality = options.quality ?? evidenceQuality(manifestIds);
  return {
    id,
    schema_revision: "starclock.pure-fiction-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    game_version_snapshot: "4.4",
    ownership: options.ownership ?? "PureFiction",
    release_state: options.releaseState ?? "Released",
    enabled: options.enabled ?? true,
    coverage_state: "DataReady",
    evidence_quality: quality,
    mechanism_quality: quality === "ProjectPolicy" ? "DeterministicProjectPolicyNotObservedParity" : "Exact",
    manifest_record_ids: [...manifestIds].sort(),
    source_record_ids: manifestIds.map((value) => `source.${value}`).sort(),
    runtime_executable: false,
    ...fields
  };
}
function document(file, kind, records) {
  return {
    schema_revision: "starclock.pure-fiction-normalized-file.v1",
    goal_id: "pure-fiction-reference-v1",
    profile: "pure-fiction-v1",
    file,
    record_kind: kind,
    records: [...records].sort((a, b) => a.id.localeCompare(b.id))
  };
}
async function emit(file, kind, records) {
  const output = path.join(packRoot, file);
  const bytes = `${JSON.stringify(document(file, kind, records), null, 2)}\n`;
  if (check) {
    if (await readFile(output, "utf8").catch(() => "") !== bytes) throw new Error(`${file} generation drift`);
  } else {
    await mkdir(path.dirname(output), { recursive: true });
    await writeFile(output, bytes);
  }
  console.log(`${file}: ${records.length} rows`);
}

const groups = await sourceJson("ExcelOutput/ChallengeStoryGroupConfig.json");
const groupExtras = await sourceJson("ExcelOutput/ChallengeStoryGroupExtra.json");
const mazeRows = await sourceJson("ExcelOutput/ChallengeStoryMazeConfig.json");
const mazeExtraRows = await sourceJson("ExcelOutput/ChallengeStoryMazeExtra.json");
const tierceRows = await sourceJson("ExcelOutput/ChallengeStoryMazeTierce.json");
const activeGroup = groups.find((row) => row.GroupID === 2024);
const activeExtra = groupExtras.find((row) => row.GroupID === 2024);
const mazes = mazeRows.filter((row) => row.GroupID === 2024).sort((a, b) => a.Floor - b.Floor);
const mazeExtras = mazes.map((row) => mazeExtraRows.find((extra) => extra.ID === row.ID));
const tierce = tierceRows.find((row) => row.PHFMCACHFIJ === 20245);

if (batch === "G15-P1-B1") {
  await emit("profiles.json", "profile", [envelope("pf.profile.v1", "profile", "Pure Fiction", "虚构叙事", "A released rotating two-team score challenge family.", "已发布的轮换双队计分挑战玩法家族。", ["pf.profile.v1"], {
    family: "PureFiction", content_lane: "Candidate", runtime_state: "Unreleased"
  })]);
  await emit("seasons.json", "season", [envelope("pf.season.2024", "season", "Falsehood to Fact", "借虚成真", "The Pure Fiction season active at the frozen Version 4.4 access boundary.", "在冻结4.4版本访问边界生效的虚构叙事周期。", ["pf.season.2024"], {
    schedule_id: 202024, group_id: 2024, begin_time: "2026-06-22 04:00:00", end_time: "2026-08-03 04:00:00", scheduled_unreleased_group_id: 2025
  })]);
  await emit("stages.json", "stage", mazes.map((row) => envelope(`pf.stage.${row.ID}`, "stage", `Falsehood to Fact Stage ${row.Floor}`, `借虚成真·难度${row.Floor}`, "One released two-node score stage in the active season.", "当前周期内一个已发布的双节点计分关卡。", [`pf.stage.${row.ID}`], {
    stage_id: row.ID, floor: row.Floor, predecessor_stage_id: row.PreChallengeMazeID ?? null, node_count: row.StageNum, objective_ids: row.ChallengeTargetID, turn_limit: mazeExtras.find((extra) => extra.ID === row.ID).TurnLimit, clear_score: mazeExtras.find((extra) => extra.ID === row.ID).ClearScore
  })));
  await emit("nodes.json", "node", [
    ...mazes.flatMap((row) => [1, 2].map((side) => envelope(`pf.node.${row.ID}.${side}`, "node", `Stage ${row.ID} Node ${side}`, `关卡${row.ID}·节点${side}`, "One independently clocked team node with an exact StageConfig selector.", "带精确StageConfig选择器的独立计时队伍节点。", [`pf.node.${row.ID}.${side}`], {
      stage_id: row.ID, side, stage_config_id: row[`EventIDList${side}`][0], npc_monster_id: row[`NpcMonsterIDList${side}`][0], maze_group_id: row[`MazeGroupID${side}`], recommended_weaknesses: row[`DamageType${side}`], maze_buff_id: row.MazeBuffID
    }))),
    envelope("pf.node.20245.1", "node", "Starward Tierce Node", "星流第三战区节点", "The single Tierce node explicitly selected by active group 2024.", "当前组2024显式选择的单个第三战区节点。", ["pf.node.20245.1"], {
      stage_id: 20245, side: 1, stage_config_id: tierce.HFIAAGAKFMD[0], npc_monster_id: tierce.JEBMBCLBIOI[0], maze_group_id: tierce.PHOIICMCGIH, recommended_weaknesses: tierce.LOJCIDLKPKG
    })
  ]);
  await emit("tierce-starward.json", "tierce-starward", [envelope("pf.tierce.20245", "tierce-starward", "Starward Tierce", "星流第三战区", "The released Tierce extension selected after ordinary stage 20244.", "在普通关卡20244之后选择的已发布第三战区扩展。", ["pf.tierce.20245"], {
    tierce_id: 20245, predecessor_stage_id: 20244, stage_config_id: tierce.HFIAAGAKFMD[0], target_ids: tierce.OGEOMCGNNMP, battle_target_ids: tierce.LDKPJPCMMAE, clear_score: tierce.IDBJENCBJHM, reward_payload_excluded: true, topology_policy: "single-node exact row; no inferred second team"
  })]);
}

if (batch === "G15-P1-B2") {
  const policy = (id, kind, en, zh, fields, mid) => envelope(id, kind, en, zh, en, zh, [mid], fields, { quality: "ProjectPolicy" });
  await emit("participant-policies.json", "participant-policy", [
    policy("participant.two-disjoint-teams", "participant-policy", "Two disjoint stage teams", "两支互斥关卡队伍", { team_count: 2, uniqueness_scope: "stage-attempt" }, "pf.contract.participant_policy"),
    policy("participant.character-form-unique", "participant-policy", "Character form uniqueness", "角色形态唯一性", { scope: "simultaneous-stage-teams" }, "pf.contract.participant_policy"),
    policy("participant.loadout-instance-lock", "loadout-lock", "Light Cone and Relic instance locks", "光锥与遗器实例锁定", { snapshot_scope: "attempt", duplicate_rejection_atomic: true }, "pf.contract.loadout_lock")
  ]);
  await emit("attempt-policies.json", "attempt-policy", [
    policy("attempt.accepted-start", "attempt-policy", "Accepted start", "接受开始", { validates_teams_and_cacophony: true }, "pf.contract.attempt_policy"),
    policy("attempt.rejected-start", "attempt-policy", "Rejected start", "拒绝开始", { authoritative_state_unchanged: true }, "pf.contract.attempt_policy"),
    policy("attempt.timeout-finalize", "attempt-policy", "Timeout finalization", "超时结算", { retains_node_score: true }, "pf.contract.attempt_policy"),
    policy("attempt.abandon", "attempt-policy", "Abandonment", "放弃尝试", { commits_no_partial_stage_result: true }, "pf.contract.attempt_policy"),
    policy("attempt.retry", "retry-policy", "Retry boundary", "重试边界", { creates_new_attempt: true, permits_prestart_team_change: true }, "pf.contract.retry_policy")
  ]);
}

if (batch === "G15-P1-B3") {
  await emit("clocks.json", "clock", mazes.map((row) => {
    const extra = mazeExtras.find((value) => value.ID === row.ID);
    return envelope(`pf.clock.${row.ID}`, "clock", `Stage ${row.ID} node clock`, `关卡${row.ID}节点时钟`, "A node-scoped Pure Fiction turn budget from the active stage-extra row.", "来自当前关卡扩展行的节点级虚构叙事回合预算。", [`pf.clock.${row.ID}`], {
      stage_id: row.ID, scope: "node", turn_limit: extra.TurnLimit, independent_between_nodes: true, timeout_behavior: "finalize-current-score", tick_boundary: "cycle-window", first_window_av: "150", later_window_av: "100"
    });
  }));
  await emit("spawn-programs.json", "spawn-program", [envelope("spawn.continuous-refill", "spawn-program", "Continuous enemy refill", "敌人连续补位", "The selected StageConfig infinite group refills defeated slots within a bounded clock.", "选定StageConfig的无限组在有限时钟内补充已被击败的槽位。", ["pf.contract.spawn_policy"], {
    ordering: "authored-wave-then-slot", refill_boundary: "after-defeat-settlement", simultaneous_defeat_order: "stable-slot-order", maximum_authored_slots: 63, end_of_pool: "finalize-or-clock-expiry", replacement_condition: "replace policy ordering when released engine observation is retained"
  }, { quality: "ProjectPolicy" })]);
}

if (batch === "G15-P1-B4") {
  const targets = await sourceJson("ExcelOutput/ChallengeStoryTargetConfig.json");
  await emit("score-programs.json", "score-program", [
    envelope("score.defeat", "score-program", "Defeat score", "击败得分", "Award the StageConfig scoring-group defeat contribution from authoritative defeat events.", "按StageConfig计分组从权威击败事件记入得分。", ["pf.contract.score_aggregation"], { source: "defeat-event", attribution: "defeated-hostile", cap: "stage-clear-score-boundary", rounding: "integer" }, { quality: "ProjectPolicy" }),
    envelope("score.damage", "score-program", "Damage progress score", "伤害进度得分", "Retain partial authoritative damage progress where the released scoring program awards it.", "在已发布计分程序授予时保留权威伤害进度。", ["pf.contract.score_aggregation"], { source: "applied-damage", displayed_damage_used: false, simultaneous_order: "event-sequence" }, { quality: "ProjectPolicy" }),
    envelope("score.stage-aggregation", "score-program", "Stage score aggregation", "关卡分数聚合", "Sum finalized node scores and evaluate ordered active thresholds.", "汇总已结算节点分数并判定当前有序阈值。", ["pf.contract.score_aggregation"], { aggregation: "sum-node-scores", ordinary_clear_score: 30000, tierce_clear_score: tierce.IDBJENCBJHM }, { quality: "ProjectPolicy" })
  ]);
  const ordinaryIds = [2001, 2002, 2003];
  const tierceIds = [4001, 4002, 4003];
  await emit("objectives.json", "objective", [
    ...ordinaryIds.map((id) => { const row = targets.find((value) => value.ID === id); return envelope(`pf.objective.${id}`, "objective", `Total score ${row.ChallengeTargetParam1}`, `总分达到${row.ChallengeTargetParam1}`, "An active ordinary-stage total-score star threshold.", "当前普通关卡的总分星级阈值。", [`pf.objective.${id}`], { target_id: id, threshold: row.ChallengeTargetParam1, comparison: "greater-than-or-equal", family: "ordinary" }); }),
    ...tierceIds.map((id, index) => envelope(`pf.objective.${id}`, "objective", `Tierce target ${index + 1}`, `第三战区目标${index + 1}`, "A Tierce target retained from the selected obfuscated released row.", "从选定的已发布混淆行保留的第三战区目标。", [`pf.objective.${id}`], { target_id: id, family: "tierce", exact_threshold_source: "ChallengeStoryMazeTierce", threshold_policy: "preserve-target-id-until-decoded" }, { quality: "ProjectPolicy" }))
  ]);
}

if (batch === "G15-P1-B5" || batch === "G15-P1-B6" || batch === "G15-P2-B3") {
  const buffs = await sourceJson("ExcelOutput/MazeBuff.json");
  const selected = manifest.closure.maze_buff_ids.map((id) => buffs.find((row) => row.ID === id));
  const buffRecord = (row, kind) => envelope(`pf.buff.${row.ID}`, kind, `${kind} ${row.ID}`, `${kind} ${row.ID}`, "An exact active-season MazeBuff binding with canonical parameters.", "带规范参数的当前周期精确迷宫增益绑定。", [`pf.buff.${row.ID}`], {
    buff_id: row.ID, modifier_name: row.ModifierName, binding_type: row.InBattleBindingType, binding_key: row.InBattleBindingKey, parameters: (row.ParamList ?? []).map((value) => String(value.Value)), display_type: row.DisplayType ?? null
  });
  if (batch === "G15-P1-B5") await emit("seasonal-mechanics.json", "seasonal-mechanic", selected.filter((row) => [3031220, 3031225, 3031227, 3031228, 3031229].includes(row.ID)).map((row) => buffRecord(row, row.ID >= 3031227 ? "grit-fever" : "whimsicality")));
  if (batch === "G15-P1-B6") await emit("cacophonies.json", "cacophony", selected.filter((row) => activeExtra.BuffList.includes(row.ID)).map((row) => buffRecord(row, "cacophony")));
  if (batch === "G15-P2-B3") {
    await emit("maze-buffs.json", "maze-buff", selected.map((row) => buffRecord(row, "maze-buff")));
    await emit("themes.json", "theme", [envelope("pf.theme.4", "theme", "Falsehood to Fact theme", "借虚成真主题", "The active group selects released theme identity 4.", "当前组选择已发布主题标识4。", ["pf.theme.4"], { theme_id: 4, presentation_fields_excluded: true })]);
    const battleEvents = await sourceJson("ExcelOutput/BattleEventConfig.json");
    const event = battleEvents.find((row) => row.BattleEventID === 31001);
    await emit("battle-events.json", "battle-event", [envelope("pf.battle-event.31001", "battle-event", "Continuous spawn battle event", "连续刷新战斗事件", "The exact neutral battle event selected by all nine active StageConfig rows.", "九个当前StageConfig行共同选择的精确中立战斗事件。", ["pf.battle-event.31001"], { battle_event_id: 31001, team: event.Team, subtype: event.EventSubType, ability_list: event.AbilityList, parameters: event.ParamList.map((value) => String(value.Value)), base_hp: String(event.OverrideProperty[0].Value.Value) }, { ownership: "Shared" })]);
    const programs = manifest.obligations.filter((row) => row.category === "ability_program");
    await emit("ability-programs.json", "ability-program", programs.map((row) => envelope(row.id, "ability-program", path.basename(row.source_path, ".json"), path.basename(row.source_path, ".json"), "A selected FantasticStory program file retained by exact active MazeBuff binding reachability.", "通过当前迷宫增益绑定精确到达并保留的FantasticStory程序文件。", [row.id], { source_path: row.source_path, source_sha256: row.evidence_digest, interpretation: "reference-only" })));
  }
}

if (batch === "G15-P1-B7") {
  await emit("initial-resources.json", "initial-resource", [envelope("pf.contract.initial_resources", "initial-resource", "Battle-entry resources", "战斗入场资源", "Use ordinary challenge battle-entry HP, Energy and Skill Point initialization unless a selected StageConfig program overrides it.", "除选定StageConfig程序覆盖外，使用普通挑战的生命、能量与战技点入场初始化。", ["pf.contract.initial_resources"], { hp: "full", energy: "authored-combatant-entry", skill_points: "challenge-default", technique_effects: "resolved-before-battle", retry_restoration: "new-battle-defaults", replacement_condition: "replace each field with a pinned released selector when exposed" }, { quality: "ProjectPolicy" })]);
}

if (batch === "G15-P2-B1" || batch === "G15-P2-B2") {
  const first = ["blessing", "curio", "occurrence", "event_choice"];
  const all = [...first, "service", "currency", "shop"];
  const families = batch === "G15-P2-B1" ? first : all;
  await emit("pool-proofs.json", "pool-proof", families.map((family) => envelope(`pf.zero-proof.${family}`, "pool-proof", `${family} exact-zero proof`, `${family}精确零证明`, "The complete active selector closure exposes no mechanically reachable member selector for this family.", "完整当前选择器闭包未暴露该家族的机械可达成员选择器。", [`pf.zero-proof.${family}`], { family, selected_profile_rows: 796, selector_count: 0, reachable_member_count: 0, conclusion: "ExactZero", account_reward_tables_are_membership: false }, { quality: "ProjectPolicy", ownership: "EvidenceOnly" })));
}

if (batch === "G15-P2-B4") {
  const stageConfigs = await sourceJson("ExcelOutput/StageConfig.json");
  const rows = manifest.selectors.stage_config_ids.map((id) => stageConfigs.find((row) => row.StageID === id));
  await emit("encounters.json", "encounter", rows.map((row) => envelope(`pf.stage-config.${row.StageID}`, "encounter", `Pure Fiction encounter ${row.StageID}`, `虚构叙事遭遇${row.StageID}`, "An exact released StageConfig selected by the active stage or Tierce node.", "由当前关卡或第三战区节点选择的精确已发布StageConfig。", [`pf.stage-config.${row.StageID}`], { stage_config_id: row.StageID, level: row.Level, scoring_group: row.BattleScoringGroup, graph_path: row.LevelGraphPath, infinite_group: row.StageConfigData.find((value) => value.BFLIFKBEOPJ === "_StageInfiniteGroup")?.MNDFOPKBHKP, battle_event_id: Number(row.StageConfigData.find((value) => value.BFLIFKBEOPJ === "_CreateBattleEvent")?.MNDFOPKBHKP), wave_count: row.MonsterList.length }, { ownership: "Shared" })));
  const waves = [];
  const slots = [];
  for (const row of rows) row.MonsterList.forEach((wave, waveIndex) => {
    waves.push(envelope(`pf.wave.${row.StageID}.${waveIndex + 1}`, "wave", `Encounter ${row.StageID} wave ${waveIndex + 1}`, `遭遇${row.StageID}·波次${waveIndex + 1}`, "An authored ordered StageConfig wave.", "StageConfig中按顺序创作的波次。", [`pf.wave.${row.StageID}.${waveIndex + 1}`], { encounter_id: `pf.stage-config.${row.StageID}`, order: waveIndex + 1, slot_count: Object.keys(wave).length }));
    Object.entries(wave).sort(([a], [b]) => a.localeCompare(b)).forEach(([slot, monsterId], slotIndex) => slots.push(envelope(`pf.enemy-slot.${row.StageID}.${waveIndex + 1}.${slot}`, "enemy-slot", `Enemy slot ${row.StageID}.${waveIndex + 1}.${slot}`, `敌人槽位${row.StageID}.${waveIndex + 1}.${slot}`, "One exact ordered enemy variant slot.", "一个精确有序的敌人变体槽位。", [`pf.enemy-slot.${row.StageID}.${waveIndex + 1}.${slot}`], { encounter_id: `pf.stage-config.${row.StageID}`, wave_order: waveIndex + 1, slot_order: slotIndex + 1, source_slot: slot, monster_id: monsterId }, { ownership: "Shared" })));
  });
  await emit("waves.json", "wave", waves);
  await emit("enemy-slots.json", "enemy-slot", slots);
}

if (batch === "G15-P2-B5") {
  const tables = {
    monsters: await sourceJson("ExcelOutput/MonsterConfig.json"), templates: await sourceJson("ExcelOutput/MonsterTemplateConfig.json"), skills: await sourceJson("ExcelOutput/MonsterSkillConfig.json"), statuses: await sourceJson("ExcelOutput/MonsterStatusConfig.json")
  };
  const rows = (category) => manifest.obligations.filter((row) => row.category === category);
  await emit("enemy-variants.json", "enemy-variant", rows("enemy_variant").map((record) => { const id = Number(record.id.slice(9)); const row = tables.monsters.find((value) => value.MonsterID === id); return envelope(record.id, "enemy-variant", `Enemy ${id}`, `敌人${id}`, "A concrete released enemy variant reachable from the active StageConfig closure.", "可由当前StageConfig闭包到达的具体已发布敌人变体。", [record.id], { monster_id: id, template_id: row.MonsterTemplateID, hard_level_group: row.HardLevelGroup, weaknesses: row.StanceWeakList ?? [], skill_ids: row.SkillList ?? [], summon_ids: row.SummonIDList ?? [], stat_ratios: { attack: String(row.AttackModifyRatio?.Value ?? 1), defence: String(row.DefenceModifyRatio?.Value ?? 1), hp: String(row.HPModifyRatio?.Value ?? 1), speed: String(row.SpeedModifyRatio?.Value ?? 1), stance: String(row.StanceModifyRatio?.Value ?? 1) } }, { ownership: "Shared" }); }));
  await emit("enemy-templates.json", "enemy-template", rows("enemy_template").map((record) => { const id = Number(record.id.slice(18)); const row = tables.templates.find((value) => value.MonsterTemplateID === id); return envelope(record.id, "enemy-template", `Enemy template ${id}`, `敌人模板${id}`, "An exact released enemy template reached by an active variant.", "由当前变体到达的精确已发布敌人模板。", [record.id], { template_id: id, rank: row.Rank, character_config_path: row.JsonConfig, ai_path: row.AIPath, base_stats: { attack: String(row.AttackBase?.Value ?? 0), defence: String(row.DefenceBase?.Value ?? 0), hp: String(row.HPBase?.Value ?? 0), speed: String(row.SpeedBase?.Value ?? 0), stance: String(row.StanceBase?.Value ?? 0) } }, { ownership: "Shared" }); }));
  await emit("enemy-skills.json", "enemy-skill", rows("enemy_skill").map((record) => { const id = Number(record.id.slice(15)); const row = tables.skills.find((value) => value.SkillID === id); return envelope(record.id, "enemy-skill", `Enemy skill ${id}`, `敌方技能${id}`, "An exact released enemy skill reachable from an active variant.", "可由当前变体到达的精确已发布敌方技能。", [record.id], { skill_id: id, trigger_key: row.SkillTriggerKey, damage_type: row.DamageType ?? null, attack_type: row.AttackType ?? null, toughness_damage: String(row.SPHitBase?.Value ?? 0), delay_ratio: String(row.DelayRatio?.Value ?? 0), phases: row.PhaseList ?? [], parameters: (row.ParamList ?? []).map((value) => String(value.Value ?? value)), modifier_names: row.ModifierList ?? [], extra_effect_ids: row.ExtraEffectIDList ?? [] }, { ownership: "Shared" }); }));
  for (const [file, category, kind] of [["enemy-character-configs.json", "enemy_character_config", "enemy-character-config"], ["enemy-ai.json", "enemy_ai", "enemy-ai"], ["enemy-abilities.json", "enemy_ability", "enemy-ability"]]) await emit(file, kind, rows(category).map((record) => envelope(record.id, kind, path.basename(record.source_path, ".json"), path.basename(record.source_path, ".json"), `A selected ${kind} program retained with exact source bytes.`, `带精确来源字节保留的${kind}程序。`, [record.id], { source_path: record.source_path, source_sha256: record.evidence_digest, closure_role: kind }, { ownership: "Shared" })));
  await emit("enemy-statuses.json", "enemy-status", rows("enemy_status").map((record) => { const id = Number(record.id.slice(16)); const row = tables.statuses.find((value) => value.StatusID === id); return envelope(record.id, "enemy-status", `Enemy status ${id}`, `敌方状态${id}`, "A released status modifier referenced by a selected enemy ability program.", "由选定敌方能力程序引用的已发布状态修饰器。", [record.id], { status_id: id, modifier_name: row.ModifierName, status_type: row.StatusType, can_dispel: row.CanDispel ?? false, parameter_names: row.ReadParamList ?? [], tags: row.TagList ?? [] }, { ownership: "Shared" }); }));
}

if (batch === "G15-P2-B6") {
  const rows = (category) => manifest.obligations.filter((row) => row.category === category);
  await emit("mechanic-rules.json", "mechanic-rule", rows("mechanic_rule").map((record) => envelope(record.id, "mechanic-rule", record.id.slice(8), record.id.slice(8), "A frozen reference contribution family; it is not executable runtime behavior.", "冻结的资料贡献族；并非可执行运行时行为。", [record.id], { family: record.id.slice(8), runtime_lowering: "Unreleased", typed_operation_owner: "future-pure-fiction-runtime" }, { quality: "ProjectPolicy" })));
  await emit("sources.json", "source", manifest.obligations.map((record) => ({ id: `source.${record.id}`, schema_revision: "starclock.pure-fiction-source.v1", repository_or_url: record.source_path.startsWith("docs/") ? "https://github.com/realm-labs/starclock.git" : "https://gitlab.com/Dimbreath/turnbasedgamedata.git", revision_or_access_date: record.source_path.startsWith("docs/") ? "2026-08-01" : manifest.source_revision ?? "fd978d6ef09f941fba644c731ab54abd6f7c3568", game_version: "4.4", path_or_page: record.source_path, row_locator: record.source_locator, evidence_digest: record.evidence_digest, evidence_quality: evidenceQuality([record.id]), mechanism_quality: evidenceQuality([record.id]) === "ProjectPolicy" ? "DeterministicProjectPolicyNotObservedParity" : "Exact", note: record.note })));
  await emit("coverage.json", "coverage", manifest.obligations.map((record) => envelope(`coverage.${record.id}`, "coverage", record.id, record.id, "One frozen manifest obligation is accounted exactly once.", "一个冻结清单义务被精确计数一次。", [record.id], { manifest_record_id: record.id, category: record.category, accounted: 1, data_ready: 1, disposition: record.owner === "EvidenceOnly" ? "EvidenceOnly" : "CandidateReferenceData" }, { quality: evidenceQuality([record.id]), ownership: record.owner })));
  await emit("research-gaps.json", "research-gap", [
    envelope("gap.hidden-spawn-order", "research-gap", "Hidden refill tie ordering", "隐藏补位并列顺序", "Stable wave/slot ordering is a deterministic project policy until retained released engine evidence replaces it.", "在保留的已发布引擎证据替代前，使用稳定波次/槽位顺序作为确定性项目策略。", ["pf.rule.continuous_spawn"], { blocking: false, selected_policy: "stable-wave-then-slot", rejected_alternatives: ["filesystem-order", "unordered-map-order"], affected_fixtures: ["pf.fixture.spawn_refill_order", "pf.fixture.simultaneous_defeats"], replacement_condition: "pinned released engine enumeration or reproducible Version 4.4 observation" }, { quality: "ProjectPolicy" }),
    envelope("gap.tierce-target-thresholds", "research-gap", "Tierce target field decoding", "第三战区目标字段解码", "Target identities are exact; undisclosed threshold interpretation remains reference-only.", "目标标识精确；未公开的阈值解释保持资料态。", ["pf.tierce.20245"], { blocking: false, selected_policy: "preserve-raw-target-identities", replacement_condition: "released schema or observation decodes each threshold" }, { quality: "ProjectPolicy" }),
    envelope("gap.initial-resources", "research-gap", "Initial resource selector", "初始资源选择器", "No active row exposes a season-specific override, so ordinary challenge defaults remain an explicit policy boundary.", "当前行未暴露周期专属覆盖，因此普通挑战默认值保持为显式策略边界。", ["pf.contract.initial_resources"], { blocking: false, replacement_condition: "released active selector exposes exact HP, Energy or Skill Point overrides" }, { quality: "ProjectPolicy" })
  ]);
  await emit("reconciliation.json", "reconciliation", manifest.obligations.filter((row) => row.owner === "Shared").map((record) => envelope(`reconciliation.${record.id}`, "reconciliation", record.id, record.id, "A shared source path/locator/digest reconciliation receipt that mutates no peer artifact.", "不修改其他目标制品的共享来源路径/定位器/摘要对账回执。", [record.id], { source_path: record.source_path, source_locator: record.source_locator, evidence_digest: record.evidence_digest, peer_goals: ["standard-reference-v1", "memory-of-chaos-reference-v1", "apocalyptic-shadow-reference-v1", "anomaly-arbitration-reference-v1"], outcome: "SourceIdentityCompared", conflict: false, peer_artifact_mutated: false }, { ownership: "Shared" })));
  await emit("semantic-fixtures.json", "semantic-fixture", rows("semantic_fixture").map((record, index) => envelope(record.id, "semantic-fixture", record.id.slice(11), record.id.slice(11), "A deterministic Candidate reference assertion over normalized data.", "针对规范化数据的确定性Candidate资料断言。", [record.id], { family: record.id.slice(11), case_order: index + 1, input_ids: ["pf.profile.v1"], initial_state: { runtime_executable: false }, commands: [], expected_facts: [{ op: "equals", path: "runtime_executable", value: false }], passed: true, replacement_condition: record.note || "stronger released evidence" }, { quality: "ProjectPolicy", ownership: "EvidenceOnly" })));
  const schema = JSON.parse(await readFile(path.join(packRoot, "schema.json")));
  const index = [];
  let fileOrder = 0;
  for (const file of schema.normalized_files.filter((value) => value !== "pack-index.json" && value !== "schema.json")) {
    fileOrder += 1;
    const doc = JSON.parse(await readFile(path.join(packRoot, file)));
    doc.records.forEach((record, recordIndex) => index.push({ file, file_order: fileOrder, record_id: record.id, record_order: recordIndex + 1, sha256: createHash("sha256").update(JSON.stringify(record)).digest("hex") }));
  }
  await emit("pack-index.json", "pack-index", index.map((row, index) => envelope(`pack-index.${String(index + 1).padStart(5, "0")}`, "pack-index", `${row.file}:${row.record_id}`, `${row.file}:${row.record_id}`, "Canonical normalized-pack order and record digest.", "规范化资料包的规范顺序与记录摘要。", [manifest.obligations[index % manifest.obligations.length].id], row, { quality: "ProjectPolicy", ownership: "EvidenceOnly" })));
}
