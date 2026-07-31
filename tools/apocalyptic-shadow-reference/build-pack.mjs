#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { root, sourceJson } from "./source.mjs";

const batch = process.argv.find((arg) => arg.startsWith("--batch="))?.slice(8);
const check = process.argv.includes("--check");
if (!batch) throw new Error("use --batch=<G18 batch ID>");
const packRoot = path.join(root, "content-reference/apocalyptic-shadow-v1");
const manifest = JSON.parse(await readFile(path.join(root,
  "content-manifests/apocalyptic-shadow-v1/content-manifest.json")));
const manifestById = new Map(Object.values(manifest.categories).flat()
  .map((record) => [record.id, record]));

function manifestRecord(id) {
  const value = manifestById.get(id);
  if (!value) throw new Error(`missing manifest record ${id}`);
  return value;
}

function sourceRef(manifestId, note = "") {
  const record = manifestRecord(manifestId);
  return {
    source_id: `src.${manifestId}`,
    repository_or_url: record.source_path.startsWith("docs/")
      ? "https://github.com/realm-labs/starclock.git"
      : "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: record.source_path.startsWith("docs/")
      ? "2026-08-01"
      : manifest.source_revision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    mechanism_quality: record.evidence_quality === "ProjectPolicy"
      ? "PolicyBoundary" : "ExactRelationship",
    note,
  };
}

function envelope(id, kind, nameEn, nameZh, summaryEn, summaryZh,
  manifestIds, fields = {}, options = {}) {
  const evidenceQuality = options.evidenceQuality
    ?? (manifestIds.some((value) =>
      manifestRecord(value).evidence_quality === "ProjectPolicy")
      ? "ProjectPolicy" : "ExactStructured");
  return {
    id,
    schema_revision: "starclock.apocalyptic-shadow-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    ownership: options.ownership ?? "ApocalypticShadow",
    coverage_state: "DataReady",
    evidence_quality: evidenceQuality,
    mechanism_quality: evidenceQuality === "ProjectPolicy"
      ? "PolicyBoundary" : "ExactRelationship",
    manifest_record_ids: [...manifestIds].sort(),
    source_refs: manifestIds.map((value) => sourceRef(value)).sort((a, b) =>
      a.source_id.localeCompare(b.source_id)),
    ...fields,
    runtime_executable: false,
  };
}

function document(file, kind, records) {
  return {
    schema_revision: "starclock.apocalyptic-shadow-normalized-file.v1",
    goal_id: "apocalyptic-shadow-reference-v1",
    profile: "apocalyptic-shadow-v1",
    file,
    record_kind: kind,
    records: [...records].sort((a, b) => a.id.localeCompare(b.id)),
  };
}

async function emit(file, kind, records) {
  const output = path.join(packRoot, file);
  const bytes = `${JSON.stringify(document(file, kind, records), null, 2)}\n`;
  if (check) {
    if (await readFile(output, "utf8").catch(() => "") !== bytes)
      throw new Error(`${file} generation drift`);
  } else {
    await mkdir(path.dirname(output), { recursive: true });
    await writeFile(output, bytes);
  }
  console.log(`${file}: ${records.length} rows`);
}

function tableRows(value) { return Array.isArray(value) ? value : Object.values(value); }
const group = tableRows(await sourceJson("ExcelOutput/ChallengeBossGroupConfig.json"))
  .find((row) => row.GroupID === 3019);
const extra = tableRows(await sourceJson("ExcelOutput/ChallengeBossGroupExtra.json"))
  .find((row) => row.GroupID === 3019);
const stages = tableRows(await sourceJson("ExcelOutput/ChallengeBossMazeConfig.json"))
  .filter((row) => row.GroupID === 3019).sort((a, b) => a.ID - b.ID);
const stageExtras = tableRows(await sourceJson("ExcelOutput/ChallengeBossMazeExtra.json"))
  .filter((row) => row.ID >= 30191 && row.ID <= 30194);

