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
const sourceRevision = "fd978d6ef09f941fba644c731ab54abd6f7c3568";

function digest(value) {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex");
}

function manifestId(id) {
  if (!manifest.categories.quadrant_options.records.some(
    (record) => record.id === id,
  )) throw new Error(`missing Quadrant obligation: ${id}`);
  return `quadrant_options:${id}`;
}

function structuredRef(id, note) {
  const record = manifest.categories.quadrant_options.records.find(
    (candidate) => candidate.id === `quadrant:${id}`,
  );
  return {
    source_id: `turnbasedgamedata:${record.source_path}:${record.row_locator}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: sourceRevision,
    game_version: "4.4",
    path_or_page: record.source_path,
    locator: record.row_locator,
    sha256: record.evidence_sha256,
    evidence_quality: "ExactStructured",
    mechanism_quality: "ExactRelationship",
    note,
  };
}

function textRef(locale, hash, value) {
  const pathName = locale === "zh_cn"
    ? "TextMap/TextMapCHS.json"
    : "TextMap/TextMapEN.json";
  return {
    source_id: `turnbasedgamedata:${pathName}:Hash=${hash}`,
    repository_or_url:
      "https://gitlab.com/Dimbreath/turnbasedgamedata.git",
    revision_or_access_date: sourceRevision,
    game_version: "4.4",
    path_or_page: pathName,
    locator: `Hash=${hash}`,
    sha256: digest({ hash, value }),
    evidence_quality: "ExactStructured",
    mechanism_quality: "IdentityCrossCheck",
    note: `Exact ${locale} released text for the referenced hash.`,
  };
}

function officialRef(id, fact) {
  return {
    source_id: `official:hoyolab:anomaly-arbitration:${id}`,
    repository_or_url: officialUrl,
    revision_or_access_date: "accessed 2026-07-29",
    game_version: "4.4",
    path_or_page: officialUrl,
    locator: "Arbitral Quadrant",
    sha256: digest(fact),
    evidence_quality: "ExactPublicText",
    mechanism_quality: "ExactRelationship",
    note: fact,
  };
}

function policyRef(id, note) {
  return {
    source_id: `goal13:quadrant-policy:${id}`,
    repository_or_url: "starclock",
    revision_or_access_date: "G13-P1-B5",
    game_version: "4.4",
    path_or_page: "docs/goals/13-anomaly-arbitration-reference-data.md",
    locator: "Phase 1 G13-P1-B5",
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

const options = [
  {
    numericId: 3033066,
    nameEn: "Navigator's Oath",
    nameZh: "领航誓言",
    nameHash: "15001982558288419463",
    descHash: "8716817855956121302",
    descEn:
      "Increases the All-Type RES PEN for Skill and Ultimate damage dealt by the character in lineup position 1.",
    descZh: "提高编队第一位角色战技与终结技伤害的全属性抗性穿透。",
    params: ["0.5"],
    bindingKey: "ChallengePeakBattle_BaseAbility_Plugins_0022",
    bindingProgramState: "NamedInLayoutButMissingFromExtractedAbilityList",
    contribution: {
      trigger: "StageAbilityBeforeCharacterBorn",
      target: "LineupPosition1",
      attack_types: ["Skill", "Ultimate"],
      property: "AllTypeResistancePenetration",
      ratio: "0.5",
    },
  },
  {
    numericId: 3033067,
    nameEn: "Endless Euphoria",
    nameZh: "狂欢不息",
    nameHash: "1144961268741207686",
    descHash: "1083179235023776380",
    descEn:
      "Raises all allies' All-Type RES PEN and adds the same increase to Elation damage.",
    descZh: "提高我方全体全属性抗性穿透，并对欢愉伤害额外提高同等数值。",
    params: ["0.2", "0.2"],
    bindingKey: "ChallengePeakBattle_BaseAbility_Plugins_0023",
    bindingProgramState: "NamedInLayoutButMissingFromExtractedAbilityList",
    contribution: {
      trigger: "StageAbilityBeforeCharacterBorn",
      target: "AllAllies",
      property: "AllTypeResistancePenetration",
      base_ratio: "0.2",
      elation_additional_ratio: "0.2",
    },
  },
  {
    numericId: 3033068,
    nameEn: "Add Insult to Injury",
    nameZh: "落井下石",
    nameHash: "15416898346825293541",
    descHash: "14543835869706011294",
    descEn:
      "Follow-Up ATKs add a stacking damage-taken increase to the struck enemy.",
    descZh: "追加攻击命中敌方后，为目标叠加受到伤害提高效果。",
    params: ["0.15", "2", "3"],
    bindingKey: "ChallengePeakBattle_BaseAbility_Plugins_0014",
    bindingProgramState: "ResolvedInExtractedAbilityList",
    contribution: {
      trigger: "AfterEnemyHitByAllyFollowUpAttack",
      target: "HitEnemy",
      property: "AllDamageTakenRatio",
      ratio_per_stack: "0.15",
      duration_turns: 2,
      maximum_stacks: 3,
      stacking: "ReplaceWithLayerIncrement",
    },
  },
];

const optionRows = options.map((option) => envelope({
  id: `quadrant-option.${option.numericId}`,
  kind: "QuadrantOption",
  nameEn: option.nameEn,
  nameZh: option.nameZh,
  summaryEn: option.descEn,
  summaryZh: option.descZh,
  ownership: "Shared",
  evidenceQuality: "ExactStructured",
  mechanismQuality: option.bindingProgramState
      === "ResolvedInExtractedAbilityList"
    ? "ExactRelationship"
    : "PolicyBoundary",
  manifestIds: [manifestId(`quadrant:${option.numericId}`)],
  sources: [
    structuredRef(
      option.numericId,
      "Active alias 804 BuffList explicitly selects this MazeBuff row.",
    ),
    textRef("zh_cn", option.nameHash, option.nameZh),
    textRef("en", option.nameHash, option.nameEn),
    textRef("zh_cn", option.descHash, option.descZh),
    textRef("en", option.descHash, option.descEn),
  ],
  tags: ["arbitral-quadrant", "king", "option"],
  fields: {
    source_numeric_id: option.numericId,
    source_buff_level: 1,
    source_parameters: option.params,
    in_battle_binding_type: "StageAbilityBeforeCharacterBorn",
    in_battle_binding_key: option.bindingKey,
    binding_program_state: option.bindingProgramState,
    contribution: option.contribution,
    stage_scope: ["KingNormal", "KingPlight"],
    selection_policy_id: "quadrant-selection.active-period",
    teardown_policy_id: "quadrant-selection.attempt-teardown",
    ...(option.bindingProgramState === "ResolvedInExtractedAbilityList"
      ? {}
      : {
        approximations: [{
          field_path: "binding_program_state",
          unavailable_fact:
            "The MazeBuff and layout name the binding, but the fixed extracted ability JSON stops at Plugins_0021 and does not contain the 0022/0023 program body.",
          selected_policy:
            "Keep the exact released description and parameters while marking the program body unresolved and non-runtime.",
          alternatives: [
            "infer a program from the localized description",
            "substitute a similar older plugin",
          ],
          rationale:
            "Descriptions establish the authored contribution, but neither inference is byte-identical program evidence.",
          affected_fixture_ids: [
            `fixture.quadrant-selection.option-${option.numericId}`,
          ],
          confidence: "High",
          replacement_condition:
            "Replace when a released fixed-revision source exposes the named plugin body and its dynamic-value mapping.",
        }],
      }),
  },
}));

const offerFact = officialRef(
  "quadrant-select-one",
  "Each update supplies several Arbitral Quadrant buffs for the King challenge, and one may be selected before challenging the boss.",
);
const selectionRows = [
  envelope({
    id: "quadrant-selection.active-period",
    kind: "QuadrantSelectionPolicy",
    nameEn: "Active-period Quadrant offer",
    nameZh: "当期仲裁区选项集",
    summaryEn:
      "Before a King attempt, the active period offers exactly the three buffs selected by alias 804.",
    summaryZh:
      "王棋尝试开始前，当期按别名 804 提供恰好三个增益选项。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ExactStructured",
    mechanismQuality: "ExactRelationship",
    manifestIds: options.map(
      ({ numericId }) => manifestId(`quadrant:${numericId}`),
    ),
    sources: [
      offerFact,
      policyRef(
        "active-period-offer",
        "ChallengePeakBossConfig ID 804 BuffList order is [3033066, 3033068, 3033067].",
      ),
    ],
    tags: ["offer", "quadrant", "selection"],
    fields: {
      selection_order: 10,
      source_alias_id: "804",
      offered_option_ids: [
        "quadrant-option.3033066",
        "quadrant-option.3033068",
        "quadrant-option.3033067",
      ],
      offer_cardinality: 3,
      choose_count: 1,
      timing: "BeforeKingAttemptStart",
      eligible_stage_scopes: ["KingNormal", "KingPlight"],
    },
  }),
  envelope({
    id: "quadrant-selection.no-selection",
    kind: "QuadrantSelectionPolicy",
    nameEn: "No-selection boundary",
    nameZh: "未选择边界",
    summaryEn:
      "A King attempt does not start until one offered Quadrant option is selected.",
    summaryZh: "在选择一个当期仲裁区选项前，王棋尝试不会开始。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    manifestIds: options.map(
      ({ numericId }) => manifestId(`quadrant:${numericId}`),
    ),
    sources: [
      offerFact,
      policyRef(
        "no-selection",
        "Released text does not state cancel/default behavior.",
      ),
    ],
    tags: ["no-selection", "quadrant", "rejection"],
    fields: {
      selection_order: 20,
      legal_selection_count: 1,
      no_selection_result: "RejectAttemptStart",
      rejected_selection_state_effect: "AuthoritativeStateUnchanged",
      approximations: [{
        field_path: "no_selection_result",
        unavailable_fact:
          "Released text says one can pick a buff but does not state whether no selection cancels, defaults or starts without a buff.",
        selected_policy:
          "Reject attempt start until exactly one offered option is selected.",
        alternatives: [
          "start without a Quadrant contribution",
          "select the first option automatically",
        ],
        rationale:
          "Fail-closed rejection avoids an invented default and preserves deterministic choice.",
        affected_fixture_ids: [
          "fixture.quadrant-selection.no-selection",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace with released UI-independent transition evidence for boss start without an explicit selection.",
      }],
    },
  }),
  envelope({
    id: "quadrant-selection.attempt-teardown",
    kind: "QuadrantSelectionPolicy",
    nameEn: "Quadrant contribution teardown",
    nameZh: "仲裁区贡献拆除",
    summaryEn:
      "The selected contribution belongs to one King attempt and is removed when that attempt terminates.",
    summaryZh:
      "已选贡献仅属于一次王棋尝试，并在该尝试终止时移除。",
    ownership: "AnomalyArbitration",
    evidenceQuality: "ProjectPolicy",
    mechanismQuality: "PolicyBoundary",
    manifestIds: options.map(
      ({ numericId }) => manifestId(`quadrant:${numericId}`),
    ),
    sources: [
      offerFact,
      policyRef(
        "attempt-teardown",
        "Attempt ownership is explicit; released text does not expose modifier teardown order.",
      ),
    ],
    tags: ["attempt", "quadrant", "teardown"],
    fields: {
      selection_order: 30,
      contribution_start: "BeforeCharacterBornInAcceptedKingAttempt",
      contribution_end: "KingAttemptTerminal",
      carry_between_attempts: false,
      ordered_teardown: [
        "CommitKingAttemptTerminalOutcome",
        "RemoveSelectedQuadrantContribution",
        "ClearAttemptSelection",
      ],
      approximations: [{
        field_path: "ordered_teardown",
        unavailable_fact:
          "Released text and MazeBuff binding identify attempt application but do not expose terminal teardown ordering.",
        selected_policy:
          "Remove the contribution and clear selection immediately after the attempt terminal result is committed.",
        alternatives: [
          "clear selection before terminal commit",
          "retain selection for automatic retry",
        ],
        rationale:
          "Attempt-local teardown prevents an unchosen contribution from leaking into later battles.",
        affected_fixture_ids: [
          "fixture.quadrant-selection.attempt-teardown",
        ],
        confidence: "Medium",
        replacement_condition:
          "Replace with released modifier lifecycle or reproducible retry/exit traces.",
      }],
    },
  }),
];

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
  "quadrant-options.json": file(
    "quadrant-options.json",
    "QuadrantOption",
    optionRows,
  ),
  "quadrant-selections.json": file(
    "quadrant-selections.json",
    "QuadrantSelectionPolicy",
    selectionRows,
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
  `Anomaly Arbitration Quadrants generated: ${optionRows.length} options, `
    + `${selectionRows.length} selection rules.`,
);
