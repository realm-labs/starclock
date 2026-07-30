#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const outputRoot = path.join(root, "content-reference/anomaly-arbitration-v1");
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const revision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function sourceRef(category, id, note, mechanism = "ExactRelationship") {
  const record = manifest.categories[category].records.find(
    (candidate) => candidate.id === id,
  );
  return {
    source_id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
    mechanism_quality: mechanism,
    note,
  };
}

function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  ownership,
  manifestIds,
  sources,
  tags,
  fields,
  mechanismQuality = "ExactRelationship",
}) {
  return {
    id,
    schema_revision: "starclock.anomaly-arbitration-row.v1",
    kind,
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    ownership,
    coverage_state: "DataReady",
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...manifestIds].sort(),
    source_refs: sources,
    tags: [...tags].sort(),
    ...fields,
    runtime_executable: false,
  };
}

const traits = [
  [3033023, "Taunting", "挑衅", ["6"],
    "ChallengePeakBattle_BaseAbility_0008", ["stage.knight-1"],
    "After six allied attacks, an enemy acts immediately; its tally clears at its turn start.",
    "敌方受到我方六次攻击后立即行动，计数在其回合开始时清空。"],
  [3033038, "Depowered", "失能", ["0.5", "0.5", "2"],
    "ChallengePeakBattle_BaseAbility_0013", ["stage.knight-2"],
    "On entry, allies lose 50% Energy and have 50% lower off-turn Energy regeneration for two turns.",
    "入战时我方失去50%能量，回合外能量恢复效率降低50%，持续两回合。"],
  [3033051, "Equilibrium", "均衡", ["1", "0.2", "1", "0.15"],
    "ChallengePeakBattle_BaseAbility_0016", ["stage.king-normal"],
    "At battle start, the fastest ally loses 20% SPD and the slowest gains 15% SPD.",
    "战斗开始时，最快的一名我方角色速度降低20%，最慢的一名提高15%。"],
  [3033052, "Equilibrium+", "均衡+", ["2", "0.2", "1", "0.15"],
    "ChallengePeakBattle_EnhancedAbility_0016", ["stage.king-plight"],
    "At battle start, the two fastest allies lose 20% SPD and the slowest gains 15% SPD.",
    "战斗开始时，最快的两名我方角色速度降低20%，最慢的一名提高15%。"],
  [3033058, "Hemovore", "血嗜", ["500"],
    "ChallengePeakBattle_BaseAbility_0018", ["stage.knight-3"],
    "At each ally turn start, that ally loses 500 HP; the loss can be fatal.",
    "我方角色回合开始时损失500点生命值，且该损失可以致命。"],
  [3033063, "Flow Break", "破势", ["0.02", "0.04", "10", "5"],
    "ChallengePeakBattle_BaseAbility_0019", ["stage.knight-2"],
    "Allied hits stack 2% damage reduction and 4% CRIT-DMG reduction on enemies up to ten; Follow-Up or Aha endpoints remove five.",
    "我方命中使敌方叠加2%减伤与4%暴伤减免，最多十层；追加攻击或阿哈时刻结束移除五层。"],
  [3033069, "Enrage", "激怒", ["0.3", "4"],
    "ChallengePeakBattle_BaseAbility_0020", ["stage.king-normal"],
    "Each allied Ultimate gives all enemies one 30% SPD stack, up to four, cleared at enemy turn start.",
    "我方每次施放终结技使敌方全体叠加一层30%速度提高，最多四层，并在敌方回合开始时清空。"],
  [3033070, "Enrage+", "激怒+", ["0.5", "4"],
    "ChallengePeakBattle_EnhancedAbility_0020", ["stage.king-plight"],
    "Each allied Ultimate gives all enemies one 50% SPD stack, up to four, cleared at enemy turn start.",
    "我方每次施放终结技使敌方全体叠加一层50%速度提高，最多四层，并在敌方回合开始时清空。"],
];
const traitRows = traits.map(
  ([numericId, en, zh, parameters, binding, stages, summaryEn, summaryZh]) =>
    envelope({
      id: `trait.${numericId}`,
      kind: "EnemyTrait",
      nameEn: en,
      nameZh: zh,
      summaryEn,
      summaryZh,
      ownership: "Shared",
      manifestIds: [`stage_traits:trait:${numericId}`],
      sources: [sourceRef(
        "stage_traits",
        `trait:${numericId}`,
        "Active normal or Plight TagList selects this exact MazeBuff row.",
      )],
      tags: ["enemy-trait", stages[0].replace("stage.", "")],
      fields: {
        source_numeric_id: numericId,
        source_parameters: parameters,
        in_battle_binding_type: "StageAbilityBeforeCharacterBorn",
        in_battle_binding_key: binding,
        stage_ids: stages,
        binding_program_state: "ResolvedInExtractedAbilityList",
      },
    }),
);