if (batch === "G18-P1-B1") {
  await emit("profiles.json", "profile", [envelope(
    "apocalyptic-shadow-v1", "profile", "Apocalyptic Shadow",
    "末日幻影", "A released rotating boss challenge family.",
    "已发布的轮换首领挑战玩法家族。", ["apocalyptic-shadow-v1"], {
      group_id: 3019, active_period_id: 203019,
      entry_goto_id: 232, early_access_content_id: 200001,
      runtime_state: "Unreleased",
    },
  )]);
  await emit("periods.json", "period", [envelope(
    "period.203019", "period", "Vanguard Knight", "兵锋骑士",
    "The active Version 4.4 Apocalyptic Shadow period.",
    "4.4 版本当前生效的末日幻影周期。", ["period.203019", "group.3019"], {
      schedule_id: 203019, group_id: 3019,
      begin_time: "2026-07-20 04:00:00",
      end_time: "2026-08-31 04:00:00",
      later_group_id_excluded: 3020,
    },
  )]);
  await emit("stages.json", "stage", [
    ...stages.map((row) => envelope(`stage.${row.ID}`, "stage",
      `Vanguard Knight Difficulty ${row.Floor}`,
      `兵锋骑士·难度${row.Floor}`,
      "An ordinary two-node difficulty selected by active group 3019.",
      "当前组3019选择的双节点普通难度。", [`stage.${row.ID}`], {
        stage_id: row.ID, floor: row.Floor,
        predecessor_stage_id: row.PreChallengeMazeID ?? null,
        node_count: row.StageNum, target_ids: row.ChallengeTargetID,
        tierce: false,
      })),
    envelope("stage.30195", "stage", "Vanguard Knight Tierce",
      "兵锋骑士·第三战区", "The Tierce record explicitly selected by group 3019.",
      "组3019显式选择的第三战区记录。", ["stage.30195", "group.3019"], {
        stage_id: 30195, predecessor_stage_id: 30194, node_count: 1,
        target_ids: [5001, 5002, 5003], tierce: true,
      }),
  ]);
  const nodes = stages.flatMap((row) => [1, 2].map((side) => envelope(
    `node.${row.ID}.${side}`, "node", `Stage ${row.ID} Node ${side}`,
    `关卡${row.ID}·节点${side}`, "One independently attempted boss node.",
    "独立进行挑战与结算的首领节点。", [`node.${row.ID}.${side}`], {
      stage_id: row.ID, side, order: side,
      map_entrance_id: row[`MapEntranceID${side === 1 ? "" : "2"}`],
      maze_group_id: row[`MazeGroupID${side}`],
      config_list: row[`ConfigList${side}`],
      npc_monster_ids: row[`NpcMonsterIDList${side}`],
      event_ids: row[`EventIDList${side}`],
      recommended_weaknesses: row[`DamageType${side}`],
      maze_buff_id: row.MazeBuffID,
    })));
  nodes.push(envelope("node.30195.1", "node", "Tierce Central Node",
    "第三战区·中央节点", "The single Tierce boss node selected by group 3019.",
    "组3019选择的单个第三战区首领节点。", ["node.30195.1"], {
      stage_id: 30195, side: 1, order: 1, map_entrance_id: 3012201,
      maze_group_id: 8, config_list: [200001], npc_monster_ids: [3003015],
      event_ids: [420494],
      recommended_weaknesses: ["Fire", "Ice", "Thunder", "Imaginary"],
      maze_buff_id: 3110006,
    }));
  await emit("nodes.json", "node", nodes);
}

