#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = path.resolve(".");
const outputRoot = path.join(
  root,
  "content-reference",
  "anomaly-arbitration-v1",
);
const manifest = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/content-manifest.json",
), "utf8"));
const schema = JSON.parse(await readFile(path.join(
  root,
  "content-manifests/anomaly-arbitration-v1/normalized-schema.json",
), "utf8"));
const files = [
  "participant-policies.json",
  "team-slots.json",
  "loadout-records.json",
  "progress-records.json",
];
const kinds = {
  "participant-policies.json": "ParticipantPolicy",
  "team-slots.json": "TeamSlot",
  "loadout-records.json": "LoadoutRecordPolicy",
  "progress-records.json": "ProgressRecordPolicy",
};

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function resolveManifest(reference) {
  const separator = reference.indexOf(":");
  const category = reference.slice(0, separator);
  const id = reference.slice(separator + 1);
  const record = manifest.categories[category]?.records.find(
    (candidate) => candidate.id === id,
  );
  assert(record !== undefined, `unresolved manifest record: ${reference}`);
}

execFileSync(process.execPath, [
  path.join(
    root,
    "tools/anomaly-arbitration-reference/import-knight-records.mjs",
  ),
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
      === "starclock.anomaly-arbitration-normalized-file.v1",
    `${file} normalized revision drift`,
  );
  assert(document.goal_id === "anomaly-arbitration-reference-v1"
    && document.profile === "anomaly-arbitration-v1"
    && document.file === file
    && document.record_kind === kinds[file],
  `${file} normalized envelope drift`);
  for (const record of document.records) {
    for (const field of schema.common_envelope.required_fields)
      assert(record[field] !== undefined,
        `${file}/${record.id} lacks ${field}`);
    assert(record.kind === kinds[file]
      && record.coverage_state === "DataReady"
      && record.ownership === "AnomalyArbitration"
      && record.runtime_executable === false,
    `${file}/${record.id} row boundary drift`);
    assert(record.name_en && record.name_zh_cn
      && record.summary_en && record.summary_zh_cn,
    `${file}/${record.id} lacks bilingual text`);
    for (const reference of record.manifest_record_ids)
      resolveManifest(reference);
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

const policies = documents["participant-policies.json"].records;
assert(policies.length === 4, "participant policy count drift");
const threeTeams = policies.find(
  ({ id }) => id === "participant-policy.three-knight-team-slots",
);
assert(threeTeams.slot_count === 3
  && threeTeams.distinct_recorded_teams === true
  && threeTeams.king_team_participates_in_uniqueness_scope === false,
"three-team scope drift");
const character = policies.find(
  ({ id }) =>
    id === "participant-policy.character-and-combat-form-uniqueness",
);
assert(character.evidence_quality === "ProjectPolicy"
  && character.mechanism_quality === "PolicyBoundary"
  && character.approximations.length === 1,
"combat-form uncertainty boundary drift");
for (const [id, key] of [
  ["participant-policy.light-cone-instance-uniqueness",
    "account-light-cone-instance-id"],
  ["participant-policy.relic-instance-uniqueness",
    "account-relic-instance-id"],
]) {
  const policy = policies.find((record) => record.id === id);
  assert(policy.identity_key === key
    && policy.conflict_effect === "ResetSourceKnightProgress",
  `${id} identity/reset drift`);
}

const slots = documents["team-slots.json"].records;
assert(slots.length === 3, "team slot count drift");
for (const [index, slot] of slots.entries())
  assert(slot.id === `team-slot.knight-${index + 1}`
    && slot.slot_order === index + 1
    && slot.stage_id === `stage.knight-${index + 1}`
    && slot.record_state === "EmptyOrSuccessfulClearSnapshot",
  `Knight team slot ${index + 1} drift`);

const loadouts = documents["loadout-records.json"].records;
assert(loadouts.length === 3, "loadout policy count drift");
const invalidation = loadouts.find(
  ({ id }) => id === "loadout-record.cross-team-equipment-invalidation",
);
assert(JSON.stringify(invalidation.ordering) === JSON.stringify([
  "DetectRecordedInstanceConflict",
  "ResetSourceKnightProgress",
  "ClearSourceTeamSnapshot",
  "PermitTargetChallengeIfNoConflictsRemain",
]), "equipment conflict order drift");
const nonconflicting = loadouts.find(
  ({ id }) => id === "loadout-record.same-team-or-unrecorded-change",
);
assert(nonconflicting.retry_allowed === true
  && nonconflicting.old_record_retained_during_attempt === true
  && nonconflicting.success_resolution
    === "ExplicitKeepOldOrReplaceWithNewChoice"
  && nonconflicting.failed_attempt_resolution === "KeepOldRecord",
"nonconflicting retry resolution drift");

const progress = documents["progress-records.json"].records;
assert(progress.length === 6, "progress policy count drift");
assert(JSON.stringify(progress.map(({ projection_order: order }) => order))
  === JSON.stringify([10, 20, 30, 40, 50, 60]),
"progress projection order drift");
const replacement = progress.find(
  ({ id }) => id === "progress-record.record-replacement-choice",
);
assert(JSON.stringify(replacement.legal_choices)
  === JSON.stringify(["KeepOldRecord", "ReplaceWithNewRecord"])
  && replacement.rejection_effect === "AuthoritativeStateUnchanged",
"record replacement choice drift");
const erasure = progress.find(
  ({ id }) => id === "progress-record.record-erasure-on-reset",
);
assert(erasure.best_projection === "Unchanged"
  && JSON.stringify(erasure.ordered_projections) === JSON.stringify([
    "ClearRecordedTeamComposition",
    "InvalidateCurrentStageResult",
    "ReevaluateCurrentProgress",
  ]),
"current reset ordering drift");
const currentBest = progress.find(
  ({ id }) => id === "progress-record.current-versus-best-progress",
);
assert(currentBest.current_reset_affects_best === false
  && currentBest.best_progress_scope
    === "HighestSimultaneousThreeKnightStarTotal"
  && currentBest.detailed_aggregation_owner_batch === "G13-P1-B6",
"current/best separation drift");

const covered = new Set(Object.values(documents).flatMap(
  ({ records }) => records.flatMap(
    ({ manifest_record_ids: references }) => references,
  ),
));
const required = [
  ...manifest.categories.participant_policies.records.map(
    ({ id }) => `participant_policies:${id}`,
  ),
  ...manifest.categories.record_progress_lifecycles.records.map(
    ({ id }) => `record_progress_lifecycles:${id}`,
  ),
];
assert(required.length === 10, "P1-B2 denominator drift");
for (const reference of required)
  assert(covered.has(reference), `uncovered P1-B2 obligation: ${reference}`);
assert(covered.size === required.length,
  "P1-B2 unexpectedly covers another batch");

console.log(
  `Anomaly Arbitration Knight records verified: ${files.map(
    (file) => `${file}=${digest(encoded[file])}`,
  ).join(" ")}`,
);
