#!/usr/bin/env node

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const check = process.argv.includes("--check");
const root = path.resolve(".");
const fragmentRoot = path.join(
  root,
  "content-reference",
  "galactic-baseballer-v1",
  "fragments",
);
const profileId = "galactic-baseballer.demon-king.v3_3";
const rowRevision = "starclock.galactic-baseballer-row.v1";
const read = async (file) =>
  JSON.parse(await readFile(path.join(fragmentRoot, file), "utf8"));

const stages = await read("demon-stages.json");
const periods = await read("demon-stage-periods.json");
const encounters = await read("demon-encounters.json");
const waves = await read("demon-waves.json");
const enemySlots = await read("demon-enemy-slots.json");
const enemies = await read("demon-enemies.json");
const statuses = await read("demon-enemy-statuses.json");
const scoring = await read("demon-scoring-rules.json");
const settlements = await read("demon-settlement-rules.json");
const teamBonuses = await read("demon-team-bonuses.json");
const bossPhases = await read("demon-boss-phases.json");
const corrections = await read("demon-released-corrections.json");

function required(rows, predicate, label) {
  const row = rows.find(predicate);
  if (row === undefined) throw new Error(`representative row missing: ${label}`);
  return row;
}
function refs(rows) {
  return rows.flatMap(({ source_refs: sourceRefs }) => sourceRefs)
    .filter((row, index, all) =>
      all.findIndex(({ source_id: id }) => id === row.source_id) === index);
}
function ids(rows) {
  return [...new Set(rows.flatMap(({ manifest_record_ids: values }) => values))]
    .sort();
}
function quality(rows) {
  if (rows.some(({ evidence_quality: value }) => value === "ProjectPolicy"))
    return "ProjectPolicy";
  if (rows.some(({ evidence_quality: value }) => value === "ExactPublicText"))
    return "ExactPublicText";
  return "ExactStructured";
}

const d007Stage = required(
  stages,
  ({ source_numeric_id: id }) => id === "424005",
  "D007 stage",
);
const d007Period = required(
  periods,
  ({ source_numeric_id: id }) => id === "424053",
  "D007 terminal period",
);
const d007Encounter = required(
  encounters,
  ({ source_stage_id: id }) => id === d007Period.shared_stage_config_id,
  "D007 terminal encounter",
);
const d007Wave = required(
  waves,
  ({ encounter_id: id }) => id === d007Encounter.id,
  "D007 wave",
);
const d007Candidate = required(
  enemySlots,
  ({ wave_id: id }) => id === d007Wave.id,
  "D007 candidate",
);
const d007Enemy = required(
  enemies,
  ({ inherited_enemy_variant_id: id }) =>
    id === d007Candidate.inherited_enemy_variant_id,
  "D007 enemy",
);
const d007Bonus = required(
  teamBonuses,
  ({ stage_id: id }) => id === d007Stage.id,
  "D007 team bonus",
);
const d007Settlement = required(
  settlements,
  ({ stage_id: id }) => id === d007Stage.id,
  "D007 settlement",
);
const d007Correction = required(
  corrections,
  ({ id }) => id.endsWith(".d007-adventure-score"),
  "D007 correction",
);
const bossStage = required(
  stages,
  ({ source_numeric_id: id }) => id === "424006",
  "Demon King's Den",
);
const bossPhase = bossPhases[0];
const bossSettlement = required(
  settlements,
  ({ stage_id: id }) => id === bossStage.id,
  "boss settlement",
);