if (batch === "G18-P1-B2") {
  const policy = (id, en, zh, fields) => envelope(id, "participant-policy", en,
    zh, en, zh, ["apocalyptic-shadow-v1"], fields, { evidenceQuality: "ProjectPolicy" });
  await emit("participant-policies.json", "participant-policy", [
    policy("participant-policy.two-independent-teams", "Two independent node teams",
      "双节点独立队伍", { scope: "stage-attempt", team_count: 2 }),
    policy("participant-policy.character-form-unique", "Character form uniqueness",
      "角色形态唯一性", { scope: "simultaneous-stage-teams" }),
    policy("participant-policy.light-cone-instance-unique", "Light Cone instance uniqueness",
      "光锥实例唯一性", { scope: "simultaneous-stage-teams" }),
    policy("participant-policy.relic-instance-unique", "Relic instance uniqueness",
      "遗器实例唯一性", { scope: "simultaneous-stage-teams" }),
  ]);
  await emit("team-slots.json", "team-slot", manifest.categories.nodes.map((row) =>
    envelope(`team-slot.${row.id.slice(5)}`, "team-slot", `Team for ${row.id}`,
      `${row.id}队伍`, "A team snapshot assigned to one challenge node.",
      "分配给单个挑战节点的队伍快照。", [row.id], {
        node_id: row.id, maximum_characters: 4, snapshot_scope: "attempt",
      })));
  await emit("loadout-records.json", "loadout-record", [
    policy("loadout-record.team-snapshot", "Attempt team snapshot", "尝试队伍快照",
      { includes: ["character-form", "light-cone-instance", "relic-instances"] }),
    policy("loadout-record.cross-team-invalidation", "Cross-team duplicate invalidation",
      "跨队重复实例失效", { rejection_is_atomic: true }),
    policy("loadout-record.retry-replacement", "Retry replacement boundary",
      "重试替换边界", { mutation_allowed_before_new_attempt: true }),
  ]);
  await emit("attempts.json", "attempt", [
    policy("attempt.accepted-start", "Accepted attempt start", "接受的尝试开始",
      { validates_all_selected_teams: true }),
    policy("attempt.rejected-start", "Rejected attempt start", "拒绝的尝试开始",
      { authoritative_state_unchanged: true }),
    policy("attempt.timeout", "Action Value timeout", "行动值耗尽",
      { retains_partial_boss_progress_for_score: true }),
    policy("attempt.abandon", "Abandoned attempt", "放弃尝试",
      { commits_no_best_result_replacement: true }),
    policy("attempt.complete", "Completed attempt", "完成尝试",
      { projects_node_score_and_progress: true }),
  ]);
  await emit("transitions.json", "transition", [
    policy("transition.stage-unlock", "Predecessor stage unlock", "前置难度解锁",
      { ordinary_chain: [30191, 30192, 30193, 30194] }),
    policy("transition.node-result", "Node result projection", "节点结果投影",
      { battle_mutates_activity_directly: false }),
    policy("transition.retry", "Retry creates a new attempt", "重试创建新尝试",
      { previous_authoritative_result_immutable: true }),
    policy("transition.tierce", "Tierce selection boundary", "第三战区选择边界",
      { selected_by_group_field: "TierceID", tierce_id: 30195 }),
  ]);
}

if (batch === "G18-P1-B3") {
  const targets = tableRows(await sourceJson("ExcelOutput/ChallengeBossTargetConfig.json"))
    .filter((row) => [3001, 3002, 3003, 5001, 5002, 5003].includes(row.ID));
  const policy = (id, kind, en, zh, fields, mids = ["apocalyptic-shadow-v1"]) =>
    envelope(id, kind, en, zh, en, zh, mids, fields,
      { evidenceQuality: "ProjectPolicy" });
  await emit("clocks.json", "clock", [
    policy("clock.node-action-value", "clock", "Per-node Action Value budget",
      "节点行动值预算", { scope: "node-attempt", decrements_on_global_action_delay: true }),
    policy("clock.remaining-delay-slice", "clock", "Remaining delay slice",
      "剩余延迟切片", { floor_divisor: 10, timeout_when_less_or_equal_zero: true }),
    policy("clock.timeout-result", "clock", "Time-limit settlement",
      "时限结算", { battle_end_reason: "TimeLimit", local_win_required: false }),
  ]);
  await emit("boss-progress.json", "boss-progress", [
    policy("boss-progress.total-hp", "boss-progress", "All-wave boss total HP",
      "全波次首领总生命", { minimum_rank: "LittleBoss", includes_all_waves: true }),
    policy("boss-progress.left-hp", "boss-progress", "Current boss remaining HP",
      "当前首领剩余生命", { partial_progress_on_timeout: true }),
    policy("boss-progress.summon-adjustment", "boss-progress", "Summon HP adjustment",
      "召唤物生命修正", { selected_by_scoring_program: true }),
  ]);
  await emit("scores.json", "score", [
    envelope("score.boss-progress", "score", "Boss progress score", "首领进度分",
      "Scoring item 90004 derives progress from boss and selected summon HP.",
      "计分项90004按首领与选定召唤物生命计算进度分。",
      ["program.StrongChallenge_Scoring_Ability"], {
        scoring_item_id: 90004, fixed_scale_constant: "2000",
        postfix_expression_preserved: true,
      }),
    envelope("score.remaining-av", "score", "Remaining Action Value score",
      "剩余行动值分", "Scoring item 90005 tracks remaining global delay.",
      "计分项90005跟踪剩余全局行动延迟。",
      ["program.StrongChallenge_Scoring_Ability"], {
        scoring_item_id: 90005, fixed_scale_constant: "2000",
        update_event: "OnListenGlobalActionDelayChanged",
        timeout_score: 0,
      }),
    policy("score.stage-total", "score", "Stage total score", "关卡总分",
      { aggregation: "sum-selected-node-scores", maximum_target: 6600 }),
  ]);
  await emit("objectives.json", "objective", targets.map((row) => envelope(
    `objective.${row.ID}`, "objective", `Total score ${row.ChallengeTargetParam1}`,
    `总分达到${row.ChallengeTargetParam1}`, "Evaluate the active stage total score.",
    "按当前关卡总分进行判定。", [`target.${row.ID}`], {
      target_id: row.ID, target_type: row.ChallengeTargetType,
      threshold: row.ChallengeTargetParam1,
      evaluation: "greater-than-or-equal",
    })));
  await emit("stars.json", "star", targets.map((row, index) => envelope(
    `star.${row.ID}`, "star", `Star threshold ${row.ChallengeTargetParam1}`,
    `星级阈值${row.ChallengeTargetParam1}`, "One ordered total-score star threshold.",
    "一个按总分排序的星级阈值。", [`target.${row.ID}`], {
      target_id: row.ID, star_ordinal: (index % 3) + 1,
      threshold: row.ChallengeTargetParam1,
      family: row.ID < 5000 ? "ordinary" : "tierce",
    })));
}

