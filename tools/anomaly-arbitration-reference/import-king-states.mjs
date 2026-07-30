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
const officialUrl = "https://www.hoyolab.com/article/41091494";

function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function obligation(id) {
  const record = manifest.categories.king_state_transitions.records.find(
    (candidate) => candidate.id === id,
  );
  if (record === undefined) throw new Error(`missing King obligation: ${id}`);
  return `king_state_transitions:${id}`;
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
  const record = manifest.categories.king_state_transitions.records.find(
    (candidate) => candidate.id === id,
  );
  return {
    source_id: `goal13:king-state-transitions:${id}`,
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

function envelope({
  id,
  kind,
  nameEn,
  nameZh,
  summaryEn,
  summaryZh,
  evidenceQuality,
  mechanismQuality,
  obligationId,
  sourceRefs,
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
    ownership: "AnomalyArbitration",
    coverage_state: "DataReady",
    evidence_quality: evidenceQuality,
    mechanism_quality: mechanismQuality,
    manifest_record_ids: [obligation(obligationId)],
    source_refs: sourceRefs,
    tags: [...tags].sort(),
    ...fields,
    runtime_executable: false,
  };
}

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

const protectedKing = officialRef(
  "knight-protection",
  "How to Play and King in Check Stage",
  "Knight-stage protection greatly enhances the King; the guide recommends clearing all three Knights to cut their energy transmission before the King challenge.",
);
const plightShortcut = officialRef(
  "direct-plight-clear",
  "King in Check Stage",
  "Directly defeating the Plight King counts as three-star clearing all Knight stages.",
);
const structuredNormal = {
  source_id: "turnbasedgamedata:StageConfig:30508021",
  repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
  revision_or_access_date: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  game_version: "4.4",
  path_or_page: "ExcelOutput/StageConfig.json",
  locator: "StageID=30508021",
  sha256: manifest.categories.stage_configs.records.find(
    ({ id }) => id === "stage:30508021",
  ).evidence_sha256,
  evidence_quality: "ExactStructured",
  mechanism_quality: "ExactRelationship",
  note: "Released normal King StageConfig selected by active alias 804.",
};
const structuredPlight = {
  source_id: "turnbasedgamedata:StageConfig:30508022",
  repository_or_url: "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
  revision_or_access_date: "fd978d6ef09f941fba644c731ab54abd6f7c3568",
  game_version: "4.4",
  path_or_page: "ExcelOutput/StageConfig.json",
  locator: "StageID=30508022",
  sha256: manifest.categories.stage_configs.records.find(
    ({ id }) => id === "stage:30508022",
  ).evidence_sha256,
  evidence_quality: "ExactStructured",
  mechanism_quality: "ExactRelationship",
  note: "Released Plight King StageConfig selected by active alias 804.",
};

const kingStates = [
  envelope({
    id: "king-state.normal",
    kind: "KingState",
    nameEn: "Normal King state",
    nameZh: "常规王棋状态",
    summaryEn:
      "The normal King reference selects StageConfig 30508021 after all three Knight transmissions are cut.",
    summaryZh:
      "常规王棋资料在三路骑士能量输送全部阻断后选择 StageConfig 30508021。",
    evidenceQuality: "ApproximateFromReleasedText",
    mechanismQuality: "PolicyBoundary",
    obligationId: "normal-king-state",
    sourceRefs: [
      structuredNormal,
      protectedKing,
      policyRef(
        "normal-king-state",
        "The exact normal-difficulty availability transition is retained as a replaceable policy.",
      ),
    ],
    tags: ["king", "normal", "state"],
    fields: {
      state_order: 20,
      stage_id: "stage.king-normal",
      source_stage_id: "30508021",
      required_cleared_knight_count: 3,
      protection_contribution_count: 0,
      availability: "AfterAllThreeKnightClears",
      approximations: [{
        field_path: "availability",
        unavailable_fact:
          "Official released text recommends cutting all three transmissions but does not define the normal-stage unlock predicate.",
        selected_policy:
          "Expose normal King only after all three current Knight clears.",
        alternatives: [
          "normal King always selectable",
          "normal King weakens incrementally after each Knight clear",
        ],
        rationale:
          "This matches released independent observation and keeps the official recommendation fail-closed.",
        affected_fixture_ids: [
          "fixture.king-protection.normal-unlock",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace when released structured data or official instructions state the normal King availability selector.",
      }],
    },
  }),
  envelope({
    id: "king-state.plight",
    kind: "KingState",
    nameEn: "Plight King state",
    nameZh: "困厄王棋状态",
    summaryEn:
      "The direct Plight alternative selects StageConfig 30508022 while Knight protection remains active.",
    summaryZh:
      "直接挑战困厄难度时，在骑士保护仍生效的情况下选择 StageConfig 30508022。",
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "ExactRelationship",
    obligationId: "plight-state",
    sourceRefs: [structuredPlight, protectedKing, plightShortcut],
    tags: ["king", "plight", "state"],
    fields: {
      state_order: 10,
      stage_id: "stage.king-plight",
      source_stage_id: "30508022",
      availability: "DirectAlternative",
      protection_state: "ActiveKnightProtection",
      direct_clear_shortcut_id: "king-protection.direct-plight-shortcut",
    },
  }),
];

const kingProtection = [
  envelope({
    id: "king-protection.composition",
    kind: "KingProtectionRule",
    nameEn: "Knight protection composition",
    nameZh: "骑士保护组成",
    summaryEn:
      "Three named Knight-clear contributions form the auditable protection boundary around the King.",
    summaryZh:
      "三条具名骑士通关贡献共同构成王棋保护的可审计边界。",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    obligationId: "king-protection-composition",
    sourceRefs: [
      protectedKing,
      policyRef(
        "king-protection-composition",
        "The released guide names three transmissions but does not expose their numeric composition.",
      ),
    ],
    tags: ["composition", "king", "protection"],
    fields: {
      contribution_order: 10,
      contribution_ids: [
        "knight-clear-contribution.knight-1",
        "knight-clear-contribution.knight-2",
        "knight-clear-contribution.knight-3",
      ],
      active_contribution_rule:
        "OneProtectionContributionPerUnclearedKnightStage",
      numeric_effects: "Unavailable",
      approximations: [{
        field_path: "active_contribution_rule",
        unavailable_fact:
          "Released text does not state whether protection is one aggregate gate or three independently weakening contributions.",
        selected_policy:
          "Track one boolean transmission per Knight solely for lifecycle accounting; make no numeric stacking claim.",
        alternatives: [
          "single protection gate removed after three clears",
          "three distinct numeric protection stacks",
        ],
        rationale:
          "Named per-Knight booleans support reset and teardown auditing without inventing combat values.",
        affected_fixture_ids: [
          "fixture.king-protection.three-contributions",
        ],
        confidence: "Low",
        replacement_condition:
          "Replace with released ability/config selectors or reproducible observations that identify contribution composition.",
      }],
    },
  }),
  envelope({
    id: "king-protection.knight-clear-contribution",
    kind: "KingProtectionRule",
    nameEn: "Knight clear contribution",
    nameZh: "骑士通关贡献",
    summaryEn:
      "A current Knight clear cuts that stage's named transmission in the reference lifecycle.",
    summaryZh:
      "骑士关当前通关会在资料生命周期中阻断该关具名能量输送。",
    evidenceQuality: "ApproximateFromReleasedText",
    mechanismQuality: "PolicyBoundary",
    obligationId: "knight-clear-contribution",
    sourceRefs: [
      protectedKing,
      policyRef(
        "knight-clear-contribution",
        "Per-Knight teardown ordering is explicit and replaceable.",
      ),
    ],
    tags: ["clear", "knight", "protection"],
    fields: {
      contribution_order: 20,
      trigger: "CurrentKnightClearCommitted",
      projection: "DeactivateMatchingKnightTransmission",
      evaluation_order: [
        "CommitKnightClear",
        "DeactivateMatchingTransmission",
        "RecomputeKingAvailability",
      ],
      approximations: [{
        field_path: "projection",
        unavailable_fact:
          "Official text says to complete all three and cut their transmissions but does not expose the per-clear state update.",
        selected_policy:
          "Deactivate the matching boolean contribution after its successful clear.",
        alternatives: [
          "deactivate all protection only after the third clear",
        ],
        rationale:
          "The policy preserves a deterministic auditable transition without numeric combat claims.",
        affected_fixture_ids: [
          "fixture.king-protection.knight-clear",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace when released evidence identifies the exact per-clear protection update.",
      }],
    },
  }),
  envelope({
    id: "king-protection.reset-and-teardown",
    kind: "KingProtectionRule",
    nameEn: "Protection reset and teardown",
    nameZh: "保护重置与拆除",
    summaryEn:
      "Resetting a current Knight clear restores its transmission before King availability is reevaluated.",
    summaryZh:
      "重置骑士关当前通关时，先恢复该关能量输送，再重新评估王棋可用状态。",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    obligationId: "protection-removal-and-teardown",
    sourceRefs: [
      protectedKing,
      policyRef(
        "protection-removal-and-teardown",
        "The Goal lifecycle requires explicit reset and teardown order.",
      ),
    ],
    tags: ["king", "reset", "teardown"],
    fields: {
      contribution_order: 30,
      clear_transition: "DeactivateMatchingKnightTransmission",
      reset_transition: "ReactivateMatchingKnightTransmission",
      ordered_reset_projection: [
        "InvalidateCurrentKnightClear",
        "ReactivateMatchingKnightTransmission",
        "RevokeNormalKingAvailabilityIfAnyTransmissionActive",
      ],
      best_battle_record_effect: "Unchanged",
      approximations: [{
        field_path: "ordered_reset_projection",
        unavailable_fact:
          "Released text separates current reset from Best Battle Records but does not state King-protection reset ordering.",
        selected_policy:
          "Restore the matching transmission atomically with current-record invalidation and reevaluate normal availability afterward.",
        alternatives: [
          "retain removed protection after current reset",
          "defer protection restoration until another King attempt",
        ],
        rationale:
          "Restoration mirrors current-state loss while preserving the exact Best Battle Record separation.",
        affected_fixture_ids: [
          "fixture.king-protection.reset-restores-transmission",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace with released transition evidence covering a Knight reset after normal King becomes available.",
      }],
    },
  }),
  envelope({
    id: "king-protection.direct-plight-shortcut",
    kind: "KingProtectionRule",
    nameEn: "Direct Plight clear shortcut",
    nameZh: "困厄直通捷径",
    summaryEn:
      "A direct Plight clear projects three-star equivalence to all Knight stages without claiming account rewards.",
    summaryZh:
      "直接通关困厄王棋会向全部骑士关投影三星等价结果，但不纳入账号奖励。",
    evidenceQuality: "ExactPublicText",
    mechanismQuality: "ExactRelationship",
    obligationId: "direct-plight-clear-shortcut",
    sourceRefs: [
      plightShortcut,
      policyRef(
        "direct-plight-clear-shortcut",
        "Snapshot creation remains a separately labeled boundary because released text states result equivalence only.",
      ),
    ],
    tags: ["king", "plight", "shortcut"],
    fields: {
      contribution_order: 40,
      trigger: "SuccessfulPlightKingClear",
      exact_projection: {
        knight_stage_ids: [
          "stage.knight-1",
          "stage.knight-2",
          "stage.knight-3",
        ],
        stars_each: 3,
        result_equivalence: "ThreeStarKnightClear",
      },
      account_reward_projection: "Excluded",
      loadout_snapshot_projection: "NoSyntheticKnightTeamSnapshots",
      downstream_order: [
        "CommitPlightKingClear",
        "ProjectThreeStarKnightEquivalence",
        "EvaluateCurrentAndBestProgress",
        "SettleMechanicalResults",
      ],
      approximations: [{
        field_path: "loadout_snapshot_projection",
        unavailable_fact:
          "Released text states three-star clear equivalence but does not state whether synthetic Knight team snapshots are created.",
        selected_policy:
          "Do not fabricate team snapshots for Knight battles that did not occur.",
        alternatives: [
          "copy the Plight team into all Knight snapshots",
          "create empty synthetic Knight snapshots",
        ],
        rationale:
          "Failing closed preserves equipment uniqueness and avoids inventing three battle records.",
        affected_fixture_ids: [
          "fixture.king-protection.plight-shortcut",
        ],
        confidence: "High",
        replacement_condition:
          "Replace if released battle-record observation shows synthetic Knight compositions after a direct Plight clear.",
      }],
    },
  }),
];

const outputs = {
  "king-states.json": file("king-states.json", "KingState", kingStates),
  "king-protection.json": file(
    "king-protection.json",
    "KingProtectionRule",
    kingProtection,
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
  `Anomaly Arbitration King states generated: ${kingStates.length} states, `
    + `${kingProtection.length} protection rules.`,
);
