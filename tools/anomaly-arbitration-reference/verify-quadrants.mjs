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
const files = ["quadrant-options.json", "quadrant-selections.json"];
const kinds = {
  "quadrant-options.json": "QuadrantOption",
  "quadrant-selections.json": "QuadrantSelectionPolicy",
};

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/import-quadrants.mjs"),
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

const options = documents["quadrant-options.json"].records;
assert(options.length === 3, "Quadrant option count drift");
const expected = [
  [3033066, "Navigator's Oath", "领航誓言", ["0.5"],
    "ChallengePeakBattle_BaseAbility_Plugins_0022",
    "NamedInLayoutButMissingFromExtractedAbilityList"],
  [3033067, "Endless Euphoria", "狂欢不息", ["0.2", "0.2"],
    "ChallengePeakBattle_BaseAbility_Plugins_0023",
    "NamedInLayoutButMissingFromExtractedAbilityList"],
  [3033068, "Add Insult to Injury", "落井下石", ["0.15", "2", "3"],
    "ChallengePeakBattle_BaseAbility_Plugins_0014",
    "ResolvedInExtractedAbilityList"],
];
for (const [numericId, en, zh, parameters, binding, state] of expected) {
  const option = options.find(
    ({ source_numeric_id: id }) => id === numericId,
  );
  assert(option !== undefined
    && option.name_en === en
    && option.name_zh_cn === zh
    && JSON.stringify(option.source_parameters)
      === JSON.stringify(parameters)
    && option.in_battle_binding_type === "StageAbilityBeforeCharacterBorn"
    && option.in_battle_binding_key === binding
    && option.binding_program_state === state
    && JSON.stringify(option.stage_scope)
      === JSON.stringify(["KingNormal", "KingPlight"]),
  `Quadrant option ${numericId} drift`);
  assert(option.source_parameters.every(
    (value) => typeof value === "string"
      && /^(?:0|[1-9]\d*)(?:\.\d*[1-9])?$/u.test(value),
  ), `Quadrant option ${numericId} has noncanonical decimal parameters`);
}
const navigator = options.find(
  ({ source_numeric_id: id }) => id === 3033066,
);
assert(navigator.contribution.target === "LineupPosition1"
  && navigator.contribution.ratio === "0.5"
  && navigator.approximations.length === 1,
"Navigator contribution drift");
const euphoria = options.find(
  ({ source_numeric_id: id }) => id === 3033067,
);
assert(euphoria.contribution.target === "AllAllies"
  && euphoria.contribution.base_ratio === "0.2"
  && euphoria.contribution.elation_additional_ratio === "0.2"
  && euphoria.approximations.length === 1,
"Euphoria contribution drift");
const insult = options.find(
  ({ source_numeric_id: id }) => id === 3033068,
);
assert(insult.contribution.trigger
    === "AfterEnemyHitByAllyFollowUpAttack"
  && insult.contribution.ratio_per_stack === "0.15"
  && insult.contribution.duration_turns === 2
  && insult.contribution.maximum_stacks === 3
  && insult.approximations === undefined,
"Follow-Up contribution drift");

const selections = documents["quadrant-selections.json"].records;
assert(selections.length === 3
  && JSON.stringify(selections.map(
    ({ selection_order: order }) => order,
  )) === JSON.stringify([10, 20, 30]),
"Quadrant selection rule count/order drift");
const active = selections.find(
  ({ id }) => id === "quadrant-selection.active-period",
);
assert(active.source_alias_id === "804"
  && JSON.stringify(active.offered_option_ids) === JSON.stringify([
    "quadrant-option.3033066",
    "quadrant-option.3033068",
    "quadrant-option.3033067",
  ])
  && active.offer_cardinality === 3
  && active.choose_count === 1
  && active.timing === "BeforeKingAttemptStart",
"active Quadrant offer drift");
const none = selections.find(
  ({ id }) => id === "quadrant-selection.no-selection",
);
assert(none.no_selection_result === "RejectAttemptStart"
  && none.rejected_selection_state_effect === "AuthoritativeStateUnchanged"
  && none.evidence_quality === "ProjectPolicy"
  && none.approximations.length === 1,
"no-selection boundary drift");
const teardown = selections.find(
  ({ id }) => id === "quadrant-selection.attempt-teardown",
);
assert(teardown.contribution_start
    === "BeforeCharacterBornInAcceptedKingAttempt"
  && teardown.contribution_end === "KingAttemptTerminal"
  && teardown.carry_between_attempts === false
  && JSON.stringify(teardown.ordered_teardown) === JSON.stringify([
    "CommitKingAttemptTerminalOutcome",
    "RemoveSelectedQuadrantContribution",
    "ClearAttemptSelection",
  ]),
"Quadrant teardown drift");

const covered = new Set(Object.values(documents).flatMap(
  ({ records }) => records.flatMap(
    ({ manifest_record_ids: references }) => references,
  ),
));
const required = manifest.categories.quadrant_options.records.map(
  ({ id }) => `quadrant_options:${id}`,
);
assert(required.length === 3 && covered.size === 3,
  "Quadrant obligation denominator drift");
for (const reference of required)
  assert(covered.has(reference), `uncovered Quadrant option ${reference}`);

console.log(
  `Anomaly Arbitration Quadrants verified: ${files.map(
    (file) => `${file}=${createHash("sha256").update(encoded[file]).digest("hex")}`,
  ).join(" ")}`,
);