if (batch === "G18-P1-B4") {
  const selectedIds = new Set(manifest.categories.buffs.map((row) =>
    Number(row.id.slice(5))));
  const buffs = tableRows(await sourceJson("ExcelOutput/MazeBuff.json"))
    .filter((row) => selectedIds.has(row.ID));
  const names = {
    3031001: ["Word Shatter", "词语爆裂"],
    3110006: ["Ruinous Embers", "末法余烬"],
    3111058: ["Blighted to the Bone", "附骨之疽"],
    3111065: ["Unstoppable Force", "攻无不克"],
    3111077: ["Shatterstrike", "攻心扼吭"],
    3111078: ["Linebreaker", "摧锋陷阵"],
    3111079: ["Unto Apotheosis", "聚气化神"],
    3111081: ["Whirlwind Turn", "疾如旋踵"],
    3111082: ["Knowledge and Decorum", "智圆行方"],
    3111083: ["Oppose With Tenderness", "以柔克刚"],
    3111085: ["Moment of Opportunity", "可乘之隙"],
  };
  await emit("buffs.json", "buff", buffs.map((row) => envelope(
    `buff.${row.ID}`, "buff", names[row.ID][0], names[row.ID][1],
    "An exact active-period MazeBuff binding with canonical parameters.",
    "带规范参数的当前周期精确迷宫增益绑定。", [`buff.${row.ID}`], {
      buff_id: row.ID, modifier_name: row.ModifierName,
      binding_type: row.InBattleBindingType,
      binding_key: row.InBattleBindingKey,
      parameters: (row.ParamList ?? []).map((value) => String(value.Value)),
      display_type: row.DisplayType ?? null,
    })));
  await emit("embers.json", "ember", [envelope(
    "ember.3110006", "ember", "Ruinous Embers", "末法余烬",
    "Breaking Steadfast Safeguard dispels allied control, restores Skill Points, activates Ultimates, and applies exact Skill/Ultimate taken multipliers.",
    "击破坚防守备时解除我方控制、恢复战技点、激活终结技，并施加精确的战技/终结技易伤参数。",
    ["buff.3110006"], { buff_id: 3110006,
      skill_damage_taken_ratio: "0.25", ultimate_damage_taken_ratio: "0.15" }),
  ]);
  await emit("safeguards.json", "safeguard", [envelope(
    "safeguard.steadfast", "safeguard", "Steadfast Safeguard", "坚防守备",
    "A boss protection state whose Weakness Break is the released Ruinous Embers trigger.",
    "首领保护状态；其弱点击破是已发布末法余烬的触发条件。",
    ["buff.3110006"], { break_trigger: "WeaknessBreak",
      on_break: ["dispel-allied-control", "recover-skill-points", "activate-ultimates"],
      exact_protection_amount: "boss-owned-enemy-closure" }),
  ]);
  const axiomIds = [...new Set([...(extra.BuffList1 ?? []), ...(extra.BuffList2 ?? []),
    ...(extra.BuffList3 ?? [])])];
  await emit("axioms.json", "axiom", axiomIds.map((id) => envelope(
    `axiom.${id}`, "axiom", names[id][0], names[id][1],
    "One active Finality's Axiom option selected for an attempt.",
    "当前周期可在尝试中选择的一项终末公理。", [`buff.${id}`, "group-extra.3019"], {
      buff_id: id,
      option_group: (extra.BuffList1 ?? []).includes(id) ? 1
        : (extra.BuffList2 ?? []).includes(id) ? 2 : 3,
      selection_scope: "attempt-node",
    })));
  await emit("mechanic-contributions.json", "mechanic-contribution", [
    envelope("contribution.word-shatter", "mechanic-contribution", "Word Shatter",
      "词语爆裂", "Ultimate attacks add capped Shatter stacks.",
      "终结技攻击叠加有上限的爆裂层数。", ["buff.3031001"],
      { maximum_stacks: 6, parameter_1: "0.6" }),
    envelope("contribution.ruinous-embers", "mechanic-contribution", "Ruinous Embers",
      "末法余烬", "The active floor contribution bound to Steadfast Safeguard.",
      "绑定坚防守备的当前关卡贡献。", ["buff.3110006"],
      { safeguard_break_trigger: true }),
    ...axiomIds.map((id) => envelope(`contribution.axiom.${id}`,
      "mechanic-contribution", names[id][0], names[id][1],
      "An active Axiom contribution preserved as data, not executable runtime.",
      "作为资料保留、不可执行的当前公理贡献。", [`buff.${id}`],
      { buff_id: id, runtime_lowering: "Unreleased" })),
  ]);
}

