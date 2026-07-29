#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const outputRoot = path.join(root, "content-reference/anomaly-arbitration-v1");
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));
const files = [
  "targets.json",
  "objectives.json",
  "stage-results.json",
  "aggregations.json",
];
const kinds = {
  "targets.json": "BattleTarget",
  "objectives.json": "ObjectiveRule",
  "stage-results.json": "StageResultPolicy",
  "aggregations.json": "AggregationRule",
};

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/import-objectives.mjs"),
  "--check",
], { stdio: "inherit" });
const encoded = Object.fromEntries(await Promise.all(files.map(
  async (file) => [file, await readFile(path.join(outputRoot, file))],
)));
const documents = Object.fromEntries(Object.entries(encoded).map(
  ([file, bytes]) => [file, JSON.parse(bytes)],
));
for (const [file, document] of Object.entries(documents)) {
  assert(
    document.schema_revision
      === "starclock.anomaly-arbitration-normalized-file.v1"
    && document.goal_id === "anomaly-arbitration-reference-v1"
    && document.profile === "anomaly-arbitration-v1"
    && document.file === file
    && document.record_kind === kinds[file],
    `${file} envelope drift`,
  );
  for (const record of document.records) {
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined,
        `${file}/${record.id} lacks ${field}`);
    assert(record.kind === kinds[file]
      && ["AnomalyArbitration", "Shared"].includes(record.ownership)
      && record.coverage_state === "DataReady"
      && record.runtime_executable === false,
    `${file}/${record.id} boundary drift`);
    assert(record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn,
    `${file}/${record.id} lacks bilingual text`);
    for (const reference of record.manifest_record_ids) {
      const separator = reference.indexOf(":");
      const category = reference.slice(0, separator);
      const id = reference.slice(separator + 1);
      assert(manifest.categories[category]?.records.some(
        (candidate) => candidate.id === id,
      ), `${file}/${record.id} unresolved manifest record ${reference}`);
    }
    for (const source of record.source_refs) {
      for (const field of schema.types.source_ref.required_fields)
        assert(source[field] !== undefined && source[field] !== "",
          `${file}/${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256)
        && source.game_version === "4.4",
      `${file}/${record.id} source receipt drift`);
    }
    for (const approximation of record.approximations ?? []) {
      for (const field of schema.types.approximation.required_fields)
        assert(approximation[field] !== undefined,
          `${file}/${record.id} approximation lacks ${field}`);
      assert(approximation.alternatives.length > 0
        && approximation.affected_fixture_ids.length > 0
        && approximation.replacement_condition.length > 0,
      `${file}/${record.id} approximation is not replaceable`);
    }
  }
}

const targets = documents["targets.json"].records;
assert(targets.length === 7, "target count drift");
const expectedTargets = [
  [3000, "BattleTarget_DeathCount", 0],
  [3001, "BattleTarget_TurnLimit_PeakBattle_1", 4],
  [3002, "BattleTarget_TurnLimit_PeakBattle_2", 2],
  [3003, "BattleTarget_TurnLimit_PeakBattle_3", 6],
  [3004, "BattleTarget_TurnLimit_PeakBattle_4", 4],
  [3005, "BattleTarget_TurnLimit_PeakBattle_5", 2],
  [3007, "BattleTarget_TurnLimit_PeakBattle_7", 2],
];
for (const [id, ability, parameter] of expectedTargets) {
  const target = targets.find(({ source_numeric_id: value }) => value === id);
  assert(target !== undefined
    && target.source_ability_name === ability
    && target.comparison === "LessEqual"
    && target.comparison_parameter === parameter
    && target.evaluation_boundary === "SuccessfulStageTerminal"
    && target.satisfied_value === 1
    && target.unsatisfied_value === 0,
  `target ${id} drift`);
}

const objectives = documents["objectives.json"].records;
assert(objectives.length === 3
  && JSON.stringify(objectives.map(
    ({ evaluation_order: order }) => order,
  )) === JSON.stringify([10, 20, 30]),
"objective count/order drift");
const stageStars = objectives.find(
  ({ id }) => id === "objective.stage-star-evaluation",
);
assert(stageStars.boundary === "SuccessfulStageTerminal"
  && stageStars.combination_scope === "OneCompletedAttempt"
  && stageStars.combine_across_attempts === false
  && stageStars.star_value_rule === "CountSatisfiedActiveTargets",
"per-stage star evaluation drift");

const stageResults = documents["stage-results.json"].records;
assert(stageResults.length === 5
  && JSON.stringify(stageResults.map(({ stage_order: order }) => order))
    === JSON.stringify([1, 2, 3, 4, 5]),
"stage result count/order drift");
for (const index of [1, 2, 3]) {
  const result = stageResults.find(
    ({ id }) => id === `stage-result.knight-${index}`,
  );
  assert(JSON.stringify(result.target_ids) === JSON.stringify([
    "battle-target.3001",
    "battle-target.3002",
    "battle-target.3000",
  ]), `Knight ${index} target set drift`);
}
const normal = stageResults.find(
  ({ id }) => id === "stage-result.king-normal",
);
assert(JSON.stringify(normal.target_ids) === JSON.stringify([
  "battle-target.3003",
  "battle-target.3004",
  "battle-target.3005",
]), "normal King target set drift");
const plight = stageResults.find(
  ({ id }) => id === "stage-result.king-plight",
);
assert(JSON.stringify(plight.target_ids)
    === JSON.stringify(["battle-target.3007"])
  && plight.current_progress_projection
    === "ApplyPlightShortcutThenStoreKingResult",
"Plight target/projection drift");

const aggregations = documents["aggregations.json"].records;
assert(aggregations.length === 5
  && JSON.stringify(aggregations.map(
    ({ projection_order: order }) => order,
  )) === JSON.stringify([10, 20, 30, 40, 50]),
"aggregation count/order drift");
const best = aggregations.find(
  ({ id }) => id === "aggregation.simultaneous-three-knight-best",
);
assert(best.candidate_rule === "SumSimultaneouslyActiveKnightStars"
  && best.retention_rule === "MaximumObservedCandidate"
  && best.maximum_total === 9
  && best.current_reset_effect === "Unchanged",
"simultaneous Knight best drift");
const historical = aggregations.find(
  ({ id }) => id === "aggregation.retained-historical-best",
);
assert(historical.retained_period_count === 3
  && historical.structured_retention_days === 160
  && historical.expiry_warning_days === 14
  && historical.wall_clock_runtime_claim === false,
"historical retention locator drift");
const king = aggregations.find(
  ({ id }) => id === "aggregation.king-medal-rating",
);
assert(king.structured_color_medal_target === 6
  && king.color_medal_target_interpretation === "UnresolvedSourceField"
  && king.account_reward_projection === "Excluded"
  && king.approximations.length === 1,
"King rating boundary drift");

const covered = new Set(Object.values(documents).flatMap(
  ({ records }) => records.flatMap(
    ({ manifest_record_ids: references }) => references,
  ),
));
const required = [
  ...manifest.categories.battle_targets.records.map(
    ({ id }) => `battle_targets:${id}`,
  ),
  ...manifest.categories.objective_aggregations.records.map(
    ({ id }) => `objective_aggregations:${id}`,
  ),
];
assert(required.length === 12 && covered.size === 12,
  "P1-B6 denominator drift");
for (const reference of required)
  assert(covered.has(reference), `uncovered P1-B6 obligation ${reference}`);

console.log(
  `Anomaly Arbitration objectives verified: ${files.map(
    (file) => `${file}=${createHash("sha256").update(encoded[file]).digest("hex")}`,
  ).join(" ")}`,
);
