#!/usr/bin/env node

import { createHash } from "node:crypto";
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
const officialUrl = "https://www.hoyolab.com/article/41091494";

function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function manifestId(category, id) {
  if (!manifest.categories[category]?.records.some(
    (record) => record.id === id,
  )) throw new Error(`missing manifest record: ${category}/${id}`);
  return `${category}:${id}`;
}

function structuredRef(category, id, note, mechanism = "ExactRelationship") {
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
    evidence_quality: "ExactStructured",
    mechanism_quality: mechanism,
    note,
  };
}

function textRef(locale, hash, value) {
  const sourcePath = locale === "zh_cn"
    ? "TextMap/TextMapCHS.json"
    : "TextMap/TextMapEN.json";
  return {
    source_id: `turnbasedgamedata:${sourcePath}:Hash=${hash}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: revision,
    game_version: "4.4",
    path_or_page: sourcePath,
    locator: `Hash=${hash}`,
    sha256: digest({ hash, value }),
    evidence_quality: "ExactStructured",
    mechanism_quality: "IdentityCrossCheck",
    note: `Exact ${locale} released target text.`,
  };
}

function officialRef(id, locator, fact) {
  return {
    source_id: `official:hoyolab:anomaly-arbitration:${id}`,
    repository_or_url: officialUrl,
    revision_or_access_date: "accessed 2026-07-29",
    game_version: "4.4",
    path_or_page: officialUrl,
    locator,
    sha256: digest(fact),
    evidence_quality: "ExactPublicText",
    mechanism_quality: "ExactRelationship",
    note: fact,
  };
}

function policyRef(id, note) {
  return {
    source_id: `goal13:objective-policy:${id}`,
    repository_or_url: "starclock",
    revision_or_access_date: "G13-P1-B6",
    game_version: "4.4",
    path_or_page: "docs/goals/13-anomaly-arbitration-reference-data.md",
    locator: "Phase 1 G13-P1-B6",
    sha256: digest({ id, note }),
    evidence_quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
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
  evidenceQuality,
  mechanismQuality,
  manifestIds,
  sources,
  tags,
  fields,
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
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [...manifestIds].sort(),
    source_refs: sources,
    tags: [...tags].sort(),
    ...fields,
    runtime_executable: false,
  };
}

const targetSpecs = [
  [3000, "No downed characters", "没有角色无法战斗",
    "2308537234688475763", "BattleTarget_DeathCount", "LessEqual", 0],
  [3001, "Victory within 4 cycles", "不超过4轮战斗胜利",
    "12967210961986691944", "BattleTarget_TurnLimit_PeakBattle_1",
    "LessEqual", 4],
  [3002, "Victory within 2 cycles", "不超过2轮战斗胜利",
    "6566380338945159645", "BattleTarget_TurnLimit_PeakBattle_2",
    "LessEqual", 2],
  [3003, "Victory within 6 cycles", "不超过6轮战斗胜利",
    "13550969154333397837", "BattleTarget_TurnLimit_PeakBattle_3",
    "LessEqual", 6],
  [3004, "Victory within 4 cycles", "不超过4轮战斗胜利",
    "14482168213018670964", "BattleTarget_TurnLimit_PeakBattle_4",
    "LessEqual", 4],
  [3005, "Victory within 2 cycles", "不超过2轮战斗胜利",
    "2827694259035707914", "BattleTarget_TurnLimit_PeakBattle_5",
    "LessEqual", 2],
  [3007, "Plight victory within 2 cycles", "困厄王棋不超过2轮战斗胜利",
    "4060858115664570856", "BattleTarget_TurnLimit_PeakBattle_7",
    "LessEqual", 2],
];
const cycleTextEn =
  "Achieve victory in no more than <color=#f29e38ff><unbreak>#1[i]</unbreak></color> Cycles";
const cycleTextZh =
  "不超过<color=#f29e38ff><unbreak>#1[i]</unbreak></color>轮战斗胜利";
const targetRows = targetSpecs.map(
  ([numericId, en, zh, hash, ability, compare, parameter]) => envelope({
    id: `battle-target.${numericId}`,
    kind: "BattleTarget",
    nameEn: en,
    nameZh: zh,
    summaryEn: numericId === 3000
      ? "The target succeeds when no party member is downed at victory."
      : `The target succeeds when the stage is won within ${parameter} cycles.`,
    summaryZh: numericId === 3000
      ? "战斗胜利时没有我方角色倒下即可满足该目标。"
      : `在${parameter}轮内取得关卡胜利即可满足该目标。`,
    ownership: "Shared",
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    manifestIds: [manifestId("battle_targets", `target:${numericId}`)],
    sources: [
      structuredRef(
        "battle_targets",
        `target:${numericId}`,
        "Active stage definition explicitly selects this target.",
      ),
      textRef(
        "en",
        hash,
        numericId === 3000 ? "Have no downed characters" : cycleTextEn,
      ),
      textRef(
        "zh_cn",
        hash,
        numericId === 3000 ? "没有角色无法战斗" : cycleTextZh,
      ),
    ],
    tags: ["battle-target", numericId === 3000 ? "survival" : "cycle"],
    fields: {
      source_numeric_id: numericId,
      source_ability_name: ability,
      comparison: compare,
      comparison_parameter: parameter,
      evaluation_boundary: "SuccessfulStageTerminal",
      satisfied_value: 1,
      unsatisfied_value: 0,
    },
  }),
);

const stageTargetSets = {
  "stage.knight-1": ["battle-target.3001", "battle-target.3002",
    "battle-target.3000"],
  "stage.knight-2": ["battle-target.3001", "battle-target.3002",
    "battle-target.3000"],
  "stage.knight-3": ["battle-target.3001", "battle-target.3002",
    "battle-target.3000"],
  "stage.king-normal": ["battle-target.3003", "battle-target.3004",
    "battle-target.3005"],
  "stage.king-plight": ["battle-target.3007"],
};
const starFact = officialRef(
  "independent-stage-stars",
  "Knight Stage and Cycles",
  "Each Knight stage calculates its stars independently at completion; stars from multiple clears of the same stage are not combined.",
);
const bestFact = officialRef(
  "simultaneous-best",
  "Knight Stage",
  "Best Battle Records use the highest total star rating active simultaneously across all three Knight stages, and current resets do not affect them.",
);
const recordsFact = officialRef(
  "retained-battle-records",
  "Battle Records",
  "Battle Records display challenge records from the last three Anomaly Arbitration phases.",
);
const kingFact = officialRef(
  "king-rating",
  "King in Check Stage and Battle Records",
  "The King stage has its own star rating, which is retained in Battle Records.",
);

const objectiveRows = [
  envelope({
    id: "objective.evaluate-cycle-targets",
    kind: "ObjectiveRule",
    nameEn: "Cycle target evaluation",
    nameZh: "轮次目标评估",
    summaryEn:
      "Cycle targets compare the consumed stage-local cycle count at successful terminal.",
    summaryZh: "轮次目标在成功终局时比较关卡局部已消耗轮次。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    manifestIds: targetSpecs.filter(([id]) => id !== 3000).map(
      ([id]) => manifestId("battle_targets", `target:${id}`),
    ),
    sources: [
      ...targetSpecs.filter(([id]) => id !== 3000).map(
        ([id]) => structuredRef(
          "battle_targets",
          `target:${id}`,
          "Exact LessEqual cycle target.",
        ),
      ),
      starFact,
    ],
    tags: ["cycle", "evaluation", "objective"],
    fields: {
      evaluation_order: 10,
      boundary: "SuccessfulStageTerminal",
      value_source: "ConsumedStageLocalCycles",
      comparison: "LessEqual",
      failed_attempt_satisfies: false,
    },
  }),
  envelope({
    id: "objective.evaluate-no-downed",
    kind: "ObjectiveRule",
    nameEn: "No-downed target evaluation",
    nameZh: "无人倒下目标评估",
    summaryEn:
      "The survival target is satisfied only when victory commits with zero downed party members.",
    summaryZh: "仅当胜利提交时我方倒下人数为零，生存目标才满足。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    manifestIds: [manifestId("battle_targets", "target:3000")],
    sources: [
      structuredRef(
        "battle_targets",
        "target:3000",
        "Exact no-downed-character target.",
      ),
      starFact,
    ],
    tags: ["evaluation", "objective", "survival"],
    fields: {
      evaluation_order: 20,
      boundary: "SuccessfulStageTerminal",
      value_source: "DownedPartyMemberCount",
      comparison: "LessEqual",
      comparison_parameter: 0,
      failed_attempt_satisfies: false,
    },
  }),
  envelope({
    id: "objective.stage-star-evaluation",
    kind: "ObjectiveRule",
    nameEn: "Per-stage star evaluation",
    nameZh: "逐关星级评估",
    summaryEn:
      "A successful stage result counts its satisfied active targets once and never combines targets from separate attempts.",
    summaryZh:
      "成功关卡结果仅计算本次满足的当期目标，不合并不同尝试的目标。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "ExactRelationship",
    manifestIds: [
      manifestId("objective_aggregations", "per-stage-star-evaluation"),
    ],
    sources: [starFact],
    tags: ["evaluation", "stage", "star"],
    fields: {
      evaluation_order: 30,
      boundary: "SuccessfulStageTerminal",
      combination_scope: "OneCompletedAttempt",
      combine_across_attempts: false,
      star_value_rule: "CountSatisfiedActiveTargets",
    },
  }),
];

const stageSpecs = [
  ["stage-result.knight-1", "stage.knight-1", 1, "Knight"],
  ["stage-result.knight-2", "stage.knight-2", 2, "Knight"],
  ["stage-result.knight-3", "stage.knight-3", 3, "Knight"],
  ["stage-result.king-normal", "stage.king-normal", 4, "KingNormal"],
  ["stage-result.king-plight", "stage.king-plight", 5, "KingPlight"],
];
const stageRows = stageSpecs.map(([id, stageId, order, kind]) => envelope({
  id,
  kind: "StageResultPolicy",
  nameEn: `${kind} stage result`,
  nameZh: `${kind === "Knight" ? "骑士" : kind === "KingNormal"
    ? "常规王棋" : "困厄王棋"}关卡结果`,
  summaryEn:
    `The ${kind} terminal result records only the targets evaluated for ${stageId}.`,
  summaryZh: `该终局结果仅记录 ${stageId} 对应目标的评估值。`,
  ownership: "AnomalyArbitration",
  evidenceQuality: "ExactStructured",
  mechanismQuality: "ExactRelationship",
  manifestIds: [
    manifestId("objective_aggregations", "current-stage-progress"),
    manifestId("objective_aggregations", "per-stage-star-evaluation"),
  ],
  sources: [kind === "Knight" ? starFact : kingFact],
  tags: ["result", "stage", kind.toLowerCase()],
  fields: {
    stage_order: order,
    stage_id: stageId,
    stage_kind: kind,
    target_ids: stageTargetSets[stageId],
    terminal_success_required: true,
    target_results_are_attempt_local: true,
    current_progress_projection:
      kind === "KingPlight" ? "ApplyPlightShortcutThenStoreKingResult"
        : "OfferOrCommitCurrentStageResult",
  },
}));

const aggregationSpecs = [
  {
    id: "current-stage-progress",
    nameEn: "Current stage progress",
    nameZh: "当前关卡进度",
    summaryEn:
      "Current progress is the active set of committed stage results and may be reset independently of retained best records.",
    summaryZh:
      "当前进度由当期已提交关卡结果组成，可独立重置且不影响保留的最佳战绩。",
    source: starFact,
    fields: {
      projection_order: 10,
      input_scope: "ActivePeriodCommittedStageResults",
      mutation_sources: [
        "SuccessfulStageResult",
        "AcceptedRecordReplacement",
        "KnightProgressReset",
        "DirectPlightShortcut",
      ],
      retained_best_effect_on_reset: "Unchanged",
    },
  },
  {
    id: "per-stage-star-evaluation",
    nameEn: "Per-stage star total",
    nameZh: "逐关星级合计",
    summaryEn:
      "Each successful attempt produces one non-combined target-satisfaction total for its stage.",
    summaryZh:
      "每次成功尝试为对应关卡生成一份不与其他尝试合并的目标满足合计。",
    source: starFact,
    fields: {
      projection_order: 20,
      input_scope: "OneSuccessfulAttemptTargets",
      output: "SatisfiedTargetCount",
      combine_across_attempts: false,
    },
  },
  {
    id: "simultaneous-three-knight-best",
    nameEn: "Simultaneous three-Knight best",
    nameZh: "三骑士同时生效最佳战绩",
    summaryEn:
      "The best Knight score is the maximum simultaneous sum of the three committed Knight stage star totals.",
    summaryZh:
      "骑士最佳战绩取三份已提交骑士关星级同时生效时的最高合计。",
    source: bestFact,
    fields: {
      projection_order: 30,
      input_stage_ids: [
        "stage.knight-1",
        "stage.knight-2",
        "stage.knight-3",
      ],
      candidate_rule: "SumSimultaneouslyActiveKnightStars",
      retention_rule: "MaximumObservedCandidate",
      maximum_total: 9,
      current_reset_effect: "Unchanged",
    },
  },
  {
    id: "retained-historical-best",
    nameEn: "Retained historical battle records",
    nameZh: "保留的历史最佳战绩",
    summaryEn:
      "Review retains the most recent three period records within the structured 160-day retention locator.",
    summaryZh:
      "战绩回顾保留最近三期记录，并保留结构化的160天留存定位值。",
    source: recordsFact,
    fields: {
      projection_order: 40,
      retained_period_count: 3,
      structured_retention_days: 160,
      expiry_warning_days: 14,
      selection_order: "PeriodDescendingThenStablePeriodId",
      wall_clock_runtime_claim: false,
    },
  },
  {
    id: "king-medal-rating",
    nameEn: "King stage rating",
    nameZh: "王棋关卡评级",
    summaryEn:
      "The King result retains its own satisfied-target rating separately from the three-Knight best.",
    summaryZh:
      "王棋结果独立保留其目标满足评级，不与三骑士最佳战绩合并。",
    source: kingFact,
    fields: {
      projection_order: 50,
      normal_target_ids: stageTargetSets["stage.king-normal"],
      plight_target_ids: stageTargetSets["stage.king-plight"],
      structured_color_medal_target: 6,
      color_medal_target_interpretation: "UnresolvedSourceField",
      account_reward_projection: "Excluded",
      approximations: [{
        field_path: "color_medal_target_interpretation",
        unavailable_fact:
          "ChallengePeakBossConfig exposes ColorMedalTarget=6 without a released semantic enum or join.",
        selected_policy:
          "Retain the exact source value as an uninterpreted locator and do not lower it into a medal formula.",
        alternatives: [
          "treat it as a six-star threshold",
          "treat it as a target ID",
        ],
        rationale:
          "Neither interpretation is supported by an explicit released relationship.",
        affected_fixture_ids: [
          "fixture.target-star-aggregation.king-rating",
        ],
        confidence: "High",
        replacement_condition:
          "Replace when released schema/config or reproducible observations define ColorMedalTarget semantics.",
      }],
    },
  },
];
const aggregationRows = aggregationSpecs.map((spec) => envelope({
  id: `aggregation.${spec.id}`,
  kind: "AggregationRule",
  nameEn: spec.nameEn,
  nameZh: spec.nameZh,
  summaryEn: spec.summaryEn,
  summaryZh: spec.summaryZh,
  ownership: "AnomalyArbitration",
  evidenceQuality: spec.id === "king-medal-rating"
    ? "ApproximateFromReleasedText"
    : "ExactPublicText",
  mechanismQuality: spec.id === "king-medal-rating"
    ? "PolicyBoundary"
    : "ExactRelationship",
  manifestIds: [manifestId("objective_aggregations", spec.id)],
  sources: [
    spec.source,
    ...(spec.id === "king-medal-rating"
      ? [policyRef(
        "king-medal-rating",
        "ColorMedalTarget remains an uninterpreted structured locator.",
      )]
      : []),
  ],
  tags: ["aggregation", "progress", "result"],
  fields: spec.fields,
}));

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
  "targets.json": file("targets.json", "BattleTarget", targetRows),
  "objectives.json": file("objectives.json", "ObjectiveRule", objectiveRows),
  "stage-results.json": file(
    "stage-results.json",
    "StageResultPolicy",
    stageRows,
  ),
  "aggregations.json": file(
    "aggregations.json",
    "AggregationRule",
    aggregationRows,
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
  `Anomaly Arbitration objectives generated: ${targetRows.length} targets, `
    + `${objectiveRows.length} objectives, ${stageRows.length} stage results, `
    + `${aggregationRows.length} aggregations.`,
);