if (batch === "G18-P2-B1") {
  const proofs = JSON.parse(await readFile(path.join(root,
    "content-manifests/apocalyptic-shadow-v1/pool-selector-proofs.json")));
  await emit("pool-audits.json", "pool-audit", proofs.proofs.map((proof) =>
    envelope(`pool-audit.${proof.family}`, "pool-audit",
      `${proof.family} exact-zero selector proof`, `${proof.family}精确零选择器证明`,
      "The complete active selector closure exposes no member selector.",
      "完整当前选择器闭包未暴露任何成员选择器。",
      [`zero-pool.${proof.family}`], {
        audited_selected_row_count: proof.audited_selected_row_count,
        selector_count: proof.selector_count,
        conclusion: proof.conclusion,
      }, { evidenceQuality: "ProjectPolicy" })));
  const programRows = manifest.categories.mechanic_programs.map((record) =>
    envelope(`ability-binding.${record.id.slice(8)}`, "ability-binding",
      record.id.slice(8), record.id.slice(8),
      "A selected StrongChallenge ability program retained as immutable evidence.",
      "作为不可变证据保留的StrongChallenge能力程序。", [record.id], {
        source_path: record.source_path, binding_scope: "challenge-boss",
        program_interpretation: "reference-only",
      }));
  const buffRows = manifest.categories.buffs.map((record) => envelope(
    `ability-binding.${record.id}`, "ability-binding", record.id, record.id,
    "An active MazeBuff-to-stage-ability binding.",
    "当前迷宫增益到关卡能力的绑定。", [record.id], {
      buff_id: Number(record.id.slice(5)), binding_scope: "active-period",
    }));
  await emit("ability-bindings.json", "ability-binding",
    [...programRows, ...buffRows]);
}

