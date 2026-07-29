#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const contentRoot = path.join(
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
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests",
  "anomaly-arbitration-v1",
  "normalized-schema.json",
), "utf8"));
const sourceCache = process.env.STARCLOCK_SOURCE_CACHE
  ?? path.join(root, ".cache/content-reference");
const fallbackSourceCache = process.env.STARCLOCK_FALLBACK_SOURCE_CACHE
  ?? "/Users/mikai/.codex/worktrees/7c74/starclock/.cache/content-reference";

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function manifestRecord(reference) {
  const separator = reference.indexOf(":");
  assert(separator > 0, `invalid manifest reference: ${reference}`);
  const category = reference.slice(0, separator);
  const id = reference.slice(separator + 1);
  const record = manifest.categories[category]?.records.find(
    (candidate) => candidate.id === id,
  );
  assert(record !== undefined, `unresolved manifest reference: ${reference}`);
  return record;
}

execFileSync(process.execPath, [
  path.join(root, "tools/anomaly-arbitration-reference/import-profile.mjs"),
  "--check",
  "--source-cache",
  sourceCache,
  "--fallback-source-cache",
  fallbackSourceCache,
], { stdio: "inherit" });

const files = [
  "profiles.json",
  "periods.json",
  "stages.json",
  "terminal-outcomes.json",
];
const encoded = Object.fromEntries(await Promise.all(files.map(
  async (file) => [file, await readFile(path.join(contentRoot, file))],
)));
const documents = Object.fromEntries(Object.entries(encoded).map(
  ([file, bytes]) => [file, JSON.parse(bytes)],
));
const expectedKinds = {
  "profiles.json": "Profile",
  "periods.json": "Period",
  "stages.json": "Stage",
  "terminal-outcomes.json": "TerminalOutcome",
};
const requiredEnvelope = schema.common_envelope.required_fields;
const sourceFields = schema.types.source_ref.required_fields;
const approximationFields = schema.types.approximation.required_fields;

for (const [file, document] of Object.entries(documents)) {
  assert(
    document.schema_revision
      === "starclock.anomaly-arbitration-normalized-file.v1",
    `${file} normalized file revision drift`,
  );
  assert(document.goal_id === "anomaly-arbitration-reference-v1",
    `${file} goal drift`);
  assert(document.profile === "anomaly-arbitration-v1",
    `${file} profile drift`);
  assert(document.file === file, `${file} self-name drift`);
  assert(document.record_kind === expectedKinds[file],
    `${file} record kind drift`);
  for (const record of document.records) {
    for (const field of requiredEnvelope)
      assert(record[field] !== undefined,
        `${file}/${record.id} lacks ${field}`);
    assert(record.schema_revision === "starclock.anomaly-arbitration-row.v1",
      `${file}/${record.id} row revision drift`);
    assert(record.kind === expectedKinds[file],
      `${file}/${record.id} kind drift`);
    assert(record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn,
    `${file}/${record.id} lacks bilingual authoring text`);
    assert(record.ownership === "AnomalyArbitration",
      `${file}/${record.id} ownership drift`);
    assert(record.coverage_state === "DataReady",
      `${file}/${record.id} is not DataReady`);
    assert(record.runtime_executable === false,
      `${file}/${record.id} improperly claims runtime execution`);
    assert(record.manifest_record_ids.length > 0,
      `${file}/${record.id} has no manifest obligation`);
    for (const reference of record.manifest_record_ids)
      manifestRecord(reference);
    assert(record.source_refs.length > 0,
      `${file}/${record.id} has no source`);
    for (const source of record.source_refs) {
      for (const field of sourceFields)
        assert(source[field] !== undefined && source[field] !== "",
          `${file}/${record.id} source lacks ${field}`);
      assert(/^[0-9a-f]{64}$/u.test(source.sha256),
        `${file}/${record.id} has invalid evidence digest`);
      assert(source.game_version === "4.4",
        `${file}/${record.id} source version drift`);
    }
    for (const approximation of record.approximations ?? []) {
      for (const field of approximationFields)
        assert(approximation[field] !== undefined,
          `${file}/${record.id} approximation lacks ${field}`);
      assert(approximation.alternatives.length > 0,
        `${file}/${record.id} approximation lacks alternatives`);
      assert(approximation.affected_fixture_ids.length > 0,
        `${file}/${record.id} approximation lacks fixtures`);
      assert(approximation.replacement_condition.length > 0,
        `${file}/${record.id} approximation lacks replacement condition`);
    }
  }
}

const [profile] = documents["profiles.json"].records;
assert(documents["profiles.json"].records.length === 1,
  "profile count drift");
assert(profile.id === "anomaly-arbitration-v1"
  && profile.availability === "PermanentWithPeriodicStages"
  && profile.minimum_equilibrium_level === 6,
"profile identity or eligibility drift");
assert(JSON.stringify(profile.participation_requirements) === JSON.stringify([
  "equilibrium-level-at-least-6",
  "maximum-stars-in-highest-memory-of-chaos-stage",
  "maximum-stars-in-highest-pure-fiction-stage",
  "maximum-stars-in-highest-apocalyptic-shadow-stage",
]), "participation requirements drift");
assert(profile.requirements_may_be_completed_in_different_versions === true,
  "cross-version eligibility rule drift");