const specs = [
  {
    family: "stage-difficulty-selection",
    rows: [d007Stage, d007Period],
    trigger: "after Demon King profile selection and before battle assembly",
    owner: "activity-local stage and difficulty selection",
    pre: { selected_profile_id: profileId, unlocks_satisfied: true },
    input: { stage_id: d007Stage.id, difficulty: d007Stage.difficulty },
    ops: [
      "validate exact Demon King stage ownership",
      "validate the released unlock edge",
      "project initial weapon, recommendations, team bonus and ordered periods",
    ],
    expected: {
      accepted: true,
      team_bonus_maze_buff_id: d007Stage.team_bonus_maze_buff_id,
      terminal_period_id: d007Period.id,
    },
  },
  {
    family: "wave-battle-phase-progression",
    rows: [
      d007Period,
      d007Encounter,
      d007Wave,
      d007Candidate,
      d007Enemy,
      ...statuses.slice(0, 1),
    ],
    trigger: "declared Demon King stage-period wave boundary",
    owner: "battle-local encounter and ordered wave state",
    pre: {
      encounter_id: d007Encounter.id,
      current_wave_order: d007Wave.wave_order,
    },
    input: { advance_to_wave_id: d007Wave.id },
    ops: [
      "resolve exact shared StageConfig and infinite group",
      "resolve ordered monster-group candidate positions",
      "reuse frozen enemy stable identities",
      "retain reachable status locators",
      "apply clear-previous-ability flag",
    ],
    expected: {
      wave_id: d007Wave.id,
      candidate_zero: d007Candidate.inherited_enemy_variant_id,
      candidate_semantics: d007Candidate.disposition,
      copied_enemy_definitions: 0,
    },
  },
  {
    family: "team-bonus",
    rows: [d007Stage, d007Bonus],
    trigger: "after D007 selection and before its first wave",
    owner: "stage-owned immutable bonus projected into battle-local state",
    pre: { stage_id: d007Stage.id },
    input: { install_team_bonus_id: d007Bonus.id },
    ops: [
      "resolve exact stage-owned MazeBuff",
      "validate structural program binding",
      "install before the first wave",
      "retain stage scope and remove at teardown",
    ],
    expected: {
      maze_buff_id: d007Bonus.source_maze_buff_id,
      binding_key: d007Bonus.binding_key,
      program_fragment_sha256: d007Bonus.program_fragment_sha256,
      runtime_executable: false,
    },
  },
  {
    family: "score-rating-clear",
    rows: [scoring[0], d007Period, d007Settlement, d007Correction],
    trigger: "D007 score contribution and terminal evaluation",
    owner: "battle-local score accumulator then activity-local result projection",
    pre: { stage_id: d007Stage.id, retained_version: "4.4" },
    input: {
      period_id: d007Period.id,
      special_monster_id: "403202302",
      terminal_evaluation: true,
    },
    ops: [
      "use only pinned post-correction Version 4.4 rows",
      "apply exact special-monster and stage-score facts",
      "sum explicit integer contributions",
      "apply the declared rounding boundary and score cap",
      "select the highest satisfied ordered rating",
    ],
    expected: {
      special_monster_score: 3000,
      stage_score: 4500,
      period_score: 45000,
      score_cap: 200000,
      obsolete_abnormal_path_modeled: false,
    },
  },
  {
    family: "boss-phase-final-settlement",
    rows: [bossStage, bossPhase, scoring[0], bossSettlement],
    trigger: "Demon King's Den boss phase and terminal settlement",
    owner: "battle-local Devil state and activity-local settlement result",
    pre: {
      stage_id: bossStage.id,
      ordered_period_count: bossPhase.ordered_period_ids.length,
    },
    input: {
      devil_card_id: bossPhase.source_devil_card_id,
      terminal_evaluation: true,
    },
    ops: [
      "preserve authored period order",
      "bind the exact Devil card and structural program",
      "finalize boss-HP contribution",
      "apply final-stage bonus once",
      "cap score, evaluate rating and project result once",
    ],
    expected: {
      ordered_period_count: 39,
      devil_card_id: "3113799",
      program_sha256: bossPhase.program_summary.whole_program_sha256,
      final_stage_extra_bonus: 5000,
      duplicate_projection: false,
      runtime_executable: false,
    },
  },
];