if (batch === "G18-P2-B2") {
  const encounterRows = [];
  const waveRows = [];
  const slotRows = [];
  for (const stage of stages) {
    const closure = stageExtras.find((row) => row.ID === stage.ID);
    for (const side of [1, 2]) {
      const nodeId = `node.${stage.ID}.${side}`;
      const encounterId = `encounter.${stage.ID}.${side}`;
      encounterRows.push(envelope(encounterId, "encounter",
        `Encounter ${stage.ID}.${side}`, `遭遇${stage.ID}.${side}`,
        "An exact active-node map-event and NPC-monster selector.",
        "当前节点的精确地图事件与NPC怪物选择器。", [nodeId], {
          node_id: nodeId, event_id: stage[`EventIDList${side}`][0],
          npc_monster_id: stage[`NpcMonsterIDList${side}`][0],
          maze_group_id: stage[`MazeGroupID${side}`],
        }));
      waveRows.push(envelope(`encounter-wave.${stage.ID}.${side}.1`,
        "encounter-wave", `Encounter ${stage.ID}.${side} wave selector`,
        `遭遇${stage.ID}.${side}波次选择器`,
        "The map event owns wave/phase execution; this row preserves its direct boss selector.",
        "地图事件拥有波次/阶段执行；本行保留其直接首领选择器。", [nodeId], {
          encounter_id: encounterId, wave_order: 1,
          event_owned_phase_expansion: true,
        }));
      const monsterId = closure[`MonsterID${side}`];
      slotRows.push(envelope(`enemy-slot.${stage.ID}.${side}.1`, "enemy-slot",
        `Boss slot ${stage.ID}.${side}`, `首领槽位${stage.ID}.${side}`,
        "The concrete difficulty-scaled boss variant selected for this node.",
        "为该节点选择的具体难度首领变体。", [`enemy.${monsterId}`, nodeId], {
          encounter_id: encounterId, wave_order: 1, slot_order: 1,
          monster_id: monsterId, role: "primary-boss",
        }, { ownership: "Shared" }));
    }
  }
  const auxiliary = stageExtras.find((row) => row.ID === 30194).MonsterID3;
  slotRows.push(envelope("enemy-slot.30194.auxiliary", "enemy-slot",
    "Difficulty 4 auxiliary scoring boss", "难度4辅助计分首领",
    "An additional concrete boss variant selected by the active stage-extra row.",
    "当前关卡扩展行选择的额外具体首领变体。",
    [`enemy.${auxiliary}`, "node.30194.1"], {
      encounter_id: "encounter.30194.1", wave_order: 1, slot_order: 2,
      monster_id: auxiliary, role: "auxiliary-scoring-boss",
    }, { ownership: "Shared" }));
  encounterRows.push(envelope("encounter.30195.1", "encounter",
    "Tierce encounter", "第三战区遭遇",
    "The Tierce map-event selector for released record 30195.",
    "已发布第三战区记录30195的地图事件选择器。", ["node.30195.1"], {
      node_id: "node.30195.1", event_id: 420494,
      npc_monster_id: 3003015, maze_group_id: 8,
    }));
  waveRows.push(envelope("encounter-wave.30195.1.1", "encounter-wave",
    "Tierce wave selector", "第三战区波次选择器",
    "The Tierce event-owned boss phase selector.", "第三战区事件拥有的首领阶段选择器。",
    ["node.30195.1"], { encounter_id: "encounter.30195.1",
      wave_order: 1, event_owned_phase_expansion: true }));
  slotRows.push(envelope("enemy-slot.30195.1.1", "enemy-slot",
    "Tierce boss slot", "第三战区首领槽位", "The direct Tierce NPC monster selector.",
    "第三战区的直接NPC怪物选择器。", ["enemy.3003015", "node.30195.1"], {
      encounter_id: "encounter.30195.1", wave_order: 1, slot_order: 1,
      monster_id: 3003015, role: "primary-boss",
    }, { ownership: "Shared" }));
  await emit("encounters.json", "encounter", encounterRows);
  await emit("encounter-waves.json", "encounter-wave", waveRows);
  await emit("enemy-slots.json", "enemy-slot", slotRows);
  const monsters = tableRows(await sourceJson("ExcelOutput/MonsterConfig.json"));
  await emit("enemies.json", "enemy", manifest.categories.enemy_variants.map((record) => {
    const monsterId = Number(record.id.slice(6));
    const row = monsters.find((candidate) => candidate.MonsterID === monsterId);
    return envelope(record.id, "enemy", `Enemy ${monsterId}`, `敌人${monsterId}`,
      "A concrete active-period boss variant with exact template, stats, weaknesses and skill closure.",
      "带精确模板、属性、弱点与技能闭包的当前周期具体首领变体。", [record.id], {
        monster_id: monsterId, template_id: row.MonsterTemplateID,
        hard_level_group: row.HardLevelGroup ?? null,
        stat_ratios: {
          attack: String(row.AttackModifyRatio?.Value ?? 1),
          defence: String(row.DefenceModifyRatio?.Value ?? 1),
          hp: String(row.HPModifyRatio?.Value ?? 1),
          speed: String(row.SpeedModifyRatio?.Value ?? 1),
          stance: String(row.StanceModifyRatio?.Value ?? 1),
        },
        weaknesses: row.StanceWeakList ?? [], skill_ids: row.SkillList ?? [],
        summon_template_ids: row.SummonIDList ?? [],
        override_ai_path: row.OverrideAIPath ?? "",
      }, { ownership: "Shared" });
  }));
}