assert(JSON.stringify(profile.entry_locators) === JSON.stringify({
  prerequisite_quest_id: "2200506",
  entrance_map_id: "2116",
  entrance_id: "100000352",
  tutorial_mission_id: "8036301",
}), "entry locator drift");

const [period] = documents["periods.json"].records;
assert(documents["periods.json"].records.length === 1
  && period.id === "period.8"
  && period.source_group_id === "8"
  && period.active_at_snapshot === true,
"active period identity drift");
assert(period.observed_end_dates.length === 2,
  "conflicting public end-date observations were not retained");
assert(period.approximations?.some(
  ({ field_path: field }) => field === "observed_end_dates",
), "period end-date uncertainty is not field-level");
assert(period.canonical_end_instant === undefined,
  "unproven canonical period end instant was invented");

const stages = documents["stages.json"].records;
assert(stages.length === 5, "stage count drift");
const expectedStages = [
  ["stage.knight-1", "801", "30508011", "Knight", "Normal", 95,
    "ArbitraryAmongKnights"],
  ["stage.knight-2", "802", "30508012", "Knight", "Normal", 95,
    "ArbitraryAmongKnights"],
  ["stage.knight-3", "803", "30508013", "Knight", "Normal", 95,
    "ArbitraryAmongKnights"],
  ["stage.king-normal", "804", "30508021", "King", "Normal", 100,
    "AfterThreeKnightClears"],
  ["stage.king-plight", "804", "30508022", "King", "Plight", 120,
    "DirectAlternative"],
];
for (const [
  id,
  alias,
  stageId,
  kind,
  difficulty,
  level,
  order,
] of expectedStages) {
  const record = stages.find((candidate) => candidate.id === id);
  assert(record !== undefined, `missing stage ${id}`);
  assert(record.source_alias_id === alias
    && record.source_stage_id === stageId
    && record.stage_kind === kind
    && record.difficulty === difficulty
    && record.level === level
    && record.legal_order === order,
  `stage relationship drift: ${id}`);
}
const normalKing = stages.find(({ id }) => id === "stage.king-normal");
assert(normalKing.evidence_quality === "ApproximateFromReleasedText"
  && normalKing.mechanism_quality === "PolicyBoundary"
  && normalKing.approximations.length === 1,
"normal King availability uncertainty lost");
const plight = stages.find(({ id }) => id === "stage.king-plight");
assert(plight.source_refs.some(
  ({ source_id: id }) => id.endsWith(":direct-plight-clear"),
), "official direct Plight path is missing");

const outcomes = documents["terminal-outcomes.json"].records;
assert(outcomes.length === 4, "terminal outcome count drift");
assert(JSON.stringify(outcomes.map(({ id }) => id)) === JSON.stringify([
  "outcome.king-normal-clear",
  "outcome.king-plight-clear",
  "outcome.knight-stage-clear",
  "outcome.stage-attempt-failure",
]), "terminal outcome identity/order drift");
for (const outcome of outcomes)
  assert(
    outcome.detailed_projection_owner_batch === (
      outcome.id === "outcome.king-plight-clear"
        ? "G13-P1-B3"
        : "G13-P1-B6"
    ),
    `${outcome.id} projection owner drift`,
  );
assert(outcomes.filter(({ result }) => result === "Success").length === 3
  && outcomes.filter(({ result }) => result === "Failure").length === 1,
"terminal result partition drift");

const covered = new Set(Object.values(documents).flatMap(
  ({ records }) => records.flatMap(
    ({ manifest_record_ids: references }) => references,
  ),
));
const required = [
  "profiles:anomaly-arbitration-v1",
  "active_periods:period:8",
  ...[
    "ChallengePeak_Pre_Quest",
    "ChallengePeak_Entrance_MapInfo",
    "ChallengePeak_Entrance",
    "ChallengePeak_TutorialMissionID",
  ].map((id) => `mode_constants:constant:${id}`),
  ...["801", "802", "803", "804"].map(
    (id) => `stage_definitions:alias:${id}`,
  ),
  ...["30508011", "30508012", "30508013", "30508021", "30508022"].map(
    (id) => `stage_configs:stage:${id}`,
  ),
  "boss_difficulty_definitions:boss:804:plight",
  ...[
    "king-normal-clear",
    "king-plight-clear",
    "knight-stage-clear",
    "stage-attempt-failure",
  ].map((id) => `terminal_outcomes:${id}`),
];
for (const reference of required)
  assert(covered.has(reference), `P1-B1 obligation is uncovered: ${reference}`);
assert(covered.size === required.length,
  "P1-B1 unexpectedly covers another batch's obligation");

console.log(
  `Anomaly Arbitration profile verified: ${files.map((file) =>
    `${file}=${digest(encoded[file])}`).join(" ")}`,
);
