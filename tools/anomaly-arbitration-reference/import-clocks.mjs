#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const output = path.join(
  root,
  "content-reference/anomaly-arbitration-v1/clocks.json",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
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

function officialRef(id, fact) {
  return {
    source_id: `official:hoyolab:anomaly-arbitration:${id}`,
    repository_or_url: officialUrl,
    revision_or_access_date: "accessed 2026-07-29",
    game_version: "4.4",
    path_or_page: officialUrl,
    locator: "Cycles",
    sha256: digest(fact),
    evidence_quality: "ExactPublicText",
    mechanism_quality: "ExactRelationship",
    note: fact,
  };
}

function policyRef(id, note) {
  const record = manifest.categories.clock_rules.records.find(
    (candidate) => candidate.id === id,
  );
  return {
    source_id: `goal13:clock-rules:${id}`,
    repository_or_url: "starclock",
    revision_or_access_date: "G13-P0-B3 manifest",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ProjectPolicy",
    mechanism_quality: "PolicyBoundary",
    note,
  };
}

function constantRef(name, note) {
  const id = `constant:${name}`;
  const record = manifest.categories.mode_constants.records.find(
    (candidate) => candidate.id === id,
  );
  return {
    source_id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: "ExactRelationship",
    note,
  };
}

function approximation(fieldPath, unavailableFact, selectedPolicy,
  alternatives, rationale, fixture, confidence, replacementCondition) {
  return {
    field_path: fieldPath,
    unavailable_fact: unavailableFact,
    selected_policy: selectedPolicy,
    alternatives,
    rationale,
    affected_fixture_ids: [fixture],
    confidence,
    replacement_condition: replacementCondition,
  };
}

function row({
  id,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  stageKind,
  order,
  evidenceQuality,
  mechanismQuality,
  obligation,
  extraManifest = [],
  sources,
  tags,
  fields,
}) {
  return {
    id,
    schema_revision: "starclock.anomaly-arbitration-row.v1",
    kind: "ClockRule",
    name_en: nameEn,
    name_zh_cn: nameZh,
    summary_en: summaryEn,
    summary_zh_cn: summaryZh,
    ownership: "AnomalyArbitration",
    coverage_state: "DataReady",
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [
      manifestId("clock_rules", obligation),
      ...extraManifest,
    ].sort(),
    source_refs: sources,
    tags: [...tags].sort(),
    stage_kind: stageKind,
    boundary_order: order,
    ...fields,
    runtime_executable: false,
  };
}

const eachCombat = officialRef(
  "independent-cycle-limits",
  "Each combat in each Anomaly Arbitration stage has its own cycle limit, and exceeding it fails the attempt.",
);
const firstCycle = officialRef(
  "first-cycle-action-value",
  "The first cycle has greatly increased total action value.",
);
const waveCarry = officialRef(
  "wave-transition-carry",
  "The cycle countdown does not reset when combat phases or waves change.",
);
const lowCycle = officialRef(
  "low-cycle-allied-buff",
  "When few cycles remain, allies receive additional combat buffs at the start of each cycle.",
);
const retryGuide = {
  ...officialRef(
    "retrying-challenges",
    "A recorded Knight team may retry its stage after a completed challenge.",
  ),
  locator: "Retrying Challenges",
};