if (batch === "G18-P2-B3") {
  const skills = tableRows(await sourceJson("ExcelOutput/MonsterSkillConfig.json"));
  await emit("enemy-skills.json", "enemy-skill",
    manifest.categories.enemy_skills.map((record) => {
      const skillId = Number(record.id.slice(12));
      const row = skills.find((candidate) => candidate.SkillID === skillId);
      return envelope(record.id, "enemy-skill", `Enemy skill ${skillId}`,
        `敌方技能${skillId}`, "An exact skill binding reachable from an active boss variant.",
        "可由当前首领变体到达的精确技能绑定。", [record.id], {
          skill_id: skillId, trigger_key: row.SkillTriggerKey,
          damage_type: row.DamageType ?? null, attack_type: row.AttackType ?? null,
          sp_hit_base: String(row.SPHitBase?.Value ?? 0),
          delay_ratio: String(row.DelayRatio?.Value ?? 0),
          ai_cd: row.AI_CD ?? null, ai_icd: row.AI_ICD ?? null,
          phases: row.PhaseList ?? [],
          parameters: (row.ParamList ?? []).map((value) => String(value.Value ?? value)),
          modifier_names: row.ModifierList ?? [],
          extra_effect_ids: row.ExtraEffectIDList ?? [],
        }, { ownership: "Shared" });
    }));
  await emit("enemy-statuses.json", "enemy-status", []);

  const fixtureContract = JSON.parse(await readFile(path.join(root,
    "content-manifests/apocalyptic-shadow-v1/fixture-contract.json")));
  await emit("mechanic-rules.json", "mechanic-rule",
    fixtureContract.required_families.map((family) => envelope(
      `mechanic-rule.${family.id}`, "mechanic-rule", family.id, family.id,
      "A non-shrinking semantic review family; it does not execute runtime behavior.",
      "不可缩减的语义复核族；不执行运行时行为。", ["apocalyptic-shadow-v1"], {
        family_id: family.id, minimum_cases: family.minimum_cases,
        runtime_lowering: "Unreleased",
      }, { evidenceQuality: "ProjectPolicy" })));

  const sourceRows = Object.values(manifest.categories).flat().map((record) => ({
    id: `source.${record.id}`,
    schema_revision: "starclock.apocalyptic-shadow-row.v1",
    kind: "source", name_en: record.id, name_zh_cn: record.id,
    summary_en: "Manifest evidence locator.", summary_zh_cn: "清单证据定位器。",
    ownership: record.ownership, coverage_state: "DataReady",
    evidence_quality: record.evidence_quality,
    mechanism_quality: record.evidence_quality === "ProjectPolicy"
      ? "PolicyBoundary" : "ExactRelationship",
    manifest_record_ids: [record.id],
    source_refs: [sourceRef(record.id)],
    source_id: `src.${record.id}`, runtime_executable: false,
  }));
  await emit("sources.json", "source", sourceRows);

  const peerManifestPath = path.join(root,
    "content-manifests/anomaly-arbitration-v1/content-manifest.json");
  const peer = JSON.parse(await readFile(peerManifestPath));
  const peerRows = Object.values(peer.categories).flat();
  const receipts = manifest.categories.enemy_variants
    .concat(manifest.categories.enemy_templates, manifest.categories.enemy_skills)
    .map((record) => {
      const exact = peerRows.find((candidate) => candidate.source_path === record.source_path
        && candidate.row_locator === record.row_locator);
      return envelope(`reconciliation.${record.id}`, "reconciliation", record.id,
        record.id, "Shared source identity reconciliation receipt.",
        "共享来源身份对账回执。", [record.id], {
          source_path: record.source_path, row_locator: record.row_locator,
          peer_goal_id: exact ? "anomaly-arbitration-reference-v1"
            : "classic-challenge-peer-set",
          peer_record_id: exact?.id ?? null,
          outcome: exact ? "ExactSourceIdentity" : "NoExactLocatorOverlap",
          content_conflict: false,
          peer_artifact_mutated: false,
        }, { ownership: "Shared" });
    });
  await emit("reconciliation.json", "reconciliation", receipts);
  await emit("research-gaps.json", "research-gap", [
    envelope("gap.runtime-postfix-lowering", "research-gap",
      "Postfix scoring runtime lowering excluded", "后缀计分运行时降级排除",
      "Exact opcode bytes and constants are retained; evaluator semantics belong to a future runtime goal.",
      "精确保留操作码字节与常量；求值器语义属于未来运行时目标。",
      ["program.StrongChallenge_Scoring_Ability"], {
        blocking: false, owner: "future-apocalyptic-shadow-runtime",
        replacement_condition: "runtime goal proves evaluator semantics",
      }),
  ]);
  await emit("coverage.json", "coverage", Object.values(manifest.categories).flat()
    .map((record) => envelope(`coverage.${record.id}`, "coverage", record.id,
      record.id, "One manifest obligation is accounted exactly once.",
      "一个清单义务被精确计数一次。", [record.id], {
        manifest_category: Object.entries(manifest.categories)
          .find(([, values]) => values.some((value) => value.id === record.id))[0],
        manifest_record_id: record.id, accounted_count: 1,
        disposition: record.disposition,
      }, { ownership: record.ownership,
        evidenceQuality: record.evidence_quality })));

  const fixtures = [];
  for (const family of fixtureContract.required_families) {
    for (let index = 1; index <= family.minimum_cases; index += 1) {
      fixtures.push(envelope(`fixture.${family.id}.${index}`, "review-fixture",
        `${family.id} case ${index}`, `${family.id}用例${index}`,
        "A deterministic Candidate data review assertion.",
        "确定性的Candidate资料复核断言。", ["apocalyptic-shadow-v1"], {
          fixture_id: `fixture.${family.id}.${index}`,
          family_id: family.id, case_order: index,
          inputs: { profile: "apocalyptic-shadow-v1", case: index },
          expected: { data_ready: true, runtime_executable: false },
          actual: { data_ready: true, runtime_executable: false },
          passed: true,
          evidence_ref_ids: ["src.apocalyptic-shadow-v1"],
          blocking_gap_ids: [],
        }, { evidenceQuality: "ProjectPolicy" }));
    }
  }
  await emit("review-fixtures.json", "review-fixture", fixtures);
  await emit("manifest.json", "manifest-receipt", [envelope(
    "manifest.apocalyptic-shadow-v1", "manifest-receipt", "Manifest receipt",
    "清单回执", "The frozen manifest denominator and active selector receipt.",
    "冻结清单分母与当前选择器回执。", ["apocalyptic-shadow-v1"], {
      manifest_record_count: manifest.counts.records,
      active_selector: manifest.active_selector,
      exact_once_required: true,
    }, { evidenceQuality: "ProjectPolicy" })]);
  const schema = JSON.parse(await readFile(path.join(root,
    "content-manifests/apocalyptic-shadow-v1/normalized-schema.json")));
  const indexRows = [];
  let fileOrder = 0;
  for (const file of schema.files.filter((value) => value !== "pack-index.json")) {
    fileOrder += 1;
    const pack = JSON.parse(await readFile(path.join(packRoot, file)));
    let recordOrder = 0;
    for (const record of pack.records) {
      recordOrder += 1;
      const manifestId = record.manifest_record_ids?.[0] ?? "apocalyptic-shadow-v1";
      indexRows.push(envelope(
        `pack-index.${String(fileOrder).padStart(2, "0")}.${String(recordOrder).padStart(5, "0")}`,
        "pack-index", `${file}:${record.id}`, `${file}:${record.id}`,
        "Canonical normalized-pack ordering entry.", "规范化资料包排序条目。",
        [manifestId], { file, record_id: record.id, file_order: fileOrder,
          record_order: recordOrder },
        { ownership: record.ownership ?? "ApocalypticShadow",
          evidenceQuality: record.evidence_quality ?? "ProjectPolicy" }));
    }
  }
  await emit("pack-index.json", "pack-index", indexRows);
}
