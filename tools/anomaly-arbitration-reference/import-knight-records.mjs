#!/usr/bin/env node

import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const root = path.resolve(".");
const check = process.argv.includes("--check");
const outputRoot = path.join(
  root,
  "content-reference",
  "anomaly-arbitration-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "content-manifest.json",
), "utf8"));
const officialUrl = "https://www.hoyolab.com/article/41091494";

function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function manifestRef(category, id) {
  const record = manifest.categories[category]?.records.find(
    (candidate) => candidate.id === id,
  );
  if (record === undefined)
    throw new Error(`manifest record is missing: ${category}/${id}`);
  return `${category}:${id}`;
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

function policyRef(category, id, note) {
  const record = manifest.categories[category].records.find(
    (candidate) => candidate.id === id,
  );
  return {
    source_id: `goal13:${category}:${id}`,
    repository_or_url: "starclock",
    revision_or_access_date: "G13-P0-B3 manifest",
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: record.evidence_quality,
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
  evidenceQuality = "ExactPublicText",
  mechanismQuality = "ExactRelationship",
  manifestRecordIds,
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
    manifest_record_ids: [...manifestRecordIds].sort(),
    source_refs: sourceRefs,
    tags: [...tags].sort(),
    ...fields,
    runtime_executable: false,
  };
}

function normalizedFile(file, kind, records) {
  return {
    schema_revision: "starclock.anomaly-arbitration-normalized-file.v1",
    goal_id: "anomaly-arbitration-reference-v1",
    profile: "anomaly-arbitration-v1",
    file,
    record_kind: kind,
    records,
  };
}

const threeTeams = officialRef(
  "three-disjoint-knight-teams",
  "Knight Stage",
  "The three Knight stages use three different character teams, and the player may choose their challenge order.",
);
const retry = officialRef(
  "retry-fixed-team",
  "Retrying Challenges",
  "After a Knight clear, the team composition is recorded and that fixed team may retry the same stage; current character progression states are used on retry.",
);
const crossTeamEquipment = officialRef(
  "cross-team-equipment-reset",
  "Retrying Challenges",
  "Moving a recorded Light Cone or Relic into another Knight retry resets the progress of the cleared Knight record that contained that equipment.",
);
const replaceChoice = officialRef(
  "record-replacement-choice",
  "Retrying Challenges",
  "Using same-team or previously unrecorded Light Cones or Relics permits a retry, and a successful clear offers a choice whether to replace the old record and composition.",
);
const lineupReset = officialRef(
  "lineup-change-reset",
  "Change Lineup",
  "Changing a cleared Knight lineup resets that challenge progress; the corresponding recorded composition is cleared and its stage result becomes invalid.",
);
const bestRecord = officialRef(
  "current-versus-best",
  "Change Lineup and Best Battle Records",
  "Resetting current challenge progress does not affect Best Battle Records, which are calculated from the highest simultaneous total stars across all three Knight stages.",
);

const participantPolicies = [
  envelope({
    id: "participant-policy.three-knight-team-slots",
    kind: "ParticipantPolicy",
    nameEn: "Three disjoint Knight teams",
    nameZh: "三支互斥骑士队伍",
    summaryEn:
      "Each Knight stage owns one recorded team slot, and cleared slots may not share characters.",
    summaryZh:
      "每个骑士关各有一个已记录队伍槽，已通关槽之间不得重复使用角色。",
    manifestRecordIds: [
      manifestRef("participant_policies", "three-knight-team-slots"),
    ],
    sourceRefs: [threeTeams],
    tags: ["knight", "participant", "team"],
    fields: {
      stage_scope: "Knight",
      slot_count: 3,
      slot_ids: ["team-slot.knight-1", "team-slot.knight-2",
        "team-slot.knight-3"],
      distinct_recorded_teams: true,
      king_team_participates_in_uniqueness_scope: false,
    },
  }),
  envelope({
    id: "participant-policy.character-and-combat-form-uniqueness",
    kind: "ParticipantPolicy",
    nameEn: "Character and combat-form uniqueness",
    nameZh: "角色与战斗形态唯一性",
    summaryEn:
      "A character identity cannot occur in two cleared Knight records; multi-form identity remains an explicit reference policy.",
    summaryZh:
      "同一角色身份不能同时出现在两份已通关骑士记录中；多形态身份仍采用显式资料策略。",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    manifestRecordIds: [
      manifestRef(
        "participant_policies",
        "character-and-combat-form-uniqueness",
      ),
    ],
    sourceRefs: [
      threeTeams,
      retry,
      policyRef(
        "participant_policies",
        "character-and-combat-form-uniqueness",
        "The frozen Goal contract requires a fail-closed combat-form identity boundary.",
      ),
    ],
    tags: ["character", "identity", "uniqueness"],
    fields: {
      uniqueness_scope: "AcrossClearedKnightRecords",
      character_identity_key: "stable-character-id",
      combat_form_identity_key: "stable-character-id-plus-path",
      conflict_effect: "ResetConflictingClearedKnightProgress",
      approximations: [{
        field_path: "combat_form_identity_key",
        unavailable_fact:
          "Released instructions describe fixed team members and current retry state but do not define whether alternate Paths share one uniqueness identity.",
        selected_policy:
          "Use character ID plus Path/form as the static authored identity while also rejecting duplicate base character IDs across cleared Knight records.",
        alternatives: [
          "base character ID only",
          "account-owned combatant instance plus Path",
        ],
        rationale:
          "The stricter base-character rejection preserves the released no-overlap rule; the form key keeps the unresolved distinction visible.",
        affected_fixture_ids: [
          "fixture.knight-uniqueness.character-form-conflict",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace when released instructions or reproducible observation explicitly demonstrate multi-Path character identity across two cleared Knight records.",
      }],
    },
  }),
  envelope({
    id: "participant-policy.light-cone-instance-uniqueness",
    kind: "ParticipantPolicy",
    nameEn: "Recorded Light Cone instance uniqueness",
    nameZh: "已记录光锥实例唯一性",
    summaryEn:
      "A recorded Light Cone instance moved across cleared Knight teams invalidates the source record.",
    summaryZh:
      "已记录光锥实例若跨已通关骑士队伍转移，会使来源记录失效。",
    manifestRecordIds: [
      manifestRef(
        "participant_policies",
        "light-cone-instance-uniqueness",
      ),
    ],
    sourceRefs: [crossTeamEquipment],
    tags: ["equipment", "light-cone", "uniqueness"],
    fields: {
      uniqueness_scope: "AcrossClearedKnightRecords",
      identity_key: "account-light-cone-instance-id",
      conflict_effect: "ResetSourceKnightProgress",
    },
  }),
  envelope({
    id: "participant-policy.relic-instance-uniqueness",
    kind: "ParticipantPolicy",
    nameEn: "Recorded Relic instance uniqueness",
    nameZh: "已记录遗器实例唯一性",
    summaryEn:
      "A recorded Relic instance moved across cleared Knight teams invalidates the source record.",
    summaryZh:
      "已记录遗器实例若跨已通关骑士队伍转移，会使来源记录失效。",
    manifestRecordIds: [
      manifestRef("participant_policies", "relic-instance-uniqueness"),
    ],
    sourceRefs: [crossTeamEquipment],
    tags: ["equipment", "relic", "uniqueness"],
    fields: {
      uniqueness_scope: "AcrossClearedKnightRecords",
      identity_key: "account-relic-instance-id",
      conflict_effect: "ResetSourceKnightProgress",
    },
  }),
];

const teamSlots = [1, 2, 3].map((slotOrder) => envelope({
  id: `team-slot.knight-${slotOrder}`,
  kind: "TeamSlot",
  nameEn: `Knight team ${slotOrder}`,
  nameZh: `骑士队伍${slotOrder}`,
  summaryEn:
    `Knight team ${slotOrder} records the participants and equipment used to clear its matching stage.`,
  summaryZh:
    `骑士队伍${slotOrder}记录对应关卡通关时使用的参与者与装备。`,
  manifestRecordIds: [
    manifestRef("participant_policies", "three-knight-team-slots"),
  ],
  sourceRefs: [threeTeams, retry],
  tags: ["knight", "slot", "team"],
  fields: {
    slot_order: slotOrder,
    stage_id: `stage.knight-${slotOrder}`,
    record_state: "EmptyOrSuccessfulClearSnapshot",
    conflict_scope: "AllClearedKnightTeamSlots",
  },
}));

const loadoutRecords = [
  envelope({
    id: "loadout-record.successful-clear-snapshot",
    kind: "LoadoutRecordPolicy",
    nameEn: "Successful Knight clear snapshot",
    nameZh: "骑士通关编队快照",
    summaryEn:
      "A successful Knight clear records its team composition for later retries.",
    summaryZh:
      "骑士关成功通关后会记录编队情况，供之后重复挑战使用。",
    manifestRecordIds: [
      manifestRef("record_progress_lifecycles", "successful-knight-record"),
    ],
    sourceRefs: [retry],
    tags: ["loadout", "record", "snapshot"],
    fields: {
      creation_trigger: "SuccessfulKnightClear",
      participant_membership: "FixedForRetry",
      progression_resolution: "LatestStatusAtRetry",
      equipment_identity_resolution: "RecordedAccountInstance",
    },
  }),
  envelope({
    id: "loadout-record.cross-team-equipment-invalidation",
    kind: "LoadoutRecordPolicy",
    nameEn: "Cross-team equipment invalidation",
    nameZh: "跨队装备记录失效",
    summaryEn:
      "Reusing recorded equipment in another Knight retry resets the cleared record that supplied it.",
    summaryZh:
      "在另一骑士重试中复用已记录装备，会重置提供该装备的已通关记录。",
    manifestRecordIds: [
      manifestRef(
        "record_progress_lifecycles",
        "loadout-change-invalidation",
      ),
    ],
    sourceRefs: [crossTeamEquipment],
    tags: ["invalidation", "loadout", "reset"],
    fields: {
      trigger:
        "RecordedLightConeOrRelicAssignedToDifferentKnightRetry",
      invalidated_scope: "SourceClearedKnightRecord",
      ordering: [
        "DetectRecordedInstanceConflict",
        "ResetSourceKnightProgress",
        "ClearSourceTeamSnapshot",
        "PermitTargetChallengeIfNoConflictsRemain",
      ],
    },
  }),
  envelope({
    id: "loadout-record.same-team-or-unrecorded-change",
    kind: "LoadoutRecordPolicy",
    nameEn: "Nonconflicting loadout retry",
    nameZh: "无冲突配装重试",
    summaryEn:
      "Same-team or previously unrecorded equipment permits a retry without first erasing the old record.",
    summaryZh:
      "同队装备或未出现在任何通关记录中的装备可直接重试，无须先清除旧记录。",
    manifestRecordIds: [
      manifestRef("record_progress_lifecycles", "rechallenge-eligibility"),
      manifestRef(
        "record_progress_lifecycles",
        "record-replacement-choice",
      ),
    ],
    sourceRefs: [replaceChoice],
    tags: ["loadout", "replacement", "retry"],
    fields: {
      eligible_change_classes: [
        "EquipmentFromSameRecordedTeam",
        "EquipmentAbsentFromAllClearRecords",
      ],
      retry_allowed: true,
      old_record_retained_during_attempt: true,
      success_resolution: "ExplicitKeepOldOrReplaceWithNewChoice",
      failed_attempt_resolution: "KeepOldRecord",
    },
  }),
];

const progressSpecs = [
  {
    id: "successful-knight-record",
    nameEn: "Successful Knight record",
    nameZh: "骑士成功记录",
    summaryEn:
      "Only a successful Knight clear creates a reusable team-composition record.",
    summaryZh: "只有骑士关成功通关才会创建可复用的编队记录。",
    projectionOrder: 10,
    source: retry,
    fields: {
      trigger: "SuccessfulKnightClear",
      current_projection: "CreateOrOfferReplacement",
      best_projection: "EvaluateSimultaneousKnightStars",
    },
  },
  {
    id: "rechallenge-eligibility",
    nameEn: "Knight re-challenge eligibility",
    nameZh: "骑士重复挑战资格",
    summaryEn:
      "A recorded team may retry its Knight stage, subject to equipment-conflict checks.",
    summaryZh: "已记录队伍可重复挑战对应骑士关，但须先检查装备冲突。",
    projectionOrder: 20,
    source: retry,
    fields: {
      trigger: "RequestKnightRetry",
      preconditions: [
        "RecordedTeamMembersUnchanged",
        "EquipmentConflictRulesSatisfiedOrResolved",
      ],
      accepted_state: "AttemptReady",
    },
  },
  {
    id: "loadout-change-invalidation",
    nameEn: "Loadout conflict invalidation",
    nameZh: "配装冲突失效",
    summaryEn:
      "A cross-team recorded-equipment conflict resets the supplying Knight's current progress.",
    summaryZh:
      "跨队已记录装备冲突会重置装备来源骑士关的当前进度。",
    projectionOrder: 30,
    source: crossTeamEquipment,
    fields: {
      trigger: "CrossTeamRecordedEquipmentConflict",
      current_projection: "ResetSourceKnightProgress",
      best_projection: "Unchanged",
    },
  },
  {
    id: "record-replacement-choice",
    nameEn: "Record replacement choice",
    nameZh: "战绩替换选择",
    summaryEn:
      "After a nonconflicting successful retry, the player explicitly keeps the old record or installs the new one.",
    summaryZh:
      "无冲突重试成功后，玩家明确选择保留旧记录或写入新记录。",
    projectionOrder: 40,
    source: replaceChoice,
    fields: {
      trigger: "SuccessfulEligibleRetry",
      legal_choices: ["KeepOldRecord", "ReplaceWithNewRecord"],
      rejection_effect: "AuthoritativeStateUnchanged",
    },
  },
  {
    id: "record-erasure-on-reset",
    nameEn: "Record erasure on reset",
    nameZh: "重置时清除记录",
    summaryEn:
      "Resetting a Knight's current progress clears its recorded composition and invalidates that stage result.",
    summaryZh:
      "重置骑士关当前进度会清除已记录编队，并使该关成绩失效。",
    projectionOrder: 50,
    source: lineupReset,
    fields: {
      trigger: "ResetKnightChallengeProgress",
      ordered_projections: [
        "ClearRecordedTeamComposition",
        "InvalidateCurrentStageResult",
        "ReevaluateCurrentProgress",
      ],
      best_projection: "Unchanged",
    },
  },
  {
    id: "current-versus-best-progress",
    nameEn: "Current and Best Battle Records",
    nameZh: "当前进度与最佳战绩",
    summaryEn:
      "Current Knight progress may reset independently while Best Battle Records preserve the highest simultaneous three-stage star total.",
    summaryZh:
      "骑士关当前进度可独立重置，最佳战绩则保留三关同时生效时的最高总星数。",
    projectionOrder: 60,
    source: bestRecord,
    fields: {
      current_progress_scope: "ActivePeriodThreeKnightRecords",
      best_progress_scope: "HighestSimultaneousThreeKnightStarTotal",
      current_reset_affects_best: false,
      detailed_aggregation_owner_batch: "G13-P1-B6",
    },
  },
];
const progressRecords = progressSpecs.map((spec) => envelope({
  id: `progress-record.${spec.id}`,
  kind: "ProgressRecordPolicy",
  nameEn: spec.nameEn,
  nameZh: spec.nameZh,
  summaryEn: spec.summaryEn,
  summaryZh: spec.summaryZh,
  manifestRecordIds: [
    manifestRef("record_progress_lifecycles", spec.id),
  ],
  sourceRefs: [spec.source],
  tags: ["knight", "progress", "record"],
  fields: {
    projection_order: spec.projectionOrder,
    ...spec.fields,
  },
}));

const outputs = {
  "participant-policies.json": normalizedFile(
    "participant-policies.json",
    "ParticipantPolicy",
    participantPolicies,
  ),
  "team-slots.json": normalizedFile(
    "team-slots.json",
    "TeamSlot",
    teamSlots,
  ),
  "loadout-records.json": normalizedFile(
    "loadout-records.json",
    "LoadoutRecordPolicy",
    loadoutRecords,
  ),
  "progress-records.json": normalizedFile(
    "progress-records.json",
    "ProgressRecordPolicy",
    progressRecords,
  ),
};

await mkdir(outputRoot, { recursive: true });
for (const [file, document] of Object.entries(outputs)) {
  const bytes = `${JSON.stringify(document, null, 2)}\n`;
  const outputPath = path.join(outputRoot, file);
  if (check) {
    const existing = await readFile(outputPath, "utf8").catch(() => "");
    if (existing !== bytes) throw new Error(`${file} generation drift`);
  } else {
    await writeFile(outputPath, bytes);
  }
}

console.log(
  "Anomaly Arbitration Knight records generated: "
    + `${participantPolicies.length} policies, ${teamSlots.length} slots, `
    + `${loadoutRecords.length} loadout rules, `
    + `${progressRecords.length} progress rules.`,
);