const records = [
  row({
    id: "clock.first-cycle-action-value",
    nameEn: "Expanded first cycle",
    nameZh: "扩展首轮",
    summaryEn:
      "Every stage attempt begins with a first cycle whose total action value exceeds later cycles.",
    summaryZh:
      "每次关卡尝试的首轮总行动值高于后续轮次。",
    stageKind: "All",
    order: 10,
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "PolicyBoundary",
    obligation: "first-cycle-action-value",
    sources: [
      firstCycle,
      policyRef(
        "first-cycle-action-value",
        "The released guide is qualitative and no numeric action-value constant was found.",
      ),
    ],
    tags: ["action-value", "cycle", "first-cycle"],
    fields: {
      cycle_index: 1,
      qualitative_rule: "GreaterTotalActionValueThanLaterCycles",
      numeric_action_value: "Unavailable",
      approximations: [approximation(
        "numeric_action_value",
        "Released evidence says the first-cycle total is greatly increased but supplies no numeric action value.",
        "Retain the qualitative ordering and leave the numeric value unavailable.",
        ["assume standard 150 action value", "assume 200 action value"],
        "No numeric value is safer than choosing an unobserved timing constant.",
        "fixture.clock-first-cycle.qualitative-only",
        "High",
        "Replace with a released configuration value or reproducible cycle/action-value trace.",
      )],
    },
  }),
  row({
    id: "clock.knight-cycle-limit",
    nameEn: "Knight stage cycle limit",
    nameZh: "骑士关轮次上限",
    summaryEn: "Each Knight attempt has a stage-local limit of six cycles.",
    summaryZh: "每次骑士关尝试具有独立的六轮上限。",
    stageKind: "Knight",
    order: 20,
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    obligation: "knight-cycle-limit",
    extraManifest: [
      manifestId(
        "mode_constants",
        "constant:ChallengePeak_Mob_Turn_Limit",
      ),
    ],
    sources: [
      constantRef(
        "ChallengePeak_Mob_Turn_Limit",
        "Pinned released Knight-stage turn limit.",
      ),
      eachCombat,
    ],
    tags: ["cycle-limit", "knight"],
    fields: {
      limit_cycles: 6,
      scope: "OneKnightStageAttempt",
      reset_on_wave_transition: false,
    },
  }),
  row({
    id: "clock.normal-king-cycle-limit",
    nameEn: "Normal King cycle limit",
    nameZh: "常规王棋轮次上限",
    summaryEn: "Each normal King attempt has a stage-local limit of six cycles.",
    summaryZh: "每次常规王棋尝试具有独立的六轮上限。",
    stageKind: "KingNormal",
    order: 30,
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    obligation: "boss-cycle-limit",
    extraManifest: [
      manifestId(
        "mode_constants",
        "constant:ChallengePeak_Boss_Turn_Limit",
      ),
    ],
    sources: [
      constantRef(
        "ChallengePeak_Boss_Turn_Limit",
        "Pinned released normal King turn limit.",
      ),
      eachCombat,
    ],
    tags: ["cycle-limit", "king", "normal"],
    fields: {
      limit_cycles: 6,
      scope: "OneNormalKingStageAttempt",
      reset_on_wave_transition: false,
    },
  }),
  row({
    id: "clock.plight-cycle-limit",
    nameEn: "Plight King cycle limit",
    nameZh: "困厄王棋轮次上限",
    summaryEn: "Each direct Plight attempt has a stage-local limit of two cycles.",
    summaryZh: "每次直接困厄王棋尝试具有独立的两轮上限。",
    stageKind: "KingPlight",
    order: 40,
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    obligation: "plight-cycle-limit",
    extraManifest: [
      manifestId(
        "mode_constants",
        "constant:ChallengePeak_HardBoss_Turn_Limit",
      ),
    ],
    sources: [
      constantRef(
        "ChallengePeak_HardBoss_Turn_Limit",
        "Pinned released Plight King turn limit.",
      ),
      eachCombat,
    ],
    tags: ["cycle-limit", "king", "plight"],
    fields: {
      limit_cycles: 2,
      scope: "OnePlightKingStageAttempt",
      reset_on_wave_transition: false,
    },
  }),
  row({
    id: "clock.wave-transition-carry",
    nameEn: "Cycle carry across waves",
    nameZh: "跨波次轮次延续",
    summaryEn:
      "The stage-local cycle countdown continues across wave or phase transitions.",
    summaryZh: "关卡局部轮次倒计时在波次或阶段切换时继续延续。",
    stageKind: "All",
    order: 50,
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "ExactRelationship",
    obligation: "wave-transition-carry",
    sources: [waveCarry],
    tags: ["carry", "cycle", "wave"],
    fields: {
      transition: "WaveOrPhaseChange",
      countdown_projection: "CarryRemainingCycles",
      reset_on_wave_transition: false,
    },
  }),
  row({
    id: "clock.warning-threshold",
    nameEn: "Low-cycle warning threshold",
    nameZh: "低轮次预警阈值",
    summaryEn:
      "A low-cycle boundary exists, but its exact remaining-cycle threshold is not exposed.",
    summaryZh: "玩法存在低轮次边界，但公开资料未给出精确剩余轮次阈值。",
    stageKind: "All",
    order: 60,
    evidenceQuality: "ApproximateFromReleasedText",
    mechanismQuality: "PolicyBoundary",
    obligation: "warning-threshold",
    sources: [
      lowCycle,
      policyRef(
        "warning-threshold",
        "The warning boundary is preserved without an invented threshold.",
      ),
    ],
    tags: ["cycle", "threshold", "warning"],
    fields: {
      threshold_cycles_remaining: "Unavailable",
      warning_state: "FewCyclesRemain",
      approximations: [approximation(
        "threshold_cycles_remaining",
        "Released text says only that few cycles remain.",
        "Keep a symbolic FewCyclesRemain boundary and author no numeric threshold.",
        ["one cycle remaining", "two cycles remaining"],
        "A symbolic state remains testable without presenting an arbitrary number as parity.",
        "fixture.clock-warning-expiry.symbolic-threshold",
        "High",
        "Replace with released configuration or a reproducible observation at the transition boundary.",
      )],
    },
  }),
  row({
    id: "clock.low-cycle-combat-effect",
    nameEn: "Low-cycle allied combat effect",
    nameZh: "低轮次我方战斗增益",
    summaryEn:
      "While the low-cycle state is active, allies receive an additional buff at each cycle start.",
    summaryZh:
      "低轮次状态生效期间，每轮开始时我方获得额外战斗增益。",
    stageKind: "All",
    order: 70,
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "PolicyBoundary",
    obligation: "low-cycle-combat-effect",
    sources: [
      lowCycle,
      policyRef(
        "low-cycle-combat-effect",
        "Buff identity and parameters remain unavailable in released evidence.",
      ),
    ],
    tags: ["allies", "buff", "cycle-start"],
    fields: {
      trigger: "CycleStartWhileFewCyclesRemain",
      target: "Allies",
      contribution: "AdditionalCombatBuff",
      buff_id: "Unavailable",
      numeric_parameters: "Unavailable",
      approximations: [approximation(
        "buff_id",
        "Released text does not identify the buff row or numeric parameters.",
        "Retain only trigger, target and qualitative contribution.",
        ["bind a generic damage buff", "bind a generic speed buff"],
        "No authored combat effect may be invented by a reference-only goal.",
        "fixture.clock-warning-expiry.low-cycle-buff-boundary",
        "High",
        "Replace with the released buff selector and its ability/config program.",
      )],
    },
  }),
  row({
    id: "clock.expiry-and-failure",
    nameEn: "Cycle expiry failure",
    nameZh: "轮次耗尽失败",
    summaryEn:
      "Exceeding the stage-local cycle limit terminates the current attempt as a failure.",
    summaryZh: "超过关卡局部轮次上限会以失败终止当前尝试。",
    stageKind: "All",
    order: 80,
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "ExactRelationship",
    obligation: "expiry-and-failure-boundary",
    sources: [eachCombat],
    tags: ["expiry", "failure", "terminal"],
    fields: {
      boundary: "CycleLimitExceeded",
      terminal_outcome_id: "outcome.stage-attempt-failure",
      current_record_projection: "Unchanged",
      best_record_projection: "Unchanged",
    },
  }),
  row({
    id: "clock.retry-boundary",
    nameEn: "Retry clock boundary",
    nameZh: "重试时钟边界",
    summaryEn:
      "A retry creates a new stage attempt clock and does not carry elapsed cycles from the failed attempt.",
    summaryZh: "重试会创建新的关卡尝试时钟，不继承失败尝试已经消耗的轮次。",
    stageKind: "All",
    order: 90,
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    obligation: "retry-boundary",
    sources: [
      retryGuide,
      policyRef(
        "retry-boundary",
        "Released instructions permit retry but do not state clock-object identity.",
      ),
    ],
    tags: ["attempt", "clock", "retry"],
    fields: {
      trigger: "AcceptedRetryRequest",
      prior_clock_state: "TerminalAndImmutable",
      new_clock_state: "FreshStageLocalClock",
      carry_elapsed_cycles: false,
      approximations: [approximation(
        "new_clock_state",
        "Released instructions permit retry but do not specify authoritative clock allocation order.",
        "Create a fresh stage-local clock only after the retry request is accepted.",
        ["reuse and reset the prior clock object"],
        "Fresh attempt identity keeps failed state immutable and makes rejection byte-stable.",
        "fixture.clock-warning-expiry.retry-boundary",
        "Medium",
        "Replace with released transition evidence or a reproducible retry trace.",
      )],
    },
  }),
];

const document = {
  schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
  goal_id: "anomaly-arbitration-reference-v1",
  profile: "anomaly-arbitration-v1",
  file: "clocks.json",
  record_kind: "ClockRule",
  records,
};
const bytes = `${JSON.stringify(document, null, 2)}\n`;
await mkdir(path.dirname(output), { recursive: true });
if (check) {
  const existing = await readFile(output, "utf8").catch(() => "");
  if (existing !== bytes) throw new Error("clocks.json generation drift");
} else {
  await writeFile(output, bytes);
}
console.log(`Anomaly Arbitration clocks generated: ${records.length} rules.`);