const quadrantOptions = JSON.parse(await readFile(path.join(
  outputRoot,
  "quadrant-options.json",
), "utf8")).records;
const bindingSpecs = [
  ...traits.map(
    ([id, en, zh, parameters, binding, stages]) => ({
      id: `trait.${id}`,
      numericId: id,
      nameEn: `${en} binding`,
      nameZh: `${zh}绑定`,
      ownership: "Shared",
      category: "stage_traits",
      manifestId: `trait:${id}`,
      stages,
      parameters,
      binding,
      state: "ResolvedInExtractedAbilityList",
      role: "EnemyTrait",
    }),
  ),
  ...quadrantOptions.map((option) => ({
    id: option.id,
    numericId: option.source_numeric_id,
    nameEn: `${option.name_en} binding`,
    nameZh: `${option.name_zh_cn}绑定`,
    ownership: "Shared",
    category: "quadrant_options",
    manifestId: `quadrant:${option.source_numeric_id}`,
    stages: option.stage_scope.map((scope) =>
      scope === "KingNormal" ? "stage.king-normal" : "stage.king-plight"),
    parameters: option.source_parameters,
    binding: option.in_battle_binding_key,
    state: option.binding_program_state,
    role: "QuadrantOption",
  })),
];
const bindingRows = bindingSpecs.flatMap((spec) =>
  spec.stages.map((stageId, stageIndex) => envelope({
    id: `maze-binding.${spec.numericId}.${stageId.replaceAll(".", "-")}`,
    kind: "MazeBuffBinding",
    nameEn: spec.nameEn,
    nameZh: spec.nameZh,
    summaryEn:
      `${spec.role} ${spec.numericId} installs ${spec.binding} before characters are created in ${stageId}.`,
    summaryZh:
      `${spec.role} ${spec.numericId} 在 ${stageId} 角色创建前安装 ${spec.binding}。`,
    ownership: spec.ownership,
    manifestIds: [`${spec.category}:${spec.manifestId}`],
    sources: [sourceRef(
      spec.category,
      spec.manifestId,
      `Exact MazeBuff binding for ${stageId}.`,
      spec.state === "ResolvedInExtractedAbilityList"
        ? "ExactRelationship"
        : "PolicyBoundary",
    )],
    tags: ["binding", "maze-buff", spec.role.toLowerCase()],
    mechanismQuality: spec.state === "ResolvedInExtractedAbilityList"
      ? "ExactRelationship"
      : "PolicyBoundary",
    fields: {
      stage_id: stageId,
      binding_order: (spec.role === "EnemyTrait" ? 10 : 20) + stageIndex,
      source_numeric_id: spec.numericId,
      source_role: spec.role,
      source_parameters: spec.parameters,
      in_battle_binding_type: "StageAbilityBeforeCharacterBorn",
      in_battle_binding_key: spec.binding,
      binding_program_state: spec.state,
    },
  })),
);
bindingRows.sort((left, right) =>
  left.stage_id.localeCompare(right.stage_id)
    || left.binding_order - right.binding_order
    || left.id.localeCompare(right.id));