function rule(spec, ordinal) {
  const evidenceQuality = quality(spec.rows);
  return {
    id: `galactic-baseballer.demon-king.rule.${spec.family}`,
    schema_revision: rowRevision,
    kind: "MechanicRule",
    name_en: `Demon King rule: ${spec.family}`,
    name_zh_cn: `魔王篇机制规则：${spec.family}`,
    summary_en:
      "ReferenceOnly rule binding exact released facts and explicit deterministic boundaries.",
    summary_zh_cn: "仅供资料使用的规则，绑定正式发布事实与显式确定性边界。",
    profile_ids: [profileId],
    ownership: "DemonKing",
    coverage_state: "Researched",
    evidence_quality: evidenceQuality,
    mechanism_quality: evidenceQuality === "ExactStructured"
      ? "ExactRelationship"
      : "PolicyBoundary",
    manifest_record_ids: ids(spec.rows),
    source_refs: refs(spec.rows),
    tags: ["demon-king", "mechanic-rule", spec.family].sort(),
    family_id: spec.family,
    rule_order: ordinal,
    trigger_point: spec.trigger,
    state_owner: spec.owner,
    preconditions: spec.pre,
    ordered_operations: spec.ops.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    fixture_ids: [`galactic-baseballer.demon-king.fixture.${spec.family}`],
    runtime_executable: false,
  };
}
function fixture(spec) {
  const evidenceQuality = quality(spec.rows);
  return {
    id: `galactic-baseballer.demon-king.fixture.${spec.family}`,
    schema_revision: rowRevision,
    kind: "SemanticReviewFixture",
    name_en: `Demon King fixture: ${spec.family}`,
    name_zh_cn: `魔王篇语义夹具：${spec.family}`,
    summary_en: "Concrete ReferenceOnly review case; it does not run gameplay.",
    summary_zh_cn: "具体的仅供资料审查案例；不执行玩法。",
    profile_ids: [profileId],
    ownership: "DemonKing",
    coverage_state: "Researched",
    evidence_quality: evidenceQuality,
    mechanism_quality: evidenceQuality === "ExactStructured"
      ? "ExactRelationship"
      : "PolicyBoundary",
    manifest_record_ids: ids(spec.rows),
    source_refs: refs(spec.rows),
    tags: ["demon-king", "semantic-review", spec.family].sort(),
    family_id: spec.family,
    source_record_ids: spec.rows.map(({ id }) => id).sort(),
    trigger_point: spec.trigger,
    state_owner: spec.owner,
    preconditions: spec.pre,
    input: spec.input,
    ordered_operations: spec.ops.map((operation, operationOrder) => ({
      operation_order: operationOrder,
      operation,
    })),
    expected_facts: spec.expected,
    evidence_refs: refs(spec.rows).map(({ source_id: sourceId }) => sourceId),
    runtime_executable: false,
  };
}

const rules = specs.map(rule).sort((left, right) =>
  left.id.localeCompare(right.id, "en"));
const fixtures = specs.map(fixture);
fixtures.push({
  ...fixture(specs[3]),
  id: "galactic-baseballer.demon-king.fixture.d007-score-correction",
  name_en: "D007 retained Version 4.4 score correction fixture",
  name_zh_cn: "D007 Version 4.4 保留分数修正夹具",
  family_id: "score-rating-clear",
});
fixtures.sort((left, right) => left.id.localeCompare(right.id, "en"));
rules.find(({ family_id: family }) => family === "score-rating-clear")
  .fixture_ids.push(
    "galactic-baseballer.demon-king.fixture.d007-score-correction",
  );

const outputs = new Map([
  ["demon-encounter-mechanic-rules.json", rules],
  ["demon-encounter-review-fixtures.json", fixtures],
]);
await mkdir(fragmentRoot, { recursive: true });
for (const [file, value] of outputs) {
  const target = path.join(fragmentRoot, file);
  const encoded = `${JSON.stringify(value, null, 2)}\n`;
  if (check) {
    const existing = await readFile(target, "utf8");
    if (existing !== encoded)
      throw new Error(`Demon King encounter fixture drift: ${target}`);
  } else {
    await writeFile(target, encoded);
  }
}
console.log(
  `Demon King encounter fixtures ${check ? "verified" : "wrote"}: `
  + `${rules.length} rules and ${fixtures.length} fixtures`,
);