const directContributions = [
  ...bindingSpecs.map((spec, index) => envelope({
    id: `mechanic-contribution.${spec.role.toLowerCase()}.${spec.numericId}`,
    kind: "MechanicContribution",
    nameEn: `${spec.nameEn} contribution`,
    nameZh: `${spec.nameZh}贡献`,
    summaryEn:
      `${spec.role} ${spec.numericId} contributes its canonical parameters through ${spec.binding}.`,
    summaryZh:
      `${spec.role} ${spec.numericId} 通过 ${spec.binding} 贡献其规范参数。`,
    ownership: spec.ownership,
    manifestIds: [`${spec.category}:${spec.manifestId}`],
    sources: [sourceRef(
      spec.category,
      spec.manifestId,
      "Explicit active MazeBuff contribution.",
      spec.state === "ResolvedInExtractedAbilityList"
        ? "ExactRelationship"
        : "PolicyBoundary",
    )],
    tags: ["contribution", "maze-buff", spec.role.toLowerCase()],
    mechanismQuality: spec.state === "ResolvedInExtractedAbilityList"
      ? "ExactRelationship"
      : "PolicyBoundary",
    fields: {
      scope: spec.role === "EnemyTrait" ? "StageBattle" : "KingAttempt",
      install_order: 10 + index,
      stage_ids: spec.stages,
      source_numeric_id: spec.numericId,
      source_parameters: spec.parameters,
      program_name: spec.binding,
      program_state: spec.state,
      contribution_start: "BeforeCharacterBorn",
      contribution_end: "BattleTerminal",
    },
  })),
  ...manifest.categories.battle_events.records.map((record, index) =>
    envelope({
      id: `mechanic-contribution.${record.id}`,
      kind: "MechanicContribution",
      nameEn: `${record.id} contribution`,
      nameZh: `${record.id}贡献`,
      summaryEn:
        `${record.id} contributes the selected stage countdown and wave-support event actor.`,
      summaryZh:
        `${record.id} 贡献关卡所选倒计时与波次支援事件参与者。`,
      ownership: record.ownership,
      manifestIds: [`battle_events:${record.id}`],
      sources: [sourceRef(
        "battle_events",
        record.id,
        "Active StageConfig battle-event contribution.",
      )],
      tags: ["battle-event", "contribution"],
      fields: {
        scope: "StageBattle",
        install_order: 40 + index,
        source_numeric_id: Number(record.id.split(":")[1]),
        program_names: [
          "BattleEventAbility_SummonMonsterInfinite",
          "BattleEventAbility_ChallengePeakBattle_CountDown",
        ],
        contribution_start: "BattleEventCreated",
        contribution_end: "BattleTerminal",
      },
    })),
];

function programScope(sourcePath) {
  if (sourcePath.includes("ConfigAI/")) return "EnemyAI";
  if (sourcePath.includes("ConfigAbility/")) return "AbilityProgram";
  if (sourcePath.includes("ConfigCharacter/")) return "CharacterProgram";
  if (sourcePath.includes("Stage")) return "StageProgram";
  return "EncounterProgram";
}
const programRows = manifest.categories.config_programs.records.map(
  (record, index) => envelope({
    id: `mechanic-contribution.config.${String(index + 1).padStart(3, "0")}`,
    kind: "MechanicContribution",
    nameEn: `Reachable configuration program ${index + 1}`,
    nameZh: `可达配置程序${index + 1}`,
    summaryEn:
      `The active encounter closure reaches ${record.source_path} through its recorded stable reference.`,
    summaryZh:
      `当期遭遇闭包通过已记录稳定引用到达 ${record.source_path}。`,
    ownership: record.ownership,
    manifestIds: [`config_programs:${record.id}`],
    sources: [sourceRef(
      "config_programs",
      record.id,
      record.selector,
      "ExactRelationship",
    )],
    tags: ["config-program", "contribution", programScope(record.source_path)],
    mechanismQuality: "ExactRelationship",
    fields: {
      scope: programScope(record.source_path),
      install_order: 100 + index,
      source_path: record.source_path,
      source_locator: record.row_locator,
      reachability: record.reachability,
      selector: record.selector,
      program_body_imported: false,
      runtime_executable: false,
    },
  }),
);
const contributionRows = [...directContributions, ...programRows].sort(
  (left, right) =>
    left.scope.localeCompare(right.scope)
      || left.install_order - right.install_order
      || left.id.localeCompare(right.id),
);

function file(name, kind, records) {
  return {
    schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
    goal_id: "anomaly-arbitration-reference-v1",
    profile: "anomaly-arbitration-v1",
    file: name,
    record_kind: kind,
    records,
  };
}
const outputs = {
  "traits.json": file("traits.json", "EnemyTrait", traitRows),
  "maze-buff-bindings.json": file(
    "maze-buff-bindings.json",
    "MazeBuffBinding",
    bindingRows,
  ),
  "mechanic-contributions.json": file(
    "mechanic-contributions.json",
    "MechanicContribution",
    contributionRows,
  ),
};
await mkdir(outputRoot, { recursive: true });
for (const [name, document] of Object.entries(outputs)) {
  const bytes = `${JSON.stringify(document, null, 2)}\n`;
  const target = path.join(outputRoot, name);
  if (check) {
    const existing = await readFile(target, "utf8").catch(() => "");
    if (existing !== bytes) throw new Error(`${name} generation drift`);
  } else {
    await writeFile(target, bytes);
  }
}
console.log(
  `Anomaly Arbitration mechanics generated: ${traitRows.length} traits, `
    + `${bindingRows.length} bindings, `
    + `${contributionRows.length} contributions.`,
);
